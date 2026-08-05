use crate::{error::MovaError, lexer::tokenize, parser::node::parse, runner::run};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct Diagnostic {
    pub line: usize,
    pub character: usize,
    pub length: usize,
    pub message: String,
}

#[wasm_bindgen]
pub fn check_code(code: &str) -> JsValue {
    let mut diagnostics = Vec::new();
    match tokenize(code) {
        Ok(tokens) => {
            for token in &tokens {
                if let crate::lexer::TokenKind::Unknown(c) = token.kind {
                    diagnostics.push(Diagnostic {
                        line: token.position.line,
                        character: token.position.character,
                        length: 1,
                        message: format!("Unexpected character: '{}'", c),
                    });
                }
            }

            let (_, errors) = parse(tokens);
            for err in errors {
                if let MovaError::Parser {
                    error: ref e,
                    ref position,
                } = err
                {
                    let length = match &e {
                        crate::error::ParserError::UnexpectedToken(s) => {
                            if let Some(start) = s.find("(\"") {
                                if let Some(end) = s.rfind("\")") {
                                    end.saturating_sub(start + 2)
                                } else {
                                    1
                                }
                            } else if let Some(start) = s.find("('") {
                                if let Some(end) = s.rfind("')") {
                                    end.saturating_sub(start + 2)
                                } else {
                                    1
                                }
                            } else if s.starts_with("Assignment") {
                                1
                            } else {
                                1
                            }
                        }
                        _ => 1,
                    };
                    diagnostics.push(Diagnostic {
                        line: position.line,
                        character: position.character,
                        length,
                        message: e.to_string(),
                    });
                }
            }
        }
        Err(_) => {}
    }
    serde_wasm_bindgen::to_value(&diagnostics).unwrap()
}

#[wasm_bindgen]
pub fn execute_code(code: &str) -> String {
    match run(code) {
        Ok(result) => {
            if let Some(value) = result {
                match value {
                    crate::interpreter::Value::Reference(r) => match r.read() {
                        Ok(guard) => format!("{:?}", guard.value),
                        Err(e) => format!("{}", e),
                    },
                    _ => format!("{:?}", value),
                }
            } else {
                "".to_string()
            }
        }
        Err(e) => format!("{}", e),
    }
}
