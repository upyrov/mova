use std::{
    cell::{Ref, RefMut},
    rc::Rc,
};

use crate::{
    error::{MovaError, Result},
    interpreter::data::{Data, Slot, State, Value},
};

#[derive(Debug)]
pub struct Reference {
    pub slot: Slot,
    pub is_mutable: bool,
}

impl Reference {
    pub fn new(slot: Slot, is_mutable: bool) -> Result<Self> {
        let mut data = slot.borrow_mut();

        if let Value::Moved = data.value {
            return Err(MovaError::Runtime(
                "Unable to borrow value because it is moved".into(),
            ));
        }

        match data.state {
            State::Deallocated => Err(MovaError::Runtime(
                "Unable to borrow value because it is deallocated".into(),
            )),
            State::MutablyBorrowed => Err(MovaError::Runtime(
                "Unable to borrow because it is already mutably borrowed".into(),
            )),
            State::Borrowed(_) if is_mutable => Err(MovaError::Runtime(
                "Unable to borrow mutably because it is already borrowed".into(),
            )),
            State::Borrowed(count) => {
                data.state = State::Borrowed(count + 1);
                Ok(Self {
                    slot: Rc::clone(&slot),
                    is_mutable,
                })
            }
            State::Free => {
                data.state = if is_mutable {
                    State::MutablyBorrowed
                } else {
                    State::Borrowed(1)
                };
                Ok(Self {
                    slot: Rc::clone(&slot),
                    is_mutable,
                })
            }
        }
    }

    pub fn read(&self) -> Result<Ref<'_, Data>> {
        let data = self.slot.borrow();
        if let State::Deallocated = data.state {
            return Err(MovaError::Runtime(
                "Accessing a deallocated reference".into(),
            ));
        }
        Ok(data)
    }

    pub fn write(&self) -> Result<RefMut<'_, Data>> {
        let data = self.slot.borrow_mut();

        if let State::Deallocated = data.state {
            return Err(MovaError::Runtime(
                "Assigning to a deallocated reference".into(),
            ));
        }

        if self.is_mutable {
            return Ok(data);
        }

        Err(MovaError::Runtime(
            "Cannot assign to an immutable reference".into(),
        ))
    }
}

impl Drop for Reference {
    fn drop(&mut self) {
        if let Ok(mut data) = self.slot.try_borrow_mut() {
            match data.state {
                State::MutablyBorrowed if self.is_mutable => {
                    data.state = State::Free;
                }
                State::Borrowed(count) if !self.is_mutable => {
                    if count > 1 {
                        data.state = State::Borrowed(count - 1);
                    } else {
                        data.state = State::Free;
                    }
                }
                // If it's deallocated, free, or inconsistent, we do nothing.
                // This prevents panics during unwinding or redundant drops
                _ => {}
            }
        }
    }
}
