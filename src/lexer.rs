#[derive(Clone, Debug, PartialEq)]
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
        if c.is_whitespace() {
            continue;
        }

        match c {
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut value = String::from(c);
                while let Some(l) = input.peek() {
                    match l {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            value += &input.next().unwrap().to_string()
                        }
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
            _ => panic!("Unexpected character found: {}", c),
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_tokenizes_identifier() {
        let identifiers = vec![
            Token::Identifier("Mova".into()),
            Token::Identifier("loves".into()),
            Token::Identifier("ownership".into()),
        ];
        assert_eq!(tokenize("Mova loves ownership"), identifiers);
    }

    #[test]
    fn it_tokenizes_number() {
        let numbers = vec![
            Token::Number("2342345".into()),
            Token::Number("123456789".into()),
            Token::Number("314".into()),
            Token::Number("1".into()),
        ];
        assert_eq!(tokenize("2342345 123456789 314 1"), numbers);
    }

    #[test]
    fn it_tokenizes_operator() {
        let operators = vec![
            Token::Operator('+'.into()),
            Token::Operator('-'.into()),
            Token::Operator('-'.into()),
            Token::Operator('/'.into()),
        ];
        assert_eq!(tokenize("+-- /"), operators);
    }

    #[test]
    fn it_tokenizes_special_character() {
        let special_characters = vec![
            Token::SpecialCharacter('{'.into()),
            Token::SpecialCharacter('}'.into()),
            Token::SpecialCharacter('}'.into()),
        ];
        assert_eq!(tokenize("{}}"), special_characters);
    }

    #[test]
    fn it_tokenizes_assignment() {
        assert_eq!(tokenize("="), vec![Token::Assignment]);
    }
}
