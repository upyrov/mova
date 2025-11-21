use std::{cell::RefCell, rc::Rc};

use crate::{
    error::{MovaError, Result},
    interpreter::{
        data::{Data, Reference},
        scope::{Scope, borrow_scope},
    },
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
            "Unexpected operator `{o}` for operands: {l:?} and {r:?}",
        ))),
    }
}

fn evaluate_call(
    scope: Rc<RefCell<Scope>>,
    name: &str,
    arguments: Rc<[Expression]>,
) -> Result<Option<Data>> {
    // Drop immediately after use so that recursive calls don't panic
    let function_data = {
        let mut s = borrow_scope(&scope)?;
        s.resolve(name)?
    };

    match function_data {
        Data::Function {
            parameters,
            body,
            scope,
        } => {
            let argument_count = arguments.len();
            let parameter_count = parameters.len();

            if argument_count != parameter_count {
                return Err(MovaError::Runtime(format!(
                    "Expected {parameter_count} arguments but received {argument_count}",
                )));
            }

            let evaluated_arguments = arguments
                .into_iter()
                .map(|argument| {
                    let node = Rc::new(Node::Expression(Rc::new(argument.clone())));
                    let data = evaluate(node, Rc::clone(&scope))?.ok_or(MovaError::Runtime(
                        "Expected expression, but received statement as argument".into(),
                    ));
                    match data.clone()? {
                        Data::Reference(r) => {
                            let mut s = borrow_scope(&scope)?;
                            let found_scope =
                                s.find_scope_by_identifier(Rc::clone(&scope), &r.identifier)?;
                            let referenced_value = s.resolve(&r.identifier)?;

                            let value = Rc::new(referenced_value.clone());
                            let reference = Reference {
                                identifier: Rc::clone(&r.identifier),
                                scope: found_scope,
                                value: Rc::clone(&value),
                            };
                            s.borrow(reference);

                            Ok(referenced_value)
                        }
                        _ => data,
                    }
                })
                .collect::<Vec<_>>();

            {
                let mut s = borrow_scope(&scope)?;

                // Map arguments to parameters
                evaluated_arguments
                    .into_iter()
                    .zip(parameters.iter())
                    .try_for_each(|(data, parameter)| {
                        s.declare(parameter, data?);
                        Ok(())
                    })?;
            }

            let result = evaluate(
                Rc::new(Node::Expression(Rc::clone(&body))),
                Rc::clone(&scope),
            );
            borrow_scope(&scope)?.return_references()?;
            result
        }
        _ => Err(MovaError::Runtime(format!(
            "Identifier {name} is not callable",
        ))),
    }
}

fn evaluate_expression(
    expression: Rc<Expression>,
    scope: Rc<RefCell<Scope>>,
) -> Result<Option<Data>> {
    match &*expression {
        Expression::Identifier(i) => Ok(Some(borrow_scope(&scope)?.resolve(&i)?)),
        Expression::Number(n) => Ok(Some(Data::Number(*n))),
        Expression::Boolean(b) => Ok(Some(Data::Boolean(*b))),
        Expression::Reference(r) => {
            let mut s = borrow_scope(&scope)?;
            let found_scope = s.find_scope_by_identifier(Rc::clone(&scope), &r)?;
            let referenced_value = s.resolve(&r)?;

            let value = Rc::new(referenced_value.clone());
            let reference = Reference {
                identifier: Rc::clone(&r),
                scope: found_scope,
                value: Rc::clone(&value),
            };
            s.borrow(reference);

            Ok(Some(referenced_value))
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
            let mut results: Vec<_> = b
                .into_iter()
                .map(|n| evaluate(Rc::new(n.clone()), Rc::clone(&child_scope)))
                .collect();
            borrow_scope(&child_scope)?.return_references()?;
            Ok(results.pop().transpose()?.flatten())
        }
        Expression::Program(p) => {
            let mut results: Vec<_> = p
                .into_iter()
                .map(|n| evaluate(Rc::new(n.clone()), Rc::clone(&scope)))
                .collect();
            Ok(results.pop().transpose()?.flatten())
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
                "Expected expression, but received statement".into(),
            ))?;
            borrow_scope(&scope)?.declare(&name, data);
        }
        Statement::Function {
            name,
            parameters,
            body,
        } => borrow_scope(&scope)?.declare(
            &name,
            Data::Function {
                parameters: Rc::clone(parameters),
                body: Rc::clone(body),
                scope: Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope))))),
            },
        ),
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
