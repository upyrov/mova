use std::rc::Rc;

use crate::{
    error::{MovaError, ParserError, Result},
    lexer::{Token, TokenKind},
    parser::{node::Node, statement::parse_statement, current_position},
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
    Dereference(Rc<Expression>),
    Block(Rc<[Node]>),
    If {
        condition: Rc<Expression>,
        consequence: Rc<Expression>,
        alternative: Option<Rc<Expression>>,
    },
    While {
        condition: Rc<Expression>,
        body: Rc<Expression>,
    },
    Program(Rc<[Node]>),
}

fn get_infix_binding_power(operator: &str) -> Option<(u8, u8)> {
    match operator {
        "==" | "!=" | "<" | ">" | "<=" | ">=" => Some((1, 2)),
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
    let pos_start = current_position(tokens);
    tokens.pop();
    let mut parameters = Vec::new();

    loop {
        let pos_loop = current_position(tokens);
        match tokens.last().map(|t| &t.kind) {
            Some(TokenKind::Operator(o)) if o == ")" => {
                tokens.pop();
                break;
            }
            Some(TokenKind::SpecialCharacter(')')) => {
                tokens.pop();
                break;
            }
            Some(_) => {
                let argument = parse_expression(tokens)?;
                parameters.push(argument);

                let pos_comma = current_position(tokens);
                match tokens.last().map(|t| &t.kind) {
                    Some(TokenKind::SpecialCharacter(',')) => {
                        tokens.pop();
                    }
                    Some(TokenKind::Operator(o)) if o == ")" => {}
                    Some(TokenKind::SpecialCharacter(')')) => {}
                    None => {
                        return Err(MovaError::Parser {
                            error: ParserError::ExpectedArgumentListToBeClosed,
                            position: pos_comma,
                        });
                    }
                    _ => {
                        return Err(MovaError::Parser {
                            error: ParserError::ExpectedCommaOrArgumentListToBeClosed,
                            position: pos_comma,
                        });
                    }
                }
            }
            None => {
                return Err(MovaError::Parser {
                    error: ParserError::ExpectedArgumentListToBeClosed,
                    position: pos_loop,
                });
            }
        }
    }

    match left {
        Expression::Identifier(i) => Ok(Expression::Call {
            name: i,
            arguments: parameters.into(),
        }),
        e => Err(MovaError::Parser {
            error: ParserError::ExpectedIdentifierToBeCalled(format!("{e:?}")),
            position: pos_start,
        }),
    }
}

fn parse_binary_expression(tokens: &mut Vec<Token>, binding_power: u8) -> Result<Expression> {
    let mut left = match tokens.last().map(|t| &t.kind) {
        Some(TokenKind::Operator(op)) if op == "&" => {
            tokens.pop();
            parse_reference(tokens)?
        }
        Some(TokenKind::Operator(op)) if op == "*" => {
            tokens.pop();
            Expression::Dereference(Rc::new(parse_binary_expression(tokens, 7)?))
        }
        Some(TokenKind::Operator(op)) if op == "(" => {
            tokens.pop();
            let expr = parse_expression(tokens)?;
            let pos_close = current_position(tokens);
            match tokens.pop().map(|t| t.kind) {
                Some(TokenKind::Operator(op)) if op == ")" => Ok(expr),
                Some(t) => Err(MovaError::Parser { error: ParserError::ExpectedClosingParenthesis(format!("{t:?}")), position: pos_close }),
                None => Err(MovaError::Parser { error: ParserError::ExpectedClosingParenthesisButFoundEndOfInput, position: pos_close }),
            }?
        }
        _ => {
            let pos_pop = current_position(tokens);
            match tokens.pop().map(|t| t.kind) {
                Some(TokenKind::Identifier(i)) => Expression::Identifier(Rc::new(i)),
                Some(TokenKind::Number(n)) => Expression::Number(
                    n.parse()
                        .map_err(|_| MovaError::Parser { error: ParserError::InvalidNumber(n), position: pos_pop.clone() })?,
                ),
                Some(TokenKind::Boolean(b)) => Expression::Boolean(b),
                Some(TokenKind::Keyword(k)) if k == "if" => {
                    let condition = Rc::new(parse_expression(tokens)?);
                    let consequence = Rc::new(parse_block(tokens)?);
                    let alternative = match tokens.last().map(|t| &t.kind) {
                        Some(TokenKind::Keyword(k)) if k == "else" => {
                            tokens.pop();
                            if let Some(TokenKind::Keyword(next_k)) = tokens.last().map(|t| &t.kind) {
                                if next_k == "if" {
                                    Some(Rc::new(parse_expression(tokens)?))
                                } else {
                                    Some(Rc::new(parse_block(tokens)?))
                                }
                            } else {
                                Some(Rc::new(parse_block(tokens)?))
                            }
                        }
                        _ => None,
                    };
                    Expression::If {
                        condition,
                        consequence,
                        alternative,
                    }
                }
                Some(TokenKind::Keyword(k)) if k == "while" => {
                    let condition = Rc::new(parse_expression(tokens)?);
                    let body = Rc::new(parse_block(tokens)?);
                    Expression::While { condition, body }
                }
                Some(TokenKind::EndOfFile) => {
                    return Err(MovaError::Parser { error: ParserError::UnexpectedEndOfInput, position: pos_pop });
                }
                Some(t) => {
                    return Err(MovaError::Parser { error: ParserError::UnexpectedToken(format!("{t:?}")), position: pos_pop });
                }
                None => {
                    return Err(MovaError::Parser { error: ParserError::UnexpectedEndOfInput, position: pos_pop });
                }
            }
        },
    };

    while let Some(t) = tokens.last().map(|t| t.kind.clone()) {
        match t {
            TokenKind::Operator(o) => {
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
            TokenKind::SpecialCharacter('(') => {
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
    let is_mutable = matches!(tokens.last().map(|t| &t.kind), Some(TokenKind::Keyword(k)) if k == "mut");
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
    match tokens.last().map(|t| &t.kind) {
        Some(TokenKind::SpecialCharacter('{')) => {
            tokens.pop();
            let mut body = Vec::new();

            loop {
                let pos_loop = current_position(tokens);
                match tokens.last().map(|t| &t.kind) {
                    Some(TokenKind::SpecialCharacter('}')) => break,
                    Some(_) => body.push(parse_statement(tokens)?),
                    None => {
                        return Err(MovaError::Parser { error: ParserError::ExpectedBlockToBeClosed, position: pos_loop });
                    }
                }
            }

            let pos_close = current_position(tokens);
            match tokens.pop().map(|t| t.kind) {
                Some(TokenKind::SpecialCharacter('}')) => Ok(Expression::Block(body.into())),
                _ => Err(MovaError::Parser { error: ParserError::ExpectedBlockToBeClosed, position: pos_close }),
            }
        }
        _ => parse_binary_expression(tokens, 0),
    }
}

pub fn parse_expression(tokens: &mut Vec<Token>) -> Result<Expression> {
    parse_block(tokens)
}
