use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{MovaError, Result},
    interpreter::data::{BorrowableData, Data, Reference, Slot},
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

    pub fn declare(&mut self, name: &str, data: Data) {
        let slot = Rc::new(RefCell::new(BorrowableData {
            value: data,
            borrow_count: 0,
            is_mutably_borrowed: false,
        }));
        self.locals.insert(name.into(), slot);
    }

    fn find_slot(&self, name: &str) -> Result<Slot> {
        if let Some(slot) = self.locals.get(name) {
            return Ok(Rc::clone(slot));
        }

        match &self.parent {
            Some(p) => p.borrow_mut().find_slot(name),
            None => Err(MovaError::Runtime(format!("Unable to resolve {name}"))),
        }
    }

    pub fn resolve(&mut self, name: &str) -> Result<Data> {
        let slot = self.find_slot(name)?;
        let mut data = slot.borrow_mut();

        if data.is_mutably_borrowed {
            return Err(MovaError::Runtime(format!(
                "Unable to use '{name}' because it is mutably borrowed"
            )));
        }

        match data.value {
            Data::Number(_) | Data::Boolean(_) => Ok(data.value.clone()),
            Data::Moved => {
                return Err(MovaError::Runtime(format!(
                    "Unable to use '{name}' because it is moved"
                )));
            }
            _ => {
                if data.borrow_count > 0 {
                    return Err(MovaError::Runtime(format!(
                        "Unable to move {name}' because it is borrowed"
                    )));
                }
                Ok(std::mem::replace(&mut data.value, Data::Moved))
            }
        }
    }

    pub fn borrow(&mut self, name: &str) -> Result<Data> {
        let slot = self.find_slot(name)?;
        let mut data = slot.borrow_mut();

        if let Data::Moved = data.value {
            return Err(MovaError::Runtime(format!(
                "Unable to borrow '{name}' because it is moved"
            )));
        }
        if data.is_mutably_borrowed {
            return Err(MovaError::Runtime(format!(
                "Unable to borrow '{name}' because it is mutably borrowed"
            )));
        }

        data.borrow_count += 1;

        Ok(Data::Reference(Rc::new(Reference {
            source: Rc::clone(&slot),
        })))
    }
}
