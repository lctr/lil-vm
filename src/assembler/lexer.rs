use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use crate::bytecode::OpCode;

use crate::data::{Int, Reg};

/// Lexical syntax
///
/// ```txt
/// Program := { Instruction }
///
/// Instruction := OpCode Register Integer "\n"
///
/// OpCode := [Letter] " "
///
/// Register := "$" Number " "
///
/// Int := "#" Number
///
/// Number := "0" | ... | "9"
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Lexeme {
    Newline,
    Op(OpCode),
    Reg(Reg),
    Int(Int),
    Label(&'static str),
    InvalidInt(usize, usize),
    InvalidReg(usize, usize),
    Unknown(usize, usize),
    Eof,
}

impl Lexeme {
    pub fn is_newline(&self) -> bool {
        matches!(self, Lexeme::Newline)
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, Lexeme::Eof)
    }
}

impl std::fmt::Display for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lexeme::Newline => write!(f, "\n"),
            Lexeme::Op(op) => write!(f, "{}", op),
            Lexeme::Reg(r) => write!(f, "{}", r),
            Lexeme::Int(n) => write!(f, "{}", n),
            Lexeme::Label(s) => write!(f, "{}", s),
            Lexeme::InvalidInt(a, b) => write!(f, "<INVALID_INT@{}:{}>", a, b),
            Lexeme::InvalidReg(a, b) => write!(f, "<INVALID_REG@{}:{}>", a, b),
            Lexeme::Unknown(a, b) => write!(f, "<UNKNOWN_TOK@{}:{}>", a, b),
            Lexeme::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub lexeme: Lexeme,
}

impl Token {}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

#[derive(Clone, Debug)]
pub struct Lexer<'t> {
    input: &'t str,
    chars: Peekable<Chars<'t>>,
    current: Option<Token>,
    lncol: (u32, u32),
    byte: usize,
    eol: bool,
}

impl<'t> Lexer<'t> {
    pub fn new(s: &'t str) -> Self {
        Lexer {
            input: s,
            chars: s.chars().peekable(),
            current: None,
            lncol: (1, 0),
            byte: 0,
            eol: s.starts_with('\n'),
        }
    }

    pub fn source(&self) -> &str {
        self.input
    }

    pub fn coord(&self) -> (u32, u32) {
        self.lncol
    }

    pub fn peek_char(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    pub fn next_char(&mut self) -> Option<char> {
        match self.chars.next() {
            Some(c) => match c {
                '\n' => {
                    self.byte += c.len_utf8();
                    let (ln, _col) = self.lncol;
                    self.lncol = (ln + 1, 0);
                    Some(c)
                }
                s => {
                    self.lncol.1 += 1;
                    self.byte += s.len_utf8();

                    Some(s)
                }
            },
            None => None,
        }
    }

    pub fn peek_tok(&mut self) -> Option<&Token> {
        match self.current {
            Some(ref tok) => Some(tok),
            None => match self.token() {
                Token {
                    lexeme: Lexeme::Eof,
                    ..
                } => None,
                t => {
                    self.current = Some(t);
                    self.current.as_ref()
                }
            },
        }
    }

    fn eat_while(&mut self, mut f: impl FnMut(&char) -> bool) -> (usize, usize) {
        let start = self.byte;
        while let Some(c) = self.peek_char() {
            if f(c) {
                self.next_char();
            } else {
                break;
            }
        }
        (start, self.byte)
    }

    fn eat_whitespace(&mut self) -> (usize, usize) {
        let line = self.lncol.0;
        let span = self.eat_while(|c| c.is_whitespace());
        self.eol = line < self.lncol.0;
        span
    }

    pub fn token(&mut self) -> Token {
        self.eat_whitespace();

        if self.eol {
            self.eol = false;
            return Token {
                lexeme: Lexeme::Newline,
            };
        }

        if self.peek_char().is_none() {
            return Token {
                lexeme: Lexeme::Eof,
            };
        }

        match self.peek_char() {
            // comments
            Some(';') => {
                self.eat_while(|c| *c != '\n');
                self.token()
            }
            // register
            Some('$') => {
                self.next_char();
                let ch = self.peek_char();
                match ch {
                    Some(c) if c.is_digit(16) => match self.number::<u8, 16>() {
                        Ok(byte) => Token {
                            lexeme: Lexeme::Reg(Reg(byte)),
                        },
                        Err((_e, (start, end))) => {
                            println!("{}\nat {}", _e, &self.input[start..end]);
                            Token {
                                lexeme: Lexeme::InvalidInt(start, end),
                            }
                        }
                    },
                    _ => Token {
                        lexeme: Lexeme::Unknown(self.byte - '$'.len_utf8(), self.byte),
                    },
                }
            }
            // integer
            Some('#') => {
                self.next_char();
                match self.number::<i32, 10>() {
                    Ok(int) => Token {
                        lexeme: Lexeme::Int(Int(int)),
                    },
                    Err((_err, (start, end))) => Token {
                        lexeme: Lexeme::InvalidInt(start, end),
                    },
                }
            }
            // letter, beginning of identifier
            Some(c) if c.is_ascii_alphabetic() => self.ident(),
            Some(c) if c.is_digit(10) => match self.number::<i32, 10>() {
                Ok(int) => Token {
                    lexeme: Lexeme::Int(Int(int)),
                },
                Err((_err, (start, end))) => Token {
                    lexeme: Lexeme::InvalidInt(start, end),
                },
            },
            _ => todo!(),
        }
    }

    fn number<N: FromStr, const R: u32>(&mut self) -> Result<N, (N::Err, (usize, usize))> {
        let (start, end) = self.eat_while(|c| c.is_digit(R));
        match N::from_str(&self.input[start..end]) {
            Ok(n) => Ok(n),
            Err(err) => Err((err, (start, end))),
        }
    }

    fn ident(&mut self) -> Token {
        let (start, end) = self.eat_while(char::is_ascii_alphabetic);
        match OpCode::from_str(&self.input[start..end]) {
            Some(op) => Token {
                lexeme: Lexeme::Op(op),
            },
            None => Token {
                lexeme: Lexeme::Unknown(start, end),
            },
        }
    }
}

impl<'t> Iterator for Lexer<'t> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(t) => Some(t),
            None => match self.token() {
                Token {
                    lexeme: Lexeme::Eof,
                    ..
                } => None,
                t => Some(t),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_registers() {
        Lexer::new(" $1 $2").enumerate().for_each(|(i, tok)| {
            assert_eq!(
                Token {
                    lexeme: Lexeme::Reg(Reg(i as u8)),
                },
                tok
            )
        });
    }
}
