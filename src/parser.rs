use crate::lexer::Token;

#[derive(Debug)]
pub enum Expression {
    Identifier(String),
    Number(i32),
    UnaryExpression {
        operator: String,
        value: Box<Expression>,
    },
    BinaryExpression {
        operator: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        name: String,
        arguments: Vec<Expression>,
    },
    Block(Vec<Node>),
    Program(Vec<Node>),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Node {
    Expression(Expression),
    Statement(Statement),
}

fn get_postfix_binding_power(operator: &str) -> Option<(u8, ())> {
    match operator {
        "(" => Some((6, ())),
        _ => None,
    }
}

fn parse_call(tokens: &mut Vec<Token>, left: Expression) -> Expression {
    tokens.pop();
    let mut parameters = Vec::new();

    loop {
        match tokens.last() {
            Some(Token::Operator(o)) if o == ")" => {
                tokens.pop();
                break;
            }
            Some(_) => {
                let argument = parse_expression(tokens).expect("Expected argument expression");
                parameters.push(argument);

                match tokens.last() {
                    Some(Token::SpecialCharacter(',')) => {
                        tokens.pop();
                    }
                    Some(Token::Operator(o)) if o == ")" => {}
                    None => panic!("Expected argument list to be closed"),
                    _ => {
                        panic!("Expected another argument expression or argument list to be closed")
                    }
                }
            }
            None => panic!("Expected argument list to be closed"),
        }
    }

    match left {
        Expression::Identifier(i) => Expression::Call {
            name: i,
            arguments: parameters,
        },
        e => panic!("Expected identifier to be called but found {:?}", e),
    }
}

fn get_infix_binding_power(operator: &str) -> Option<(u8, u8)> {
    match operator {
        "+" | "-" => Some((1, 2)),
        "*" | "/" => Some((3, 4)),
        _ => None,
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
                if let Some((lbp, ())) = get_postfix_binding_power(&o) {
                    if lbp < binding_power {
                        break;
                    }

                    left = match o.as_str() {
                        "(" => parse_call(tokens, left),
                        _ => Expression::UnaryExpression {
                            operator: o,
                            value: Box::new(
                                parse_expression(tokens)
                                    .expect("Expected expression in unary expression"),
                            ),
                        },
                    };
                    continue;
                }

                if let Some((lbp, rbp)) = get_infix_binding_power(&o) {
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
                    };
                    continue;
                }

                break;
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
                match tokens.last().expect("Expected block to be closed") {
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

fn parse_variable_declaration(tokens: &mut Vec<Token>) -> Option<Node> {
    tokens.pop();

    let name = match tokens.pop()? {
        Token::Identifier(i) => i,
        t => panic!("Expected identifier but got: {:?}", t),
    };

    match tokens.pop()? {
        Token::Assignment => {
            let value = Box::new(parse_expression(tokens).expect("Unexpected statement found"));
            Some(Node::Statement(Statement::VariableDeclaration {
                name,
                value,
            }))
        }
        t => panic!("Unexpected token found: {:?}", t),
    }
}

fn parse_function(tokens: &mut Vec<Token>) -> Option<Node> {
    tokens.pop();

    let name = match tokens
        .pop()
        .expect("Expected function name after `fn` keyword")
    {
        Token::Identifier(i) => i,
        _ => panic!("Expected function name after `fn` keyword"),
    };
    match tokens
        .pop()
        .expect("Expected parameter list after function name")
    {
        Token::Operator(o) if o == "(" => {}
        _ => panic!("Expected parameter list after function name"),
    }

    let mut parameters = Vec::new();
    loop {
        match tokens.last().expect("Expected parameter list to be closed") {
            Token::Operator(o) if o == ")" => break,
            _ => {
                if let Some(t) = tokens.pop() {
                    if let Token::Identifier(i) = t {
                        parameters.push(i);
                    }
                }
            }
        }
    }

    match tokens.pop().expect("Expected parameter list to be closed") {
        Token::Operator(o) if o == ")" => {}
        _ => panic!("Expected parameter list to be closed"),
    }

    let body = parse_block(tokens).expect("Expected function body");
    Some(Node::Statement(Statement::Function {
        name,
        parameters,
        body,
    }))
}

fn parse_statement(tokens: &mut Vec<Token>) -> Option<Node> {
    match tokens.last()? {
        Token::Keyword(k) => match k.as_str() {
            "let" => parse_variable_declaration(tokens),
            "fn" => parse_function(tokens),
            k => panic!("Unexpected keyword found: {}", k),
        },
        _ => parse_expression(tokens).map(|t| Node::Expression(t)),
    }
}

pub fn parse(mut tokens: Vec<Token>) -> Node {
    tokens.reverse();
    let mut body = Vec::new();

    while let Some(node) = parse_statement(&mut tokens) {
        body.push(node);
    }

    Node::Expression(Expression::Program(body))
}
