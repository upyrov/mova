use std::{cell::RefCell, rc::Rc};

use crate::*;

pub fn run(input: &str) -> Option<interpreter::Data> {
    let tokens = lexer::tokenize(input);
    let ast = parser::parse(tokens);
    interpreter::evaluate(ast, Rc::new(RefCell::new(interpreter::Scope::new(None))))
}
