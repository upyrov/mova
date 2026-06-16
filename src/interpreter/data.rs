use std::{cell::RefCell, rc::Rc};

use crate::{
    interpreter::{reference::Reference, scope::Scope},
    parser::expression::Expression,
};

#[derive(Clone, Debug)]
pub enum Value {
    Number(i32),
    Boolean(bool),
    Function {
        parameters: Rc<[String]>,
        body: Rc<Expression>,
        definition_scope: Rc<RefCell<Scope>>,
    },
    Reference(Rc<Reference>),
    Moved,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Reference(l), Value::Reference(r)) => l == r,
            (Value::Moved, Value::Moved) => true,
            (
                Value::Function {
                    parameters: lp,
                    body: lb,
                    definition_scope: ls,
                },
                Value::Function {
                    parameters: rp,
                    body: rb,
                    definition_scope: rs,
                },
            ) => {
                // For functions, we'll consider them equal only if they are the same instance
                Rc::ptr_eq(lp, rp) && Rc::ptr_eq(lb, rb) && Rc::ptr_eq(ls, rs)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Free,
    Borrowed(usize),
    MutablyBorrowed,
    Deallocated,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Data {
    pub value: Value,
    pub state: State,
    pub is_mutable: bool,
}

pub type Slot = Rc<RefCell<Data>>;
