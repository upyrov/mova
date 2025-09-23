use crate::{
    lexer::Token,
    parser::{expression::Expression, statement::*},
};

#[derive(Debug)]
pub enum Node {
    Expression(Expression),
    Statement(Statement),
}

pub fn parse(mut tokens: Vec<Token>) -> Node {
    let mut body = Vec::new();

    tokens.reverse();
    while let Some(node) = parse_statement(&mut tokens) {
        body.push(node);
    }

    Node::Expression(Expression::Program(body))
}
