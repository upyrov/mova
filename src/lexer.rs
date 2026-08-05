use crate::error::{Position, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Keyword(String),
    Identifier(String),
    Number(String),
    Boolean(bool),
    Operator(String),
    Assignment,
    SpecialCharacter(char),
    Unknown(char),
    EndOfFile,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut input = input.char_indices().peekable();
    let mut line = 1;
    let mut line_start_index = 0;

    while let Some((i, c)) = input.next() {
        if c.is_whitespace() {
            if c == '\n' {
                line += 1;
                line_start_index = i + 1;
            }
            continue;
        }

        let position = Position {
            line,
            character: i - line_start_index,
        };

        match c {
            '/' => {
                if let Some((_, '/')) = input.peek() {
                    input.next();
                    while let Some((n_i, n)) = input.next() {
                        if n == '\n' {
                            line += 1;
                            line_start_index = n_i + 1;
                            break;
                        }
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Operator(c.into()),
                        position,
                    });
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
                let kind = match value.as_str() {
                    "let" | "mut" | "fn" | "if" | "else" | "while" => TokenKind::Keyword(value),
                    "true" => TokenKind::Boolean(true),
                    "false" => TokenKind::Boolean(false),
                    _ => TokenKind::Identifier(value),
                };
                tokens.push(Token { kind, position });
            }
            '0'..='9' => {
                let mut value = String::from(c);
                while let Some(&(idx, l)) = input.peek() {
                    match l {
                        '0'..='9' => {
                            let (_, next) = input.next().unwrap();
                            value += &next.to_string()
                        }
                        'a'..='z' | 'A'..='Z' | '_' => {
                            let (_, next) = input.next().unwrap();
                            tokens.push(Token {
                                kind: TokenKind::Unknown(next),
                                position: Position {
                                    line,
                                    character: idx - line_start_index,
                                },
                            });
                        }
                        _ => break,
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::Number(value),
                    position,
                });
            }
            '+' | '-' | '*' | '(' | ')' | '&' => tokens.push(Token {
                kind: TokenKind::Operator(c.into()),
                position,
            }),
            '<' | '>' | '!' => {
                if let Some((_, '=')) = input.peek() {
                    input.next();
                    tokens.push(Token {
                        kind: TokenKind::Operator(format!("{c}=")),
                        position,
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Operator(c.into()),
                        position,
                    });
                }
            }
            '=' => {
                if let Some((_, '=')) = input.peek() {
                    input.next();
                    tokens.push(Token {
                        kind: TokenKind::Operator("==".into()),
                        position,
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Assignment,
                        position,
                    });
                }
            }
            '{' | '}' | ',' | ';' => tokens.push(Token {
                kind: TokenKind::SpecialCharacter(c),
                position,
            }),
            _ => {
                tokens.push(Token {
                    kind: TokenKind::Unknown(c),
                    position,
                });
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::EndOfFile,
        position: Position {
            line,
            character: input.size_hint().1.unwrap_or(0),
        },
    });

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tokens(input: &str, expected_kinds: Vec<TokenKind>) -> Result<()> {
        let tokens = tokenize(input)?;
        let mut kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        if kinds.last() == Some(&TokenKind::EndOfFile) {
            kinds.pop();
        }
        assert_eq!(kinds, expected_kinds);
        Ok(())
    }

    #[test]
    fn it_tokenizes_identifier() -> Result<()> {
        assert_tokens(
            "Mova loves ownership",
            vec![
                TokenKind::Identifier("Mova".into()),
                TokenKind::Identifier("loves".into()),
                TokenKind::Identifier("ownership".into()),
            ],
        )
    }

    #[test]
    fn it_tokenizes_number() -> Result<()> {
        assert_tokens(
            "2342345 123456789 314 1",
            vec![
                TokenKind::Number("2342345".into()),
                TokenKind::Number("123456789".into()),
                TokenKind::Number("314".into()),
                TokenKind::Number("1".into()),
            ],
        )
    }

    #[test]
    fn it_tokenizes_operator() -> Result<()> {
        assert_tokens(
            "+-- /",
            vec![
                TokenKind::Operator('+'.into()),
                TokenKind::Operator('-'.into()),
                TokenKind::Operator('-'.into()),
                TokenKind::Operator('/'.into()),
            ],
        )
    }

    #[test]
    fn it_tokenizes_special_character() -> Result<()> {
        assert_tokens(
            "{}}",
            vec![
                TokenKind::SpecialCharacter('{'.into()),
                TokenKind::SpecialCharacter('}'.into()),
                TokenKind::SpecialCharacter('}'.into()),
            ],
        )
    }

    #[test]
    fn it_tokenizes_assignment() -> Result<()> {
        assert_tokens("=", vec![TokenKind::Assignment])
    }

    #[test]
    fn it_skips_comment() -> Result<()> {
        assert_tokens(
            "1 // comment here\n2",
            vec![TokenKind::Number("1".into()), TokenKind::Number("2".into())],
        )
    }
}
