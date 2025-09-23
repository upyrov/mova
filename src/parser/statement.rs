use crate::{
    lexer::Token,
    parser::{expression::*, node::Node},
};

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

fn parse_variable_declaration(tokens: &mut Vec<Token>) -> Option<Node> {
    tokens.pop();

    let name = match tokens
        .pop()
        .expect("Expected identifier after `let` keyword")
    {
        Token::Identifier(i) => i,
        t => panic!("Expected identifier but got: {:?}", t),
    };

    match tokens.pop().expect("Expected assignmnet after identifier") {
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

    match tokens
        .pop()
        .expect("Expected assignment before function body")
    {
        Token::Assignment => {}
        _ => panic!("Expected assignment before function body"),
    }

    let body = parse_expression(tokens).expect("Expected function body");
    Some(Node::Statement(Statement::Function {
        name,
        parameters,
        body,
    }))
}

pub fn parse_statement(tokens: &mut Vec<Token>) -> Option<Node> {
    match tokens.last()? {
        Token::Keyword(k) => match k.as_str() {
            "let" => parse_variable_declaration(tokens),
            "fn" => parse_function(tokens),
            k => panic!("Unexpected keyword found: {}", k),
        },
        _ => parse_expression(tokens).map(|t| Node::Expression(t)),
    }
}
