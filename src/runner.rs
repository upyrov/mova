use std::{cell::RefCell, rc::Rc};

use crate::{error::Result, interpreter::*, lexer::tokenize, parser::parse};

pub fn run(input: &str) -> Result<Option<Value>> {
    let tokens = tokenize(input)?;
    let (program, errors) = parse(tokens);
    if let Some(err) = errors.into_iter().next() {
        return Err(err);
    }
    evaluate(Rc::new(program), Rc::new(RefCell::new(Scope::new(None))))
}
