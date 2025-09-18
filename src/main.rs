use std::{cell::RefCell, rc::Rc};

use mova::*;

fn main() {
    let tokens = lexer::tokenize("fn add(q){q + 5} add(4)");
    let ast = parser::parse(tokens);
    let result = interpreter::evaluate(ast, Rc::new(RefCell::new(interpreter::Scope::new(None))));
    println!("{:?}", result);
}
