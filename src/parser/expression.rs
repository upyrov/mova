use std::rc::Rc;

use crate::{
    error::{MovaError, Result},
    lexer::Token,
    parser::{node::Node, statement::parse_statement},
};

#[derive(Clone, Debug)]
pub enum Expression {
    Number(i32),
    Boolean(bool),
    Identifier(Rc<String>),
    Reference {
        data: Rc<Expression>,
        is_mutable: bool,
    },
    BinaryExpression {
        operator: Rc<String>,
        left: Rc<Expression>,
        right: Rc<Expression>,
    },
    Call {
        name: Rc<String>,
        arguments: Rc<[Expression]>,
    },
    Block(Rc<[Node]>),
    Program(Rc<[Node]>),
}

fn get_infix_binding_power(operator: &str) -> Option<(u8, u8)> {
    match operator {
        "+" | "-" => Some((3, 4)),
        "*" | "/" => Some((5, 6)),
        _ => None,
    }
}

fn get_postfix_binding_power(operator: &str) -> Option<(u8, ())> {
    match operator {
        "(" => Some((2, ())),
        _ => None,
    }
}

fn parse_call(tokens: &mut Vec<Token>, left: Expression) -> Result<Expression> {
    tokens.pop();
    let mut parameters = Vec::new();

    loop {
        match tokens.last() {
            Some(Token::Operator(o)) if o == ")" => {
                tokens.pop();
                break;
            }
            Some(Token::SpecialCharacter(')')) => {
                tokens.pop();
                break;
            }
            Some(_) => {
                let argument = parse_expression(tokens)?;
                parameters.push(argument);

                match tokens.last() {
                    Some(Token::SpecialCharacter(',')) => {
                        tokens.pop();
                    }
                    Some(Token::Operator(o)) if o == ")" => {}
                    Some(Token::SpecialCharacter(')')) => {}
                    None => {
                        return Err(MovaError::Parser(
                            "Expected argument list to be closed".into(),
                        ));
                    }
                    _ => {
                        return Err(MovaError::Parser(
                            "Expected comma or argument list to be closed".into(),
                        ));
                    }
                }
            }
            None => {
                return Err(MovaError::Parser(
                    "Expected argument list to be closed".into(),
                ));
            }
        }
    }

    match left {
        Expression::Identifier(i) => Ok(Expression::Call {
            name: i,
            arguments: parameters.into(),
        }),
        e => Err(MovaError::Parser(format!(
            "Expected identifier to be called but found {e:?}"
        ))),
    }
}

fn parse_binary_expression(tokens: &mut Vec<Token>, binding_power: u8) -> Result<Expression> {
    let mut left = match tokens.last() {
        Some(Token::Operator(op)) if op == "&" => {
            tokens.pop();
            parse_reference(tokens)?
        }
        _ => match tokens.pop() {
            Some(Token::Identifier(i)) => Expression::Identifier(Rc::new(i)),
            Some(Token::Number(n)) => Expression::Number(
                n.parse()
                    .map_err(|_| MovaError::Parser(format!("Invalid number: {n}")))?,
            ),
            Some(Token::Boolean(b)) => Expression::Boolean(b),
            Some(t) => {
                return Err(MovaError::Parser(format!("Unexpected token found: {t:?}",)));
            }
            None => {
                return Err(MovaError::Parser("Unexpected end of input".into()));
            }
        },
    };

    while let Some(t) = tokens.last().cloned() {
        match t {
            Token::Operator(o) => {
                if let Some((lbp, ())) = get_postfix_binding_power(&o) {
                    if lbp < binding_power {
                        break;
                    }
                    if o == "(" {
                        left = parse_call(tokens, left)?;
                    }
                    continue;
                }

                if let Some((lbp, rbp)) = get_infix_binding_power(&o) {
                    if lbp < binding_power {
                        break;
                    }

                    tokens.pop();
                    let right = Rc::new(parse_binary_expression(tokens, rbp)?);
                    left = Expression::BinaryExpression {
                        left: Rc::new(left),
                        right,
                        operator: Rc::new(o),
                    };
                    continue;
                }

                break;
            }
            Token::SpecialCharacter('(') => {
                if let Some((lbp, ())) = get_postfix_binding_power("(") {
                    if lbp < binding_power {
                        break;
                    }
                    left = parse_call(tokens, left)?;
                    continue;
                }
                break;
            }
            _ => break,
        }
    }

    Ok(left)
}

fn parse_reference(tokens: &mut Vec<Token>) -> Result<Expression> {
    let is_mutable = matches!(tokens.last(), Some(Token::Keyword(k)) if k == "mut");
    if is_mutable {
        tokens.pop();
    }
    let right = parse_binary_expression(tokens, 7)?;
    Ok(Expression::Reference {
        data: Rc::new(right),
        is_mutable,
    })
}

fn parse_block(tokens: &mut Vec<Token>) -> Result<Expression> {
    match tokens.last() {
        Some(Token::SpecialCharacter('{')) => {
            tokens.pop();
            let mut body = Vec::new();

            loop {
                match tokens.last() {
                    Some(Token::SpecialCharacter('}')) => break,
                    Some(_) => body.push(parse_statement(tokens)?),
                    None => {
                        return Err(MovaError::Parser("Expected block to be closed".into()));
                    }
                }
            }

            match tokens.pop() {
                Some(Token::SpecialCharacter('}')) => Ok(Expression::Block(body.into())),
                _ => Err(MovaError::Parser("Expected block to be closed".into())),
            }
        }
        _ => parse_binary_expression(tokens, 0),
    }
}

pub fn parse_expression(tokens: &mut Vec<Token>) -> Result<Expression> {
    parse_block(tokens)
}
