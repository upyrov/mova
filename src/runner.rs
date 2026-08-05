use std::{cell::RefCell, rc::Rc};

use crate::{error::Result, interpreter::*, lexer::tokenize, parser::parse};

thread_local! {
    pub static OUTPUT_BUFFER: RefCell<String> = RefCell::new(String::new());
}

pub fn run(input: &str) -> Result<Option<Value>> {
    let tokens = tokenize(input)?;
    let (program, errors) = parse(tokens);
    if let Some(err) = errors.into_iter().next() {
        return Err(err);
    }

    OUTPUT_BUFFER.with(|b| b.borrow_mut().clear());

    let global_scope = Rc::new(RefCell::new(Scope::new(None)));

    // Seed built-in functions
    global_scope.borrow_mut().declare(
        &Rc::new("print".to_string()),
        Value::BuiltInFunction { name: "print" },
        false,
    );
    global_scope.borrow_mut().declare(
        &Rc::new("println".to_string()),
        Value::BuiltInFunction { name: "println" },
        false,
    );

    evaluate(Rc::new(program), global_scope)
}
