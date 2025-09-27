use crate::{
    error::Result,
    lexer::Token,
    parser::{expression::Expression, statement::*},
};

#[derive(Clone, Debug)]
pub enum Node {
    Expression(Expression),
    Statement(Statement),
}

pub fn parse(mut tokens: Vec<Token>) -> Result<Node> {
    let mut body = Vec::new();

    tokens.reverse();
    while tokens.len() != 0 {
        body.push(parse_statement(&mut tokens)?);
    }

    Ok(Node::Expression(Expression::Program(body)))
}
