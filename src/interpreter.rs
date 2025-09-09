use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::parser::{Expression, Node, Statement};

#[derive(Clone, Debug)]
pub enum Data {
    Number(i32),
    Tuple(Vec<Data>),
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

    pub fn resolve(&self, identifier: &str) -> Data {
        if let Some(data) = self.locals.get(identifier) {
            return data.clone();
        }

        match &self.parent {
            Some(p) => p.borrow().resolve(identifier).clone(),
            None => panic!("Unable to resolve identifier: {}", identifier),
        }
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

pub fn evaluate(node: Node, scope: Option<Rc<RefCell<Scope>>>) -> Option<Data> {
    match node {
        Node::Expression(e) => match e {
            Expression::Identifier(i) => Some(scope.unwrap().borrow().resolve(&i)),
            Expression::Number(n) => Some(Data::Number(n)),
            Expression::BinaryExpression {
                operator,
                left,
                right,
            } => {
                let left = evaluate(
                    Node::Expression(*left),
                    Some(Rc::clone(scope.as_ref().unwrap())),
                );
                let right = evaluate(
                    Node::Expression(*right),
                    Some(Rc::clone(scope.as_ref().unwrap())),
                );
                Some(evaluate_binary_expression(
                    operator,
                    left.expect("Unexpected statement found"),
                    right.expect("Unexpected statement found"),
                ))
            }
            Expression::Block(b) => {
                let child_scope = Rc::new(RefCell::new(Scope::new(
                    scope.as_ref().map(|s| Rc::clone(&s)),
                )));
                b.into_iter()
                    .map(move |n| evaluate(n, Some(Rc::clone(&child_scope))))
                    .last()
                    .unwrap_or(Some(Data::Tuple(Vec::new())))
            }
        },
        Node::Statement(s) => match s {
            Statement::VariableDeclaration { name, value } => {
                let data = evaluate(
                    Node::Expression(*value),
                    Some(Rc::clone(scope.as_ref().unwrap())),
                );
                scope
                    .unwrap()
                    .borrow_mut()
                    .declare(&name, data.expect("Unexpected statement found"));
                None
            }
        },
    }
}
