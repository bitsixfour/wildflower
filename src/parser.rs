use crate::search::{Expr, Field, FieldOp};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    And,
    Or,
    Not,
    Eq,
    NotEq,
    Field(String),
    Value(String),
}

pub fn parse_filter(input: &str) -> Option<Expr> {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return None;
    }
    Expr::create_eval(tokens)
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut expect_value = false;

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '"' | '\'' => {
                let quote = c;
                chars.next();
                let mut value = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == quote {
                        break;
                    }
                    value.push(ch);
                    chars.next();
                }
                chars.next();
                tokens.push(Token::Value(value));
                expect_value = false;
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                }
                tokens.push(Token::Eq);
                expect_value = true;
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::NotEq);
                    expect_value = true;
                }
            }
            'A'..='Z' | 'a'..='z' | '_' => {
                let mut word = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                        word.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let token = match word.as_str() {
                    "AND" => Token::And,
                    "OR" => Token::Or,
                    "NOT" => Token::Not,
                    "contains" => Token::NotEq,
                    _ => {
                        if expect_value {
                            Token::Value(word)
                        } else {
                            Token::Field(word)
                        }
                    }
                };
                if matches!(token, Token::Value(..)) {
                    expect_value = false;
                } else if matches!(token, Token::Field(..)) {
                    expect_value = true;
                }
                tokens.push(token);
            }
            '0'..='9' => {
                let mut num = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' {
                        num.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Value(num));
                expect_value = false;
            }
            _ => {
                chars.next();
            }
        }
    }
    tokens
}

pub fn create_arg_struct(tkn: &[Token], k: usize) -> Field {
    let field = match tkn.get(k) {
        Some(Token::Field(s)) => s.clone(),
        _ => panic!("expected Field token at position {k}"),
    };

    match tkn.get(k + 1) {
        Some(Token::Eq) => {
            let value = match tkn.get(k + 2) {
                Some(Token::Value(s)) => s.clone(),
                _ => panic!("expected Value token after Eq"),
            };
            Field {
                field,
                op: FieldOp::Eq,
                value,
            }
        }
        Some(Token::NotEq) => {
            let value = match tkn.get(k + 2) {
                Some(Token::Value(s)) => s.clone(),
                _ => panic!("expected Value token after NotEq"),
            };
            Field {
                field,
                op: FieldOp::NotEq,
                value,
            }
        }
        Some(Token::Value(v)) => Field {
            field,
            op: FieldOp::Contains,
            value: v.clone(),
        },
        _ => panic!("expected Value or operator after Field at position {k}"),
    }
}

impl Expr {
    fn create_eval(tkn: Vec<Token>) -> Option<Expr> {
        let mut i = 0;
        while i < tkn.len() {
            match tkn[i] {
                Token::And => {
                    let left = create_arg_struct(&tkn, 0);
                    let right = create_arg_struct(&tkn, i + 1);
                    return Some(Expr::And(Box::new(left), Box::new(right)));
                }
                Token::Or => {
                    let left = create_arg_struct(&tkn, 0);
                    let right = create_arg_struct(&tkn, i + 1);
                    return Some(Expr::Or(Box::new(left), Box::new(right)));
                }
                Token::Not => {
                    let inner = create_arg_struct(&tkn, i + 1);
                    let mut neg = inner;
                    neg.op = match neg.op {
                        FieldOp::Contains | FieldOp::Eq => FieldOp::NotEq,
                        FieldOp::NotEq => FieldOp::Eq,
                    };
                    return Some(Expr::Def(neg));
                }
                _ => {}
            }
            i += 1;
        }
        if !tkn.is_empty() {
            let arg = create_arg_struct(&tkn, 0);
            return Some(Expr::Def(arg));
        }
        None
    }
}
