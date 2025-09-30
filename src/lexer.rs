use crate::error::{MovaError, Position, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Number(String),
    Operator(String),
    Assignment,
    SpecialCharacter(char),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut input = input.char_indices().peekable();
    let mut line = 1;

    while let Some((i, c)) = input.next() {
        if c.is_whitespace() {
            if c == '\n' {
                line += 1;
            }
            continue;
        }

        match c {
            '/' => {
                if let Some((_, '/')) = input.peek() {
                    input.next();
                    while let Some((_, n)) = input.next() {
                        if n == '\n' {
                            line += 1;
                            break;
                        }
                    }
                } else {
                    tokens.push(Token::Operator(c.into()));
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut value = String::from(c);
                while let Some((_, l)) = input.peek() {
                    match l {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            let (_, next) = input.next().unwrap();
                            value += &next.to_string();
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
                while let Some((_, l)) = input.peek() {
                    match l {
                        '0'..='9' => {
                            let (_, next) = input.next().unwrap();
                            value += &next.to_string()
                        }
                        _ => break,
                    }
                }
                tokens.push(Token::Number(value));
            }
            '+' | '-' | '*' | '(' | ')' => tokens.push(Token::Operator(c.into())),
            '=' => tokens.push(Token::Assignment),
            '{' | '}' | ',' => tokens.push(Token::SpecialCharacter(c)),
            _ => {
                return Err(MovaError::Lexer {
                    message: format!("Unexpected character: '{}'", c),
                    position: Position { line, character: i },
                });
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_tokenizes_identifier() -> Result<()> {
        let identifiers = vec![
            Token::Identifier("Mova".into()),
            Token::Identifier("loves".into()),
            Token::Identifier("ownership".into()),
        ];
        assert_eq!(tokenize("Mova loves ownership")?, identifiers);
        Ok(())
    }

    #[test]
    fn it_tokenizes_number() -> Result<()> {
        let numbers = vec![
            Token::Number("2342345".into()),
            Token::Number("123456789".into()),
            Token::Number("314".into()),
            Token::Number("1".into()),
        ];
        assert_eq!(tokenize("2342345 123456789 314 1")?, numbers);
        Ok(())
    }

    #[test]
    fn it_tokenizes_operator() -> Result<()> {
        let operators = vec![
            Token::Operator('+'.into()),
            Token::Operator('-'.into()),
            Token::Operator('-'.into()),
            Token::Operator('/'.into()),
        ];
        assert_eq!(tokenize("+-- /")?, operators);
        Ok(())
    }

    #[test]
    fn it_tokenizes_special_character() -> Result<()> {
        let special_characters = vec![
            Token::SpecialCharacter('{'.into()),
            Token::SpecialCharacter('}'.into()),
            Token::SpecialCharacter('}'.into()),
        ];
        assert_eq!(tokenize("{}}")?, special_characters);
        Ok(())
    }

    #[test]
    fn it_tokenizes_assignment() -> Result<()> {
        assert_eq!(tokenize("=")?, vec![Token::Assignment]);
        Ok(())
    }

    #[test]
    fn it_skips_comment() -> Result<()> {
        assert_eq!(
            tokenize("1 // comment here\n2")?,
            vec![Token::Number("1".into()), Token::Number("2".into())]
        );
        Ok(())
    }
}
