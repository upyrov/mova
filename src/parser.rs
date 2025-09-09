use crate::lexer::Token;

#[derive(Debug)]
pub enum Expression {
    Identifier(String),
    Number(i32),
    BinaryExpression {
        operator: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Block(Vec<Node>),
}

#[derive(Debug)]
pub enum Statement {
    VariableDeclaration {
        name: String,
        value: Box<Expression>,
    },
}

#[derive(Debug)]
pub enum Node {
    Expression(Expression),
    Statement(Statement),
}

fn get_binding_power(operator: &str) -> (u8, u8) {
    match operator {
        "+" | "-" => (1, 2),
        "*" | "/" => (3, 4),
        o => panic!("Unexpected operator found: {}", o),
    }
}

fn parse_binary_expression(tokens: &mut Vec<Token>, binding_power: u8) -> Option<Expression> {
    let mut left = match tokens.pop()? {
        Token::Number(n) => Expression::Number(n.parse().unwrap()),
        Token::Identifier(i) => Expression::Identifier(i),
        t => panic!("Unexpected token found: {:?}", t),
    };

    while let Some(t) = tokens.last().cloned() {
        match t {
            Token::Operator(o) => {
                let (lbp, rbp) = get_binding_power(&o);
                if lbp < binding_power {
                    break;
                }

                tokens.pop();
                let right = Box::new(
                    parse_binary_expression(tokens, rbp)
                        .expect("Expected another expression but found none"),
                );
                left = Expression::BinaryExpression {
                    left: Box::new(left),
                    right,
                    operator: o,
                }
            }
            _ => break,
        }
    }

    Some(left)
}

fn parse_block(tokens: &mut Vec<Token>) -> Option<Expression> {
    match tokens.last()? {
        Token::SpecialCharacter('{') => {
            tokens.pop();
            let mut body = Vec::new();

            loop {
                match tokens.last()? {
                    Token::SpecialCharacter('}') => break,
                    _ => {
                        if let Some(node) = parse_statement(tokens) {
                            body.push(node);
                        }
                    }
                }
            }

            match tokens.pop()? {
                Token::SpecialCharacter('}') => Some(Expression::Block(body)),
                _ => panic!("Expected block to be closed"),
            }
        }
        _ => parse_binary_expression(tokens, 0),
    }
}

fn parse_expression(tokens: &mut Vec<Token>) -> Option<Expression> {
    parse_block(tokens)
}

fn parse_statement(tokens: &mut Vec<Token>) -> Option<Node> {
    match tokens.last()? {
        Token::Keyword(k) => match k.as_str() {
            "let" => {
                tokens.pop();
                let name = match tokens.pop()? {
                    Token::Identifier(i) => i,
                    t => panic!("Expected identifier but got: {:?}", t),
                };
                match tokens.pop()? {
                    Token::Assignment => {
                        let value =
                            Box::new(parse_expression(tokens).expect("Unexpected statement found"));
                        Some(Node::Statement(Statement::VariableDeclaration {
                            name,
                            value,
                        }))
                    }
                    t => panic!("Unexpected token found: {:?}", t),
                }
            }
            k => panic!("Unexpected keyword found: {}", k),
        },
        _ => parse_expression(tokens).map(|t| Node::Expression(t)),
    }
}

pub fn parse(mut tokens: Vec<Token>) -> Node {
    tokens.reverse();
    let mut program = Vec::new();

    while let Some(node) = parse_statement(&mut tokens) {
        program.push(node);
    }

    Node::Expression(Expression::Block(program))
}
