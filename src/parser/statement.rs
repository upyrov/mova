use crate::{
    error::{MovaError, Result},
    lexer::Token,
    parser::{expression::*, node::Node},
};

#[derive(Clone, Debug)]
pub enum Statement {
    VariableDeclaration {
        name: String,
        value: Box<Expression>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Expression,
    },
}

fn parse_variable_declaration(tokens: &mut Vec<Token>) -> Result<Node> {
    tokens.pop();

    let name = match tokens.pop() {
        Some(Token::Identifier(i)) => i,
        Some(t) => {
            return Err(MovaError::Parser(format!(
                "Expected identifier but got: {:?}",
                t
            )));
        }
        None => {
            return Err(MovaError::Parser(
                "Expected identifier after `let` keyword".into(),
            ));
        }
    };

    match tokens.pop() {
        Some(Token::Assignment) => {
            let value = Box::new(parse_expression(tokens)?);
            Ok(Node::Statement(Statement::VariableDeclaration {
                name,
                value,
            }))
        }
        Some(t) => Err(MovaError::Parser(format!(
            "Unexpected token found: {:?}",
            t
        ))),
        None => Err(MovaError::Parser(
            "Expected assignment after identifier".into(),
        )),
    }
}

fn parse_function(tokens: &mut Vec<Token>) -> Result<Node> {
    tokens.pop();

    let name = match tokens.pop() {
        Some(Token::Identifier(i)) => i,
        _ => {
            return Err(MovaError::Parser(
                "Expected function name after `fn` keyword".into(),
            ));
        }
    };
    match tokens.pop() {
        Some(Token::Operator(o)) if o == "(" => {}
        _ => {
            return Err(MovaError::Parser(
                "Expected parameter list after function name".into(),
            ));
        }
    }

    let mut parameters = Vec::new();
    loop {
        match tokens.last() {
            Some(token) => match token {
                Token::Operator(o) if o == ")" => break,
                _ => {
                    if let Some(t) = tokens.pop() {
                        if let Token::Identifier(i) = t {
                            parameters.push(i);
                        }
                    }
                }
            },
            None => {
                return Err(MovaError::Parser(
                    "Expected parameter list to be closed".into(),
                ));
            }
        }
    }

    match tokens.pop() {
        Some(Token::Operator(o)) if o == ")" => {}
        _ => {
            return Err(MovaError::Parser(
                "Expected parameter list to be closed".into(),
            ));
        }
    }

    match tokens.pop() {
        Some(Token::Assignment) => {}
        _ => {
            return Err(MovaError::Parser(
                "Expected assignment before function body".into(),
            ));
        }
    }

    let body = parse_expression(tokens)?;
    Ok(Node::Statement(Statement::Function {
        name,
        parameters,
        body,
    }))
}

pub fn parse_statement(tokens: &mut Vec<Token>) -> Result<Node> {
    match tokens.last() {
        Some(Token::Keyword(k)) => match k.as_str() {
            "let" => parse_variable_declaration(tokens),
            "fn" => parse_function(tokens),
            k => Err(MovaError::Parser(format!("Unexpected keyword found: {k}",))),
        },
        Some(_) => parse_expression(tokens).map(|t| Node::Expression(t)),
        None => Err(MovaError::Parser("Unexpected end of input".into())),
    }
}
