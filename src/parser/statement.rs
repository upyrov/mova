use std::rc::Rc;

use crate::{
    error::{MovaError, Result},
    lexer::Token,
    parser::{expression::*, node::Node},
};

#[derive(Clone, Debug)]
pub enum Statement {
    Variable {
        name: Rc<String>,
        value: Rc<Expression>,
        is_mutable: bool,
    },
    Assignment {
        name: Rc<String>,
        value: Rc<Expression>,
    },
    DereferenceAssignment {
        target: Rc<Expression>,
        value: Rc<Expression>,
    },
    Function {
        name: Rc<String>,
        parameters: Rc<[String]>,
        body: Rc<Expression>,
    },
}

fn parse_variable(tokens: &mut Vec<Token>) -> Result<Node> {
    tokens.pop();

    let is_mutable = matches!(tokens.last(), Some(Token::Keyword(k)) if k == "mut");
    if is_mutable {
        tokens.pop();
    }

    let name = Rc::new(match tokens.pop() {
        Some(Token::Identifier(i)) => i,
        Some(t) => {
            return Err(MovaError::Parser(format!(
                "Expected identifier but got: {t:?}"
            )));
        }
        None => {
            return Err(MovaError::Parser(
                "Expected identifier after `let` keyword".into(),
            ));
        }
    });

    match tokens.pop() {
        Some(Token::Assignment) => {
            let value = Rc::new(parse_expression(tokens)?);
            Ok(Node::Statement(Rc::new(Statement::Variable {
                name,
                value,
                is_mutable,
            })))
        }
        Some(t) => Err(MovaError::Parser(format!("Unexpected token found: {t:?}"))),
        None => Err(MovaError::Parser(
            "Expected assignment after identifier".into(),
        )),
    }
}

fn parse_function(tokens: &mut Vec<Token>) -> Result<Node> {
    tokens.pop();

    let name = Rc::new(match tokens.pop() {
        Some(Token::Identifier(i)) => i,
        _ => {
            return Err(MovaError::Parser(
                "Expected function name after `fn` keyword".into(),
            ));
        }
    });
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
        _ => Err(MovaError::Parser(
            "Expected assignment before function body".into(),
        ))?,
    }

    Ok(Node::Statement(Rc::new(Statement::Function {
        name,
        parameters: parameters.into(),
        body: Rc::new(parse_expression(tokens)?),
    })))
}

pub fn parse_statement(tokens: &mut Vec<Token>) -> Result<Node> {
    while let Some(Token::SpecialCharacter(';')) = tokens.last() {
        tokens.pop();
    }

    let node = match tokens.last() {
        Some(Token::Keyword(k)) => match k.as_str() {
            "let" => parse_variable(tokens),
            "fn" => parse_function(tokens),
            k => Err(MovaError::Parser(format!("Unexpected keyword found: {k}",))),
        },
        Some(_) => {
            let result = parse_expression(tokens);
            match result? {
                Expression::Identifier(name) => match tokens.last() {
                    Some(Token::Assignment) => {
                        tokens.pop();
                        let value = parse_expression(tokens)?;
                        Ok(Node::Statement(Rc::new(Statement::Assignment {
                            name,
                            value: Rc::new(value),
                        })))
                    }
                    _ => Ok(Node::Expression(Rc::new(Expression::Identifier(name)))),
                },
                Expression::Dereference(target) => match tokens.last() {
                    Some(Token::Assignment) => {
                        tokens.pop();
                        let value = parse_expression(tokens)?;
                        Ok(Node::Statement(Rc::new(Statement::DereferenceAssignment {
                            target,
                            value: Rc::new(value),
                        })))
                    }
                    _ => Ok(Node::Expression(Rc::new(Expression::Dereference(target)))),
                },
                e => Ok(Node::Expression(Rc::new(e))),
            }
        }
        None => Err(MovaError::Parser("Unexpected end of input".into())),
    }?;

    while let Some(Token::SpecialCharacter(';')) = tokens.last() {
        tokens.pop();
    }

    Ok(node)
}
