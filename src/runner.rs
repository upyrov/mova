use std::{cell::RefCell, rc::Rc};

use crate::{interpreter::*, lexer::tokenize, parser::parse};

pub fn run(input: &str) -> Option<Data> {
    let tokens = tokenize(input);
    let program = parse(tokens);
    evaluate(program, Rc::new(RefCell::new(Scope::new(None))))
}
