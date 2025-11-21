use std::{cell::RefCell, rc::Rc};

use crate::{interpreter::scope::Scope, parser::expression::Expression};

#[derive(Clone, Debug)]
pub struct Reference {
    pub identifier: Rc<String>,
    pub scope: Rc<RefCell<Scope>>,
    pub value: Rc<Data>,
}

// We use reference counter to make cloning cheap
#[derive(Clone, Debug)]
pub enum Data {
    Number(i32),
    Boolean(bool),
    Function {
        parameters: Rc<[String]>,
        body: Rc<Expression>,
        scope: Rc<RefCell<Scope>>,
    },
    Reference(Rc<Reference>),
}
