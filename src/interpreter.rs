use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{MovaError, Result},
    parser::{expression::Expression, node::Node, statement::Statement},
};

#[derive(Clone, Debug)]
pub enum Data {
    Number(i32),
    Tuple(Vec<Data>),
    Function(Vec<String>, Expression),
}

#[derive(Debug)]
pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    locals: HashMap<String, Data>,
}

impl Scope {
    pub fn new(parent: Option<Rc<RefCell<Scope>>>) -> Self {
        Self {
            parent,
            locals: HashMap::new(),
        }
    }

    pub fn declare(&mut self, identifier: &str, data: Data) {
        self.locals.insert(identifier.into(), data);
    }

    pub fn resolve(&mut self, identifier: &str) -> Result<Data> {
        if let Some(l) = self.locals.get(identifier) {
            return Ok(match l {
                Data::Number(n) => Data::Number(*n),
                _ => self.locals.remove(identifier).ok_or({
                    MovaError::Runtime(format!("Unable to remove identifier: {}", identifier))
                })?,
            });
        }

        match &self.parent {
            Some(p) => p
                .try_borrow_mut()
                .map_err(|_| MovaError::Runtime("Unable to borrow data".into()))?
                .resolve(identifier),
            None => Err(MovaError::Runtime(format!(
                "Unable to resolve identifier: {}",
                identifier
            ))),
        }
    }
}

fn evaluate_unary_expression(operator: String, value: Data) -> Result<Data> {
    match (operator.as_str(), value) {
        (o, v) => Err(MovaError::Runtime(format!(
            "Unexpected operator `{}` for value: {:?}",
            o, v
        ))),
    }
}

fn evaluate_binary_expression(operator: String, left: Data, right: Data) -> Result<Data> {
    match (operator.as_str(), left, right) {
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
            "Unexpected operator `{o}` for operands: {:?} and {:?}",
            l, r
        ))),
    }
}

fn evaluate_call(
    scope: Rc<RefCell<Scope>>,
    name: &str,
    arguments: Vec<Expression>,
) -> Result<Option<Data>> {
    let function_data = scope
        .try_borrow_mut()
        .map_err(|_| MovaError::Runtime("Unable to borrow data".into()))?
        .resolve(&name)?;
    match function_data {
        Data::Function(parameters, body) => {
            let argument_count = arguments.len();
            let parameter_count = parameters.len();
            if argument_count != parameter_count {
                return Err(MovaError::Runtime(format!(
                    "Expected {} arguments but received {}",
                    parameter_count, argument_count
                )));
            }

            let child_scope = Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope)))));
            let evaluated_arguments = arguments.into_iter().map(|argument| {
                evaluate(Node::Expression(argument), Rc::clone(&scope))?.ok_or(MovaError::Runtime(
                    "Expected expression, but received statement".into(),
                ))
            });

            // Map arguments to parameters
            evaluated_arguments
                .zip(parameters.iter())
                .try_for_each(|(data, parameter)| {
                    let mut child_scope = child_scope
                        .try_borrow_mut()
                        .map_err(|_| MovaError::Runtime("Unable to borrow data".into()))?;
                    child_scope.declare(parameter, data?);
                    Ok(())
                })?;

            evaluate(Node::Expression(body), Rc::clone(&child_scope))
        }
        _ => Err(MovaError::Runtime(format!(
            "Unable to call non-function data: {:?}",
            function_data
        ))),
    }
}

fn evaluate_expression(expression: Expression, scope: Rc<RefCell<Scope>>) -> Result<Option<Data>> {
    match expression {
        Expression::Identifier(i) => Ok(Some(
            scope
                .try_borrow_mut()
                .map_err(|_| MovaError::Runtime("Unable to borrow data".into()))?
                .resolve(&i)?,
        )),
        Expression::Number(n) => Ok(Some(Data::Number(n))),
        Expression::UnaryExpression { operator, value } => {
            let value = evaluate(Node::Expression(*value), scope)?.ok_or(MovaError::Runtime(
                "Expected expression, but received statement".into(),
            ))?;
            Ok(Some(evaluate_unary_expression(operator, value)?))
        }
        Expression::BinaryExpression {
            operator,
            left,
            right,
        } => {
            let left = evaluate(Node::Expression(*left), Rc::clone(&scope))?.ok_or(
                MovaError::Runtime("Expected expression, but received statement".into()),
            )?;
            let right = evaluate(Node::Expression(*right), Rc::clone(&scope))?.ok_or(
                MovaError::Runtime("Expected expression, but received statement".into()),
            )?;
            Ok(Some(evaluate_binary_expression(operator, left, right)?))
        }
        Expression::Call { name, arguments } => evaluate_call(scope, &name, arguments),
        Expression::Block(b) => {
            let child_scope = Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope)))));
            let mut results: Vec<_> = b
                .into_iter()
                .map(|n| evaluate(n, Rc::clone(&child_scope)))
                .collect();
            Ok(results.pop().transpose()?.flatten())
        }
        Expression::Program(p) => {
            let mut results: Vec<_> = p
                .into_iter()
                .map(|n| evaluate(n, Rc::clone(&scope)))
                .collect();
            Ok(results.pop().transpose()?.flatten())
        }
    }
}

fn evaluate_statement(statement: Statement, scope: Rc<RefCell<Scope>>) -> Result<()> {
    match statement {
        Statement::VariableDeclaration { name, value } => {
            let data = evaluate(Node::Expression(*value), Rc::clone(&scope))?.ok_or(
                MovaError::Runtime("Expected expression, but received statement".into()),
            )?;
            scope
                .try_borrow_mut()
                .map_err(|_| MovaError::Runtime("Unable to borrow data".into()))?
                .declare(&name, data);
        }
        Statement::Function {
            name,
            parameters,
            body,
        } => {
            scope
                .try_borrow_mut()
                .map_err(|_| MovaError::Runtime("Unable to borrow data".into()))?
                .declare(&name, Data::Function(parameters, body));
        }
    }
    Ok(())
}

pub fn evaluate(node: Node, scope: Rc<RefCell<Scope>>) -> Result<Option<Data>> {
    match node {
        Node::Expression(e) => evaluate_expression(e, scope),
        Node::Statement(s) => {
            evaluate_statement(s, scope)?;
            Ok(None)
        }
    }
}
