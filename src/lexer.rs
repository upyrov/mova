#[derive(Clone, Debug)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Number(String),
    Operator(String),
    Assignment,
    SpecialCharacter(char),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut input = input.chars().peekable();
    while let Some(c) = input.next() {
        match c {
            'a'..='z' => {
                let mut value = String::from(c);
                while let Some(l) = input.peek() {
                    match l {
                        'a'..='z' => value += &input.next().unwrap().to_string(),
                        _ => break,
                    }
                }
                let token = match value.as_str() {
                    "let" | "fn" => Token::Keyword(value),
                    _ => Token::Identifier(value),
                };
                tokens.push(token);
            }
            '0'..='9' => {
                let mut value = String::from(c);
                while let Some(l) = input.peek() {
                    match l {
                        '0'..='9' => value += &input.next().unwrap().to_string(),
                        _ => break,
                    }
                }
                tokens.push(Token::Number(value));
            }
            '+' | '-' | '*' | '/' | '(' | ')' => tokens.push(Token::Operator(c.into())),
            '=' => tokens.push(Token::Assignment),
            '{' | '}' | ',' => tokens.push(Token::SpecialCharacter(c)),
            ' ' => continue,
            _ => panic!("Unexpected character found: {}", c),
        }
    }

    tokens
}
