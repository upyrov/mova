use std::{
    cell::{Ref, RefMut},
    rc::Rc,
};

use crate::{
    error::{MovaError, Result, RuntimeError},
    interpreter::data::{Data, Slot, State, Value},
};

#[derive(Debug)]
pub struct Reference {
    pub slot: Slot,
    pub is_mutable: bool,
}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.slot, &other.slot) && self.is_mutable == other.is_mutable
    }
}

impl Reference {
    pub fn new(slot: Slot, is_mutable: bool) -> Result<Self> {
        let mut data = slot.borrow_mut();

        if let Value::Moved = data.value {
            return Err(MovaError::Runtime(
                RuntimeError::UnableToBorrowBecauseMoved,
            ));
        }

        match data.state {
            State::Deallocated => Err(MovaError::Runtime(
                RuntimeError::UnableToBorrowBecauseDeallocated,
            )),
            State::MutablyBorrowed => Err(MovaError::Runtime(
                RuntimeError::UnableToBorrowBecauseMutablyBorrowed,
            )),
            State::Borrowed(_) if is_mutable => Err(MovaError::Runtime(
                RuntimeError::UnableToBorrowMutablyBecauseBorrowed,
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
                RuntimeError::AccessingDeallocatedReference,
            ));
        }
        Ok(data)
    }

    pub fn write(&self) -> Result<RefMut<'_, Data>> {
        let data = self.slot.borrow_mut();

        if let State::Deallocated = data.state {
            return Err(MovaError::Runtime(
                RuntimeError::AssigningToDeallocatedReference,
            ));
        }

        if self.is_mutable {
            return Ok(data);
        }

        Err(MovaError::Runtime(
            RuntimeError::CannotAssignToImmutableReference,
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
