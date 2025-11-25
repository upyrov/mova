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

#[derive(Debug, Clone, Copy)]
pub enum State {
    Free,
    Borrowed(usize),
    MutablyBorrowed,
}

#[derive(Clone, Debug)]
pub struct Data {
    pub value: Value,
    pub state: State,
    pub is_mutable: bool,
}

pub type Slot = Rc<RefCell<Data>>;
