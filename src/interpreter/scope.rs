use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{MovaError, Result},
    interpreter::data::{Data, Reference},
};

#[derive(Debug)]
pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    locals: HashMap<String, Data>,
    history: Vec<String>,
    references: Vec<Reference>,
}

impl Scope {
    pub fn new(parent: Option<Rc<RefCell<Scope>>>) -> Self {
        Self {
            parent,
            locals: HashMap::new(),
            history: Vec::new(),
            references: Vec::new(),
        }
    }

    pub fn declare(&mut self, identifier: &str, data: Data) {
        self.locals.insert(identifier.into(), data);
        self.history.push(identifier.into());
    }

    pub fn borrow(&mut self, reference: Reference) {
        self.references.push(reference);
    }

    fn is_in_history(&self, identifier: &str) -> Result<bool> {
        let contains = self.history.contains(&identifier.to_string());
        Ok(contains
            || match &self.parent {
                Some(p) => {
                    let s = p
                        .try_borrow()
                        .map_err(|_| MovaError::Runtime("Scope is already borrowed".into()))?;
                    s.is_in_history(identifier)?
                }
                None => false,
            })
    }

    fn find_local(&mut self, identifier: &str, is_parent: bool) -> Result<Data> {
        if let Some(l) = self.locals.get(identifier) {
            return Ok(match l {
                Data::Number(n) => Data::Number(*n),
                _ => {
                    if is_parent {
                        self.locals.remove(identifier).ok_or({
                            MovaError::Runtime(format!("Unable to remove identifier: {identifier}"))
                        })?
                    } else {
                        self.locals
                            .get(identifier)
                            .ok_or({
                                MovaError::Runtime(format!(
                                    "Unable to remove identifier: {identifier}"
                                ))
                            })?
                            .clone()
                    }
                }
            });
        }

        let data = match &self.parent {
            Some(p) => borrow_scope(p)?.find_local(identifier, true),
            None => {
                let is_moved = self.is_in_history(identifier)?;
                let resolution_error = if is_moved {
                    ". Value was moved to another scope"
                } else {
                    ""
                };
                Err(MovaError::Runtime(format!(
                    "Unable to resolve identifier: {identifier}{resolution_error}",
                )))
            }
        }?;
        self.declare(identifier, data.clone());
        Ok(data)
    }

    pub fn resolve(&mut self, identifier: &str) -> Result<Data> {
        self.find_local(identifier, false)
    }

    pub fn find_scope_by_identifier(
        &mut self,
        current_scope: Rc<RefCell<Scope>>,
        identifier: &str,
    ) -> Result<Rc<RefCell<Scope>>> {
        if self.locals.contains_key(identifier) {
            return Ok(current_scope);
        }

        match &self.parent {
            Some(p) => borrow_scope(p)?.find_scope_by_identifier(Rc::clone(&p), identifier),
            None => Err(MovaError::Runtime(format!(
                "Unable to find scope by identifier: {identifier}",
            ))),
        }
    }

    pub fn return_references(&mut self) -> Result<()> {
        while let Some(r) = self.references.pop() {
            let mut s = borrow_scope(&r.scope)?;
            s.declare(&r.identifier, (*r.value).clone());
        }
        Ok(())
    }
}

pub fn borrow_scope(scope: &Rc<RefCell<Scope>>) -> Result<std::cell::RefMut<Scope>> {
    scope
        .try_borrow_mut()
        .map_err(|_| MovaError::Runtime("Scope is already borrowed".into()))
}
