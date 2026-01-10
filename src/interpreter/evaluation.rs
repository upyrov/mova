use std::{cell::RefCell, rc::Rc};

use crate::{
    error::{MovaError, Result},
    interpreter::{
        data::{Data, Slot, State, Value},
        reference::Reference,
        scope::Scope,
    },
    parser::{expression::Expression, node::Node, statement::Statement},
};

/// Peels off Value::Reference layers to get to the actual data
fn resolve_value(value: Value) -> Result<Value> {
    match value {
        Value::Reference(r) => {
            let data = r.read()?; // checks for deallocated state
            if let Value::Moved = data.value {
                return Err(MovaError::Runtime("Cannot read from moved value".into()));
            }

            // Recursively resolve in case of reference chains
            resolve_value(data.value.clone())
        }
        val => Ok(val),
    }
}

fn evaluate_binary_expression(operator: &str, left: Value, right: Value) -> Result<Value> {
    // Resolve operands (auto-dereference)
    let left_val = resolve_value(left)?;
    let right_val = resolve_value(right)?;

    match (operator, left_val, right_val) {
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

            let result = evaluate(
                Rc::new(Node::Expression(Rc::clone(&body))),
                Rc::clone(&execution_scope),
            );

            execution_scope.borrow_mut().invalidate();

            result
        }
        _ => Err(MovaError::Runtime(format!("'{name}' is not callable",))),
    }
}

fn evaluate_slot(expression: &Expression, scope: Rc<RefCell<Scope>>) -> Result<Slot> {
    match expression {
        Expression::Identifier(name) => scope.borrow().find_slot(name),
        _ => Err(MovaError::Runtime("Expression cannot be referenced".into())),
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
            // Auto-dereference on read
            let val = scope.borrow_mut().resolve(i)?;
            if let Value::Reference(r) = val {
                let data = r.read()?;
                if let Value::Moved = data.value {
                    return Err(MovaError::Runtime("Cannot read from moved value".into()));
                }
                return Ok(Some(data.value.clone()));
            }
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
                    "Reference target yielded no value".into(),
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
                "Expected expression, but received statement as left operand".into(),
            ))?;

            let right = evaluate(
                Rc::new(Node::Expression(Rc::clone(right))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                "Expected expression, but received statement as right operand".into(),
            ))?;

            Ok(Some(evaluate_binary_expression(operator, left, right)?))
        }
        Expression::Call { name, arguments } => evaluate_call(scope, name, Rc::clone(arguments)),
        Expression::Block(b) => {
            let child_scope = Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope)))));
            let mut result = None;
            for node in b.into_iter() {
                result = evaluate(Rc::new(node.clone()), Rc::clone(&child_scope))?;
            }

            child_scope.borrow_mut().invalidate();

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
            scope.borrow_mut().declare(name, value, *is_mutable);
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
            let mut data = slot.borrow_mut();

            if let crate::interpreter::data::State::Deallocated = data.state {
                return Err(MovaError::Runtime(
                    format!("Cannot assign to deallocated variable '{}'", name).into(),
                ));
            }

            if data.is_mutable {
                data.value = new_value;
            } else {
                return Err(MovaError::Runtime(
                    format!("Cannot assign to immutable variable '{}'", name).into(),
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
