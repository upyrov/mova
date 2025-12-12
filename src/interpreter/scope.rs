use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{MovaError, Result},
    interpreter::{
        data::{Data, Slot, State, Value},
        reference::Reference,
    },
};

#[derive(Clone, Debug)]
pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    locals: HashMap<String, Slot>,
}

impl Scope {
    pub fn new(parent: Option<Rc<RefCell<Scope>>>) -> Self {
        Self {
            parent,
            locals: HashMap::new(),
        }
    }

    pub fn declare(&mut self, name: &str, value: Value, is_mutable: bool) {
        let slot = Rc::new(RefCell::new(Data {
            value,
            state: State::Free,
            is_mutable,
        }));
        self.locals.insert(name.into(), slot);
    }

    pub fn find_slot(&self, name: &str) -> Result<Slot> {
        if let Some(slot) = self.locals.get(name) {
            return Ok(Rc::clone(slot));
        }

        match &self.parent {
            Some(p) => p.borrow().find_slot(name),
            None => Err(MovaError::Runtime(format!("Unable to resolve {name}"))),
        }
    }

    pub fn resolve(&mut self, name: &str) -> Result<Value> {
        let slot = self.find_slot(name)?;
        let mut data = slot.borrow_mut();

        if matches!(data.state, State::MutablyBorrowed) {
            return Err(MovaError::Runtime(format!(
                "Unable to use '{name}' because it is mutably borrowed"
            )));
        }

        match &data.value {
            Value::Number(_) | Value::Boolean(_) => Ok(data.value.clone()),
            Value::Moved => {
                return Err(MovaError::Runtime(format!(
                    "Unable to use '{name}' because it is moved"
                )));
            }
            _ => {
                if matches!(
                    data.state,
                    State::Borrowed(count) if count > 0
                ) {
                    return Err(MovaError::Runtime(format!(
                        "Unable to move {name}' because it is borrowed"
                    )));
                }

                Ok(std::mem::replace(&mut data.value, Value::Moved))
            }
        }
    }

    pub fn borrow(&mut self, name: &str) -> Result<Value> {
        let slot = self.find_slot(name)?;
        let mut data = slot.borrow_mut();

        if let Value::Moved = data.value {
            return Err(MovaError::Runtime(format!(
                "Unable to borrow '{name}' because it is moved"
            )));
        }
        if matches!(data.state, State::MutablyBorrowed) {
            return Err(MovaError::Runtime(format!(
                "Unable to borrow '{name}' because it is mutably borrowed"
            )));
        }

        if let State::Borrowed(count) = data.state {
            data.state = State::Borrowed(count + 1);
        }

        Ok(Value::Reference(Rc::new(Reference {
            slot: Rc::clone(&slot),
            is_mutable: false,
        })))
    }

    pub fn borrow_mut(&mut self, name: &str) -> Result<Value> {
        let slot = self.find_slot(name)?;
        let mut data = slot.borrow_mut();

        if let Value::Moved = data.value {
            return Err(MovaError::Runtime(format!(
                "Unable to borrow '{name}' because it is moved"
            )));
        }

        if matches!(data.state, State::MutablyBorrowed) {
            return Err(MovaError::Runtime(format!(
                "Unable to borrow '{name}' because it is mutably borrowed"
            )));
        }
        if matches!(
            data.state,
            State::Borrowed(count) if count > 0
        ) {
            return Err(MovaError::Runtime(format!(
                "Unable to move {name}' because it is borrowed"
            )));
        }

        data.state = State::MutablyBorrowed;
        Ok(Value::Reference(Rc::new(Reference {
            slot: Rc::clone(&slot),
            is_mutable: true,
        })))
    }
}
