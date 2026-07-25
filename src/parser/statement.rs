use std::rc::Rc;

use crate::{
    error::{MovaError, ParserError, Result},
    lexer::{Token, TokenKind},
    parser::{expression::*, node::Node, current_position},
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

    let is_mutable = matches!(tokens.last(), Some(Token { kind: TokenKind::Keyword(k), .. }) if k == "mut");
    if is_mutable {
        tokens.pop();
    }

    let pos_before_name = current_position(tokens);
    let name = Rc::new(match tokens.pop().map(|t| t.kind) {
        Some(TokenKind::Identifier(i)) => i,
        Some(t) => {
            return Err(MovaError::Parser {
                error: ParserError::ExpectedIdentifierButGot(format!("{t:?}")),
                position: pos_before_name,
            });
        }
        None => {
            return Err(MovaError::Parser {
                error: ParserError::ExpectedIdentifierAfterLet,
                position: pos_before_name,
            });
        }
    });

    let pos_before_assignment = current_position(tokens);
    match tokens.pop().map(|t| t.kind) {
        Some(TokenKind::Assignment) => {
            let value = Rc::new(parse_expression(tokens)?);
            Ok(Node::Statement(Rc::new(Statement::Variable {
                name,
                value,
                is_mutable,
            })))
        }
        Some(t) => Err(MovaError::Parser {
            error: ParserError::UnexpectedToken(format!("{t:?}")),
            position: pos_before_assignment,
        }),
        None => Err(MovaError::Parser {
            error: ParserError::ExpectedAssignmentAfterIdentifier,
            position: pos_before_assignment,
        }),
    }
}

fn parse_function(tokens: &mut Vec<Token>) -> Result<Node> {
    tokens.pop();

    let pos_before_name = current_position(tokens);
    let name = Rc::new(match tokens.pop().map(|t| t.kind) {
        Some(TokenKind::Identifier(i)) => i,
        _ => {
            return Err(MovaError::Parser {
                error: ParserError::ExpectedFunctionName,
                position: pos_before_name,
            });
        }
    });

    let pos_before_param = current_position(tokens);
    match tokens.pop().map(|t| t.kind) {
        Some(TokenKind::Operator(o)) if o == "(" => {}
        _ => {
            return Err(MovaError::Parser {
                error: ParserError::ExpectedParameterList,
                position: pos_before_param,
            });
        }
    }

    let mut parameters = Vec::new();
    loop {
        let pos_loop = current_position(tokens);
        match tokens.last().map(|t| &t.kind) {
            Some(TokenKind::Operator(o)) if o == ")" => break,
            Some(_) => {
                if let Some(t) = tokens.pop() {
                    if let TokenKind::Identifier(i) = t.kind {
                        parameters.push(i);
                    }
                }
            },
            None => {
                return Err(MovaError::Parser {
                    error: ParserError::ExpectedParameterListToBeClosed,
                    position: pos_loop,
                });
            }
        }
    }

    let pos_after_param = current_position(tokens);
    match tokens.pop().map(|t| t.kind) {
        Some(TokenKind::Operator(o)) if o == ")" => {}
        _ => {
            return Err(MovaError::Parser {
                error: ParserError::ExpectedParameterListToBeClosed,
                position: pos_after_param,
            });
        }
    }

    let pos_before_assignment = current_position(tokens);
    match tokens.pop().map(|t| t.kind) {
        Some(TokenKind::Assignment) => {}
        _ => return Err(MovaError::Parser {
            error: ParserError::ExpectedAssignmentBeforeFunctionBody,
            position: pos_before_assignment,
        }),
    }

    Ok(Node::Statement(Rc::new(Statement::Function {
        name,
        parameters: parameters.into(),
        body: Rc::new(parse_expression(tokens)?),
    })))
}

pub fn parse_statement(tokens: &mut Vec<Token>) -> Result<Node> {
    while let Some(Token { kind: TokenKind::SpecialCharacter(';'), .. }) = tokens.last() {
        tokens.pop();
    }

    let pos_start = current_position(tokens);
    let node = match tokens.last().map(|t| &t.kind) {
        Some(TokenKind::Keyword(k)) if k == "let" => parse_variable(tokens),
        Some(TokenKind::Keyword(k)) if k == "fn" => parse_function(tokens),
        Some(_) => {
            let result = parse_expression(tokens);
            match result? {
                Expression::Identifier(name) => match tokens.last().map(|t| &t.kind) {
                    Some(TokenKind::Assignment) => {
                        tokens.pop();
                        let value = parse_expression(tokens)?;
                        Ok(Node::Statement(Rc::new(Statement::Assignment {
                            name,
                            value: Rc::new(value),
                        })))
                    }
                    _ => Ok(Node::Expression(Rc::new(Expression::Identifier(name)))),
                },
                Expression::Dereference(target) => match tokens.last().map(|t| &t.kind) {
                    Some(TokenKind::Assignment) => {
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
        None => Err(MovaError::Parser {
            error: ParserError::UnexpectedEndOfInput,
            position: pos_start,
        }),
    }?;

    while let Some(Token { kind: TokenKind::SpecialCharacter(';'), .. }) = tokens.last() {
        tokens.pop();
    }

    Ok(node)
}
