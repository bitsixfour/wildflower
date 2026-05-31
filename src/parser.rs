use crate::search::{Expr, Field};

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

struct Parser {
    tokens: Vec<Token>,
    size: i32,
}


mod parse_exp {
    fn new(needed_data: &str) -> Vec<Token> {
        let vec: Vec<Token> = tokenize_and_filter(needed_data);
        vec
    }

    fn tokenize_and_filter(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();
        while let Some(&c) = chars.peek() {
            match c {
                /* uneeded: sort vec by just not including this
                ' ' => { chars.next(); }
                '(' => { tokens.push(Token::LParen); chars.next(); }
                ')' => { tokens.push(Token::RParen); chars.next(); }
                '"' | '\'' => {
                    let quote = c;
                    chars.next();
                    let mut value = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == quote { break; }
                        value.push(ch);
                        chars.next();
                    }
                    chars.next(); // closing quote
                    tokens.push(Token::Value(value));
                } */
                '=' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                       tokens.push(Token::Eq);
                    }
                }
                '!' => {
                    chars.next();
                    if chars.peek() == Some(&'=') {
                        tokens.push(Token::NotEq);
                    }
                }
                'A'..='Z' | 'a'..='z' => {
                    let mut word = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphabetic() {
                            word.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let token = match word.as_str() {
                        "AND"      => Token::And,
                        "OR"       => Token::Or,
                        "NOT"      => Token::Not,
                        "contains" => Token::NotEq,
                        _          => Token::Field(word),
                    };
                    tokens.push(token);
                }
                '0'..='9' => {
                    let mut num = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() {
                            num.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Value(num));
                }
                _ => { chars.next(); } 
                /* ex: ARTIST EQUALS VALUE OR ALBUM EQUALS WHATEVER */
            }
        }
        tokens
    }
}
enum Check {
    And,
    Or,
    None,
}

impl Expr {
    fn create_eval(tkn: Vec<Token>) -> Option<Expr> { 
      //  let mut xenu = Check::None;
        let mut special: bool = false;
        for i in &tkn { // TODO: OPTIMIZE THIS LATER 
            match i {
               Token::And => {
                   let arg_1 = create_arg_struct(tkn.clone(), 0);
                   let arg_2 = create_arg_struct(tkn, 3);
                   special = true;
                   return Some(Expr::And(arg_1,arg_2))


                }
                Token::Or => {
                   let arg_1 = create_arg_struct(tkn, 0);
                   let arg_2 = create_arg_struct(tkn.clone(), 3);
                   special = true;
                   return Some(Expr::Or(arg_1, arg_2))
                }
                _ => println!("IDK")
            }
        }
        if special == false {
            let arg = create_arg_struct(tkn, 0);
            return Some(Expr::Def(arg))
        }

        None
    }

}


pub fn create_arg_struct(tkn: Vec<Token>, k: usize) -> Field {
    let arg = Field {
        field: match tkn.get(k+1) {
            Some(Token::Field(s)) => s.clone(),
            _ => panic!("expected Field token"),
        },
        op: match tkn.get(k+2) {
            Some(Token::And) => true,
            Some(Token::Or) => false,
            _ => panic!("expected And/Or token"),
        },
        value: match tkn.get(k+3) {
            Some(Token::Value(s)) => s.clone(),
            _ => panic!("expected Value token"),
        },
    };
    arg

    

}
