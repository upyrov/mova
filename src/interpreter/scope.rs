use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{MovaError, Result, RuntimeError},
    interpreter::data::{Data, Slot, State, Value},
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

    /// This ensures that any lingering references to these variables become invalid
    pub fn invalidate(&mut self) {
        self.locals.values().for_each(|slot| {
            let mut data = slot.borrow_mut();
            data.state = State::Deallocated;
            data.value = Value::Moved; // clears value to free resources
        });
    }

    pub fn find_slot(&self, name: &str) -> Result<Slot> {
        if let Some(slot) = self.locals.get(name) {
            return Ok(Rc::clone(slot));
        }

        match &self.parent {
            Some(p) => p.borrow().find_slot(name),
            None => Err(MovaError::Runtime(RuntimeError::UnableToResolve(name.to_string()))),
        }
    }

    pub fn resolve(&mut self, name: &str) -> Result<Value> {
        let slot = self.find_slot(name)?;
        let mut data = slot.borrow_mut();

        if let State::Deallocated = data.state {
            return Err(MovaError::Runtime(RuntimeError::UnableToUseBecauseDeallocated(name.to_string())));
        }

        if matches!(data.state, State::MutablyBorrowed) {
            return Err(MovaError::Runtime(RuntimeError::UnableToMutateBecauseMutablyBorrowed(name.to_string())));
        }

        match &data.value {
            Value::Number(_) | Value::Boolean(_) => {
                Ok(data.value.clone())
            }
            Value::Moved => {
                return Err(MovaError::Runtime(RuntimeError::UnableToUseBecauseMoved(name.to_string())));
            }
            _ => {
                if matches!(
                    data.state,
                    State::Borrowed(count) if count > 0
                ) {
                    return Err(MovaError::Runtime(RuntimeError::UnableToMutateBecauseImmutablyBorrowed(name.to_string())));
                }

                Ok(std::mem::replace(&mut data.value, Value::Moved))
            }
        }
    }
}
