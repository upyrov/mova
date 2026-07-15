use std::{cell::RefCell, rc::Rc};

use crate::{
    error::{MovaError, Result, RuntimeError},
    interpreter::{
        data::{Data, Slot, State, Value},
        reference::Reference,
        scope::Scope,
    },
    parser::{expression::Expression, node::Node, statement::Statement},
};

fn evaluate_binary_expression(operator: &str, left: Value, right: Value) -> Result<Value> {
    match (operator, left, right) {
        ("+", Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
        ("-", Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
        ("*", Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
        ("/", Value::Number(l), Value::Number(r)) => {
            if r == 0 {
                return Err(MovaError::Runtime(RuntimeError::DivisionByZero));
            }
            Ok(Value::Number(l / r))
        }
        ("<", Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
        (">", Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
        ("==", Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l == r)),
        (o, l, r) => Err(MovaError::Runtime(RuntimeError::UnexpectedOperator {
            operator: o.to_string(),
            left: format!("{l:?}"),
            right: format!("{r:?}"),
        })),
    }
}

fn evaluate_call(
    scope: Rc<RefCell<Scope>>,
    name: &str,
    arguments: Rc<[Expression]>,
) -> Result<Option<Value>> {
    // Drop immediately after use so that recursive calls don't panic
    let callee = { scope.borrow_mut().resolve(name)? };
    match callee {
        Value::Function {
            parameters,
            body,
            definition_scope,
        } => {
            let argument_count = arguments.len();
            let parameter_count = parameters.len();
            if argument_count != parameter_count {
                return Err(MovaError::Runtime(RuntimeError::InvalidArgumentCount {
                    expected: parameter_count,
                    received: argument_count,
                }));
            }

            let evaluated_arguments: Vec<Value> = arguments
                .iter()
                .map(|argument| {
                    let node = Rc::new(Node::Expression(Rc::new(argument.clone())));
                    let value = evaluate(node, Rc::clone(&scope))?.ok_or(MovaError::Runtime(
                        RuntimeError::ExpectedExpressionAsArgument,
                    ))?;
                    Ok(value)
                })
                .collect::<Result<Vec<Value>>>()?;

            // Create execution scope in order to avoid interfering with other calls
            let execution_scope =
                Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&definition_scope)))));
            {
                let mut s = execution_scope.borrow_mut();

                // Map arguments to parameters
                evaluated_arguments
                    .into_iter()
                    .zip(parameters.iter())
                    .for_each(|(value, parameter)| s.declare(parameter, value, false));
            }

            let result = evaluate(
                Rc::new(Node::Expression(Rc::clone(&body))),
                Rc::clone(&execution_scope),
            );

            execution_scope.borrow_mut().invalidate();

            result
        }
        _ => Err(MovaError::Runtime(RuntimeError::NotCallable(name.to_string()))),
    }
}

fn evaluate_slot(expression: &Expression, scope: Rc<RefCell<Scope>>) -> Result<Slot> {
    match expression {
        Expression::Identifier(name) => scope.borrow().find_slot(name),
        _ => Err(MovaError::Runtime(RuntimeError::ExpressionCannotBeReferenced)),
    }
}

fn evaluate_expression(
    expression: Rc<Expression>,
    scope: Rc<RefCell<Scope>>,
) -> Result<Option<Value>> {
    match &*expression {
        Expression::Number(n) => Ok(Some(Value::Number(*n))),
        Expression::Boolean(b) => Ok(Some(Value::Boolean(*b))),
        Expression::Identifier(i) => {
            let val = scope.borrow_mut().resolve(i)?;
            Ok(Some(val))
        }
        Expression::Reference {
            data: target_data,
            is_mutable,
        } => {
            let is_lvalue = matches!(**target_data, Expression::Identifier(_));

            let slot = if is_lvalue {
                evaluate_slot(target_data, Rc::clone(&scope))?
            } else {
                let val = evaluate(
                    Rc::new(Node::Expression(Rc::clone(target_data))),
                    Rc::clone(&scope),
                )?
                .ok_or(MovaError::Runtime(
                    RuntimeError::ReferenceTargetYieldedNoValue,
                ))?;

                Rc::new(RefCell::new(Data {
                    value: val,
                    state: State::Free,
                    is_mutable: *is_mutable,
                }))
            };

            let reference = Reference::new(slot, *is_mutable)?;
            Ok(Some(Value::Reference(Rc::new(reference))))
        }
        Expression::BinaryExpression {
            operator,
            left,
            right,
        } => {
            let left = evaluate(
                Rc::new(Node::Expression(Rc::clone(left))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                RuntimeError::ExpectedExpressionAsLeftOperand,
            ))?;

            let right = evaluate(
                Rc::new(Node::Expression(Rc::clone(right))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                RuntimeError::ExpectedExpressionAsRightOperand,
            ))?;

            Ok(Some(evaluate_binary_expression(operator, left, right)?))
        }
        Expression::Call { name, arguments } => evaluate_call(scope, name, Rc::clone(arguments)),
        Expression::Dereference(inner) => {
            let val = evaluate(
                Rc::new(Node::Expression(Rc::clone(inner))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                RuntimeError::DereferenceTargetYieldedNoValue,
            ))?;

            if let Value::Reference(r) = val {
                let data = r.read()?;
                if let Value::Moved = data.value {
                    return Err(MovaError::Runtime(RuntimeError::CannotReadFromMovedValue));
                }
                Ok(Some(data.value.clone()))
            } else {
                Err(MovaError::Runtime(
                    RuntimeError::CannotDereferenceNonReferenceValue,
                ))
            }
        }
        Expression::Block(b) => {
            let child_scope = Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope)))));
            let mut result = None;
            for node in b.into_iter() {
                result = evaluate(Rc::new(node.clone()), Rc::clone(&child_scope))?;
            }

            child_scope.borrow_mut().invalidate();

            Ok(result)
        }
        Expression::If {
            condition,
            consequence,
            alternative,
        } => {
            let condition_value = evaluate(
                Rc::new(Node::Expression(Rc::clone(condition))),
                Rc::clone(&scope),
            )?
            .ok_or_else(|| MovaError::Runtime(RuntimeError::ConditionYieldedNoValue))?;

            match condition_value {
                Value::Boolean(true) => evaluate(
                    Rc::new(Node::Expression(Rc::clone(consequence))),
                    Rc::clone(&scope),
                ),
                Value::Boolean(false) => {
                    if let Some(alt) = alternative {
                        evaluate(
                            Rc::new(Node::Expression(Rc::clone(alt))),
                            Rc::clone(&scope),
                        )
                    } else {
                        Ok(None)
                    }
                }
                _ => Err(MovaError::Runtime(RuntimeError::ConditionMustBeBoolean)),
            }
        }
        Expression::While { condition, body } => {
            let mut result = None;
            loop {
                let condition_value = evaluate(
                    Rc::new(Node::Expression(Rc::clone(condition))),
                    Rc::clone(&scope),
                )?
                .ok_or_else(|| MovaError::Runtime(RuntimeError::ConditionYieldedNoValue))?;

                match condition_value {
                    Value::Boolean(true) => {
                        result = evaluate(
                            Rc::new(Node::Expression(Rc::clone(body))),
                            Rc::clone(&scope),
                        )?;
                    }
                    Value::Boolean(false) => break,
                    _ => return Err(MovaError::Runtime(RuntimeError::ConditionMustBeBoolean)),
                }
            }
            Ok(result)
        }
        Expression::Program(p) => {
            let mut result = None;
            for node in p.into_iter() {
                result = evaluate(Rc::new(node.clone()), Rc::clone(&scope))?;
            }
            Ok(result)
        }
    }
}

fn evaluate_statement(statement: Rc<Statement>, scope: Rc<RefCell<Scope>>) -> Result<()> {
    match &*statement {
        Statement::Variable {
            name,
            value,
            is_mutable,
        } => {
            let value = evaluate(
                Rc::new(Node::Expression(Rc::clone(value))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                RuntimeError::ExpectedExpressionAsValue,
            ))?;
            scope.borrow_mut().declare(name, value, *is_mutable);
        }
        Statement::Assignment { name, value } => {
            let new_value = evaluate(
                Rc::new(Node::Expression(Rc::clone(value))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                RuntimeError::ExpectedExpressionAsValue,
            ))?;

            let slot = scope.borrow().find_slot(name)?;
            let mut data = slot.borrow_mut();

            match data.state {
                State::Deallocated => {
                    return Err(MovaError::Runtime(
                        RuntimeError::CannotAssignToDeallocatedVariable(name.to_string()),
                    ));
                }
                State::Borrowed(count) if count > 0 => {
                    return Err(MovaError::Runtime(
                        RuntimeError::CannotAssignToBorrowedVariable(name.to_string()),
                    ));
                }
                State::MutablyBorrowed => {
                    return Err(MovaError::Runtime(
                        RuntimeError::CannotAssignToMutablyBorrowedVariable(name.to_string()),
                    ));
                }
                _ => {}
            }

            if data.is_mutable {
                data.value = new_value;
            } else {
                return Err(MovaError::Runtime(
                    RuntimeError::CannotAssignToImmutableVariable(name.to_string()),
                ));
            }
        }
        Statement::Function {
            name,
            parameters,
            body,
        } => {
            let function = Value::Function {
                parameters: Rc::clone(parameters),
                body: Rc::clone(body),
                definition_scope: Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope))))),
            };
            scope.borrow_mut().declare(name, function, false);
        }
        Statement::DereferenceAssignment { target, value } => {
            let target_val = evaluate(
                Rc::new(Node::Expression(Rc::clone(target))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                RuntimeError::DereferenceTargetYieldedNoValue,
            ))?;

            let new_value = evaluate(
                Rc::new(Node::Expression(Rc::clone(value))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                RuntimeError::AssignmentValueYieldedNoValue,
            ))?;

            if let Value::Reference(r) = target_val {
                let mut data = r.write()?;
                data.value = new_value;
            } else {
                return Err(MovaError::Runtime(
                    RuntimeError::CannotDereferenceNonReferenceValue,
                ));
            }
        }
    }
    Ok(())
}

pub fn evaluate(node: Rc<Node>, scope: Rc<RefCell<Scope>>) -> Result<Option<Value>> {
    match &*node {
        Node::Expression(e) => evaluate_expression(Rc::clone(e), scope),
        Node::Statement(s) => {
            evaluate_statement(Rc::clone(s), scope)?;
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::run;

    #[test]
    fn test_cannot_assign_to_borrowed_variable() {
        let input = "
            let mut x = 10
            let y = &x
            x = 20
        ";
        let result = run(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot assign to borrowed variable 'x'")
        );
    }

    #[test]
    fn test_cannot_assign_to_mutably_borrowed_variable() {
        let input = "
            let mut x = 10
            let y = &mut x
            x = 20
        ";
        let result = run(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot assign to mutably borrowed variable 'x'")
        );
    }

    #[test]
    fn test_can_assign_after_borrow_ends() {
        let input = "
            let mut x = 10
            {
                let y = &x
            }
            x = 20
            x
        ";
        let result = run(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Value::Number(20)));
    }

    #[test]
    fn test_explicit_dereference() {
        let input = "
            let x = 10;
            let y = &x;
            *y
        ";
        let result = run(input);
        match &result {
            Ok(val) => assert_eq!(val, &Some(Value::Number(10))),
            Err(e) => panic!("Test failed with error: {}", e),
        }
    }

    #[test]
    fn test_dereference_assignment() {
        let input = "
            let mut x = 10;
            let y = &mut x;
            *y = 20;
            x
        ";
        let result = run(input);
        match &result {
            Ok(val) => assert_eq!(val, &Some(Value::Number(20))),
            Err(e) => panic!("Test failed with error: {}", e),
        }
    }

    #[test]
    fn test_cannot_dereference_immutable_reference_for_assignment() {
        let input = "
            let mut x = 10;
            let y = &x;
            *y = 20;
        ";
        let result = run(input);
        match &result {
            Ok(val) => panic!("Test should have failed but succeeded with: {:?}", val),
            Err(e) => assert!(
                e.to_string()
                    .contains("Cannot assign to an immutable reference"),
                "Error message was: {}",
                e
            ),
        }
    }

    #[test]
    fn test_auto_dereference_in_binary_expression_is_no_longer_supported() {
        let input = "
            let x = 10;
            let y = &x;
            y + 5
        ";
        let result = run(input);
        match &result {
            Ok(val) => panic!("Test should have failed but succeeded with: {:?}", val),
            Err(e) => assert!(
                e.to_string().contains("Unexpected operator '+'"),
                "Error message was: {}",
                e
            ),
        }
    }

    #[test]
    fn test_reference_move_semantics() {
        let input = "
            let x = 10;
            let y = &x;
            let z = y;
            y
        ";
        let result = run(input);
        match &result {
            Ok(val) => panic!("Test should have failed but succeeded with: {:?}", val),
            Err(e) => assert!(
                e.to_string()
                    .contains("Unable to use 'y' because it is moved"),
                "Error message was: {}",
                e
            ),
        }
    }

    #[test]
    fn test_if_expression() {
        let input = "
            let mut x = 10;
            if true {
                x = 20;
            }
            x
        ";
        let result = run(input);
        assert_eq!(result.unwrap(), Some(Value::Number(20)));
    }

    #[test]
    fn test_if_else_expression() {
        let input = "
            let mut x = 10;
            if false {
                x = 20;
            } else {
                x = 30;
            }
            x
        ";
        let result = run(input);
        assert_eq!(result.unwrap(), Some(Value::Number(30)));
    }

    #[test]
    fn test_if_else_if_expression() {
        let input = "
            let mut x = 10;
            if false {
                x = 20;
            } else if true {
                x = 40;
            } else {
                x = 30;
            }
            x
        ";
        let result = run(input);
        assert_eq!(result.unwrap(), Some(Value::Number(40)));
    }

    #[test]
    fn test_while_loop() {
        let input = "
            let mut x = 0;
            while x < 5 {
                x = x + 1;
            }
            x
        ";
        let result = run(input);
        assert_eq!(result.unwrap(), Some(Value::Number(5)));
    }
}
