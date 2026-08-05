use std::rc::Rc;

use crate::{
    error::MovaError,
    lexer::Token,
    parser::{expression::Expression, statement::*},
};

#[derive(Clone, Debug)]
pub enum Node {
    Expression(Rc<Expression>),
    Statement(Rc<Statement>),
}

pub fn parse(mut tokens: Vec<Token>) -> (Node, Vec<MovaError>) {
    let mut body = Vec::new();
    let mut errors = Vec::new();

    tokens.reverse();
    while tokens.len() != 0 {
        if let Some(Token { kind: crate::lexer::TokenKind::EndOfFile, .. }) = tokens.last() {
            tokens.pop();
            break;
        }
        match parse_statement(&mut tokens) {
            Ok(stmt) => body.push(stmt),
            Err(e) => {
                errors.push(e);
                while let Some(t) = tokens.last() {
                    if let crate::lexer::TokenKind::SpecialCharacter(';') = t.kind {
                        tokens.pop();
                        break;
                    }
                    let skipped = tokens.pop().unwrap();
                    errors.push(MovaError::Parser {
                        error: crate::error::ParserError::UnexpectedToken(format!("{:?}", skipped.kind)),
                        position: skipped.position
                    });
                }
            }
        }
    }

    (
        Node::Expression(Rc::new(Expression::Program(body.into()))),
        errors,
    )
}
