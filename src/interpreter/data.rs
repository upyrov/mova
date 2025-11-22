use std::{cell::RefCell, rc::Rc};

use crate::{interpreter::scope::Scope, parser::expression::Expression};

pub type Slot = Rc<RefCell<BorrowableData>>;

#[derive(Clone, Debug)]
pub struct BorrowableData {
    pub value: Data,
    pub borrow_count: usize,
    pub is_mutably_borrowed: bool,
}

#[derive(Clone, Debug)]
pub struct Reference {
    pub source: Slot,
}

impl Reference {
    pub fn value(&self) -> Data {
        self.source.borrow().value.clone()
    }
}

impl Drop for Reference {
    fn drop(&mut self) {
        if let Ok(mut data) = self.source.try_borrow_mut() {
            if data.borrow_count > 0 {
                data.borrow_count -= 1;
            }
        }
    }
}

// We use reference counter to make cloning cheap
#[derive(Clone, Debug)]
pub enum Data {
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
