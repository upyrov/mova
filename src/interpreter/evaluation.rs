use std::{cell::RefCell, rc::Rc};

use crate::{
    error::{MovaError, Result},
    interpreter::{data::Data, scope::Scope},
    parser::{expression::Expression, node::Node, statement::Statement},
};

fn evaluate_binary_expression(operator: &str, left: Data, right: Data) -> Result<Data> {
    match (operator, left, right) {
        ("+", Data::Number(l), Data::Number(r)) => Ok(Data::Number(l + r)),
        ("-", Data::Number(l), Data::Number(r)) => Ok(Data::Number(l - r)),
        ("*", Data::Number(l), Data::Number(r)) => Ok(Data::Number(l * r)),
        ("/", Data::Number(l), Data::Number(r)) => {
            if r == 0 {
                Err(MovaError::Runtime("Division by zero".into()))
            } else {
                Ok(Data::Number(l / r))
            }
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
) -> Result<Option<Data>> {
    // Drop immediately after use so that recursive calls don't panic
    let function_data = { scope.borrow_mut().resolve(name)? };

    match function_data {
        Data::Function {
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

            let evaluated_arguments: Vec<Data> = arguments
                .iter()
                .map(|argument| {
                    let node = Rc::new(Node::Expression(Rc::new(argument.clone())));
                    let data = evaluate(node, Rc::clone(&scope))?.ok_or(MovaError::Runtime(
                        "Expected expression, but received statement as argument".into(),
                    ))?;
                    Ok(data)
                })
                .collect::<Result<Vec<Data>>>()?;

            // Avoid interfering with other calls
            let execution_scope =
                Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&definition_scope)))));
            {
                let mut s = execution_scope.borrow_mut();

                // Map arguments to parameters
                evaluated_arguments
                    .into_iter()
                    .zip(parameters.iter())
                    .for_each(|(data, parameter)| s.declare(parameter, data));
            }

            evaluate(Rc::new(Node::Expression(Rc::clone(&body))), execution_scope)
        }
        _ => Err(MovaError::Runtime(format!("'{name}' is not callable",))),
    }
}

fn evaluate_expression(
    expression: Rc<Expression>,
    scope: Rc<RefCell<Scope>>,
) -> Result<Option<Data>> {
    match &*expression {
        Expression::Number(n) => Ok(Some(Data::Number(*n))),
        Expression::Boolean(b) => Ok(Some(Data::Boolean(*b))),
        Expression::Identifier(i) => Ok(Some(scope.borrow_mut().resolve(i)?)),
        Expression::Reference(r) => {
            let reference = scope.borrow_mut().borrow(r)?;
            match reference {
                Data::Reference(r) => Ok(Some(r.value())),
                _ => unreachable!(),
            }
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
                drop(result);
                result = evaluate(Rc::new(node.clone()), Rc::clone(&child_scope))?;
            }
            Ok(result)
        }
        Expression::Program(p) => {
            let mut result = None;
            for node in p.into_iter() {
                drop(result);
                result = evaluate(Rc::new(node.clone()), Rc::clone(&scope))?;
            }
            Ok(result)
        }
    }
}

fn evaluate_statement(statement: Rc<Statement>, scope: Rc<RefCell<Scope>>) -> Result<()> {
    match &*statement {
        Statement::Variable { name, value } => {
            let data = evaluate(
                Rc::new(Node::Expression(Rc::clone(value))),
                Rc::clone(&scope),
            )?
            .ok_or(MovaError::Runtime(
                "Expected expression, but received statement as value".into(),
            ))?;
            scope.borrow_mut().declare(&name, data);
        }
        Statement::Function {
            name,
            parameters,
            body,
        } => {
            let function = Data::Function {
                parameters: Rc::clone(parameters),
                body: Rc::clone(body),
                definition_scope: Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope))))),
            };
            scope.borrow_mut().declare(&name, function);
        }
    }
    Ok(())
}

pub fn evaluate(node: Rc<Node>, scope: Rc<RefCell<Scope>>) -> Result<Option<Data>> {
    match &*node {
        Node::Expression(e) => evaluate_expression(Rc::clone(e), scope),
        Node::Statement(s) => {
            evaluate_statement(Rc::clone(s), scope)?;
            Ok(None)
        }
    }
}
