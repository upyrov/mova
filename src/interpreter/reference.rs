use std::cell::{Ref, RefMut};

use crate::{
    error::{MovaError, Result},
    interpreter::data::{Data, Slot, State},
};

#[derive(Debug)]
pub struct Reference {
    pub slot: Slot,
    pub is_mutable: bool,
}

impl Reference {
    pub fn read(&self) -> Ref<'_, Data> {
        self.slot.borrow()
    }

    pub fn write(&self) -> Result<RefMut<'_, Data>> {
        if self.is_mutable {
            return Ok(self.slot.borrow_mut());
        }

        Err(MovaError::Runtime(
            "Cannot assign to an immutable reference".into(),
        ))
    }
}

impl Drop for Reference {
    fn drop(&mut self) {
        if let Ok(mut data) = self.slot.try_borrow_mut() {
            if self.is_mutable {
                data.state = State::Free;
            } else if let State::Borrowed(count) = data.state {
                data.state = State::Borrowed(count - 1);
            }
        }
    }
}
