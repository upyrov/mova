pub use crate::parser::node::parse;

pub mod expression;
pub mod node;
pub mod statement;

use crate::error::Position;
use crate::lexer::Token;

pub fn current_position(tokens: &[Token]) -> Position {
    tokens.last().map(|t| t.position.clone()).unwrap_or(Position { line: 1, character: 0 })
}
