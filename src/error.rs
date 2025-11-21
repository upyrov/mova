use std::fmt;

#[derive(Debug)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug)]
pub enum MovaError {
    Lexer { message: String, position: Position },
    Parser(String),
    Runtime(String),
}

impl fmt::Display for MovaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MovaError::Lexer { message, position } => {
                write!(
                    f,
                    "Lexer error at {}:{}: {message}",
                    position.line, position.character
                )
            }
            MovaError::Parser(message) => {
                write!(f, "Parser error: {message}")
            }
            MovaError::Runtime(message) => {
                write!(f, "Runtime error: {message}")
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, MovaError>;
