use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::parser::{Expression, Node, Statement};

#[derive(Debug)]
pub enum Data {
    Number(i32),
    Tuple(Vec<Data>),
    Function(Vec<String>, Expression),
}

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

    pub fn resolve(&mut self, identifier: &str) -> Data {
        if let Some(data) = self.locals.remove(identifier) {
            return data;
        }

        match &self.parent {
            Some(p) => p.borrow_mut().resolve(identifier),
            None => panic!("Unable to resolve identifier: {}", identifier),
        }
    }
}

fn evaluate_unary_expression(operator: String, value: Data) -> Data {
    match (operator.as_str(), value) {
        (o, v) => panic!("Unexpected operator `{}` for value: {:?}", o, v),
    }
}

fn evaluate_binary_expression(operator: String, left: Data, right: Data) -> Data {
    match (operator.as_str(), left, right) {
        ("+", Data::Number(l), Data::Number(r)) => Data::Number(l + r),
        ("-", Data::Number(l), Data::Number(r)) => Data::Number(l - r),
        ("*", Data::Number(l), Data::Number(r)) => Data::Number(l * r),
        ("/", Data::Number(l), Data::Number(r)) => Data::Number(l / r),
        (o, l, r) => panic!(
            "Unexpected operator `{}` for operands: {:?} and {:?}",
            o, l, r
        ),
    }
}

fn evaluate_call(
    scope: Rc<RefCell<Scope>>,
    name: &str,
    arguments: Vec<Expression>,
) -> Option<Data> {
    match scope.borrow_mut().resolve(name) {
        Data::Function(parameters, body) => {
            let child_scope = Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope)))));

            // Map arguments to parameters
            arguments
                .into_iter()
                .zip(parameters.iter())
                .for_each(|(argument, parameter)| {
                    let data = evaluate(Node::Expression(argument), Rc::clone(&child_scope))
                        .expect("Unexpected statement found");
                    child_scope.borrow_mut().declare(&parameter, data);
                });

            evaluate(Node::Expression(body), Rc::clone(&child_scope))
        }
        _ => panic!("Unable to call non-function data"),
    }
}

fn evaluate_expression(expression: Expression, scope: Rc<RefCell<Scope>>) -> Option<Data> {
    match expression {
        Expression::Identifier(i) => Some(scope.borrow_mut().resolve(&i)),
        Expression::Number(n) => Some(Data::Number(n)),
        Expression::UnaryExpression { operator, value } => {
            let value =
                evaluate(Node::Expression(*value), scope).expect("Unexpected statement found");
            Some(evaluate_unary_expression(operator, value))
        }
        Expression::BinaryExpression {
            operator,
            left,
            right,
        } => {
            let left = evaluate(Node::Expression(*left), Rc::clone(&scope))
                .expect("Unexpected statement found");
            let right = evaluate(Node::Expression(*right), Rc::clone(&scope))
                .expect("Unexpected statement found");
            Some(evaluate_binary_expression(operator, left, right))
        }
        Expression::Call { name, arguments } => evaluate_call(scope, &name, arguments),
        Expression::Block(b) => {
            let child_scope = Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope)))));
            b.into_iter()
                .map(move |n| evaluate(n, Rc::clone(&child_scope)))
                .last()
                .unwrap_or(Some(Data::Tuple(Vec::new())))
        }
        Expression::Program(p) => p
            .into_iter()
            .map(move |n| evaluate(n, Rc::clone(&scope)))
            .last()
            .unwrap_or(Some(Data::Tuple(Vec::new()))),
    }
}

fn evaluate_statement(statement: Statement, scope: Rc<RefCell<Scope>>) {
    match statement {
        Statement::VariableDeclaration { name, value } => {
            let data = evaluate(Node::Expression(*value), Rc::clone(&scope));
            scope
                .borrow_mut()
                .declare(&name, data.expect("Unexpected statement found"));
        }
        Statement::Function {
            name,
            parameters,
            body,
        } => {
            scope
                .borrow_mut()
                .declare(&name, Data::Function(parameters, body));
        }
    }
}

pub fn evaluate(node: Node, scope: Rc<RefCell<Scope>>) -> Option<Data> {
    match node {
        Node::Expression(e) => evaluate_expression(e, scope),
        Node::Statement(s) => {
            evaluate_statement(s, scope);
            None
        }
    }
}
