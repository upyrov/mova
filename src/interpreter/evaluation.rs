use std::{cell::RefCell, rc::Rc};

use crate::{
    error::{MovaError, Result},
    interpreter::{data::Value, scope::Scope},
    parser::{expression::Expression, node::Node, statement::Statement},
};

fn evaluate_binary_expression(operator: &str, left: Value, right: Value) -> Result<Value> {
    match (operator, left, right) {
        ("+", Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
        ("-", Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
        ("*", Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
        ("/", Value::Number(l), Value::Number(r)) => {
            if r == 0 {
                return Err(MovaError::Runtime("Division by zero".into()));
            }
            Ok(Value::Number(l / r))
        }
        (o, l, r) => Err(MovaError::Runtime(format!(
            "Unexpected operator '{o}' for operands '{l:?}' and '{r:?}'",
        ))),
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
                return Err(MovaError::Runtime(format!(
                    "Expected {parameter_count} arguments but received {argument_count}",
                )));
            }

            let evaluated_arguments: Vec<Value> = arguments
                .iter()
                .map(|argument| {
                    let node = Rc::new(Node::Expression(Rc::new(argument.clone())));
                    let value = evaluate(node, Rc::clone(&scope))?.ok_or(MovaError::Runtime(
                        "Expected expression, but received statement as argument".into(),
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

            evaluate(Rc::new(Node::Expression(Rc::clone(&body))), execution_scope)
        }
        _ => Err(MovaError::Runtime(format!("'{name}' is not callable",))),
    }
}

fn evaluate_expression(
    expression: Rc<Expression>,
    scope: Rc<RefCell<Scope>>,
) -> Result<Option<Value>> {
    match &*expression {
        Expression::Number(n) => Ok(Some(Value::Number(*n))),
        Expression::Boolean(b) => Ok(Some(Value::Boolean(*b))),
        Expression::Identifier(i) => Ok(Some(scope.borrow_mut().resolve(i)?)),
        Expression::Reference { name, is_mutable } => {
            let mut env = scope.borrow_mut();
            let value = if *is_mutable {
                env.borrow_mut(name)?
            } else {
                env.borrow(name)?
            };
            Ok(Some(value))
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
                "Expected expression, but received statement as left operand".into(),
            ))?;

            let right = evaluate(
                Rc::new(Node::Expression(Rc::clone(right))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                "Expected expression, but received statement as right operand".into(),
            ))?;

            Ok(Some(evaluate_binary_expression(&operator, left, right)?))
        }
        Expression::Call { name, arguments } => evaluate_call(scope, &name, Rc::clone(arguments)),
        Expression::Block(b) => {
            let child_scope = Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope)))));
            let mut result = None;
            for node in b.into_iter() {
                result = evaluate(Rc::new(node.clone()), Rc::clone(&child_scope))?;
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
                "Expected expression, but received statement as value".into(),
            ))?;
            scope.borrow_mut().declare(&name, value, *is_mutable);
        }
        Statement::Assignment { name, value } => {
            let new_value = evaluate(
                Rc::new(Node::Expression(Rc::clone(value))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                "Expected expression, but received statement as value".into(),
            ))?;
            let slot = scope.borrow().find_slot(name)?;
            let maybe_ref = {
                let data = slot.borrow();
                match &data.value {
                    Value::Reference(r) => Some(r.clone()),
                    _ => None,
                }
            };
            if let Some(reference) = maybe_ref {
                let mut old_value = reference.write()?;
                old_value.value = new_value;
            } else {
                let mut data = slot.borrow_mut();
                if data.is_mutable {
                    data.value = new_value;
                } else {
                    return Err(MovaError::Runtime(
                        format!("Cannot assign to immutable variable '{}'", name).into(),
                    ));
                }
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
            scope.borrow_mut().declare(&name, function, false);
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
