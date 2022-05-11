use crate::bytecode::{Arity, OpCode};
use crate::data::{Int, Reg};

use super::lexer::{Lexeme, Lexer, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Unexpected(Token),
    ExpectedLabel(Token),
    ExpectedOpCode(Token),
    ExpectedOperand(Token),
    ExpectedInteger(Token),
    ExpectedRegister(Token),
    UnexpectedEof,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unexpected(t) => write!(f, "unexpected token `{}` found", t),
            Error::ExpectedLabel(t) => {
                write!(f, "expected a label token, but found `{}` instead", t)
            }
            Error::ExpectedOpCode(t) => {
                write!(f, "expected an opcode token, but found `{}` instead", t)
            }
            Error::ExpectedOperand(t) => write!(
                f,
                "expected either a register token or integer token, but found `{}` instead",
                t
            ),
            Error::ExpectedInteger(t) => {
                write!(f, "expected an integer token, but found `{}` instead", t)
            }
            Error::ExpectedRegister(t) => {
                write!(f, "expected a register token, but found `{}` instead", t)
            }
            Error::UnexpectedEof => write!(f, "unexpected end of input"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operand {
    Int(Int),
    Reg(Reg),
}

impl Operand {
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Operand::Int(Int(n)) => {
                let m = *n as u16;
                let [b1, b2] = [m, m >> 8];
                // todo: confirm endianness
                vec![b2 as u8, b1 as u8]
            }
            Operand::Reg(Reg(r)) => {
                vec![*r]
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    line: usize,
    label: Option<Token>,
    opcode: OpCode,
    operands: [Option<Operand>; Arity::MAX],
}

impl Instruction {
    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.push(self.opcode as u8);
        for oparg in self.operands {
            if let Some(arg) = oparg {
                bytes.extend(arg.bytes())
            }
        }
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub instrs: Vec<Instruction>,
    pub errors: Vec<Error>,
}

impl Program {
    pub fn bytes(&self) -> Vec<u8> {
        self.instrs.iter().flat_map(|instr| instr.bytes()).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Parser<'t> {
    lexer: Lexer<'t>,
}

impl<'t> Parser<'t> {
    pub fn new(src: &'t str) -> Self {
        Parser {
            lexer: Lexer::new(src),
        }
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.lexer.peek_tok()
    }

    pub fn bump(&mut self) -> Token {
        match self.peek() {
            Some(Token {
                lexeme: Lexeme::Eof,
                ..
            }) => Token {
                lexeme: Lexeme::Eof,
            },
            Some(_) => self.lexer.next().unwrap(),
            None => Token {
                lexeme: Lexeme::Eof,
            },
        }
    }

    pub fn is_done(&mut self) -> bool {
        matches!(
            self.peek(),
            Some(Token {
                lexeme: Lexeme::Eof,
                ..
            }) | None
        )
    }

    pub fn eat(&mut self, mut f: impl FnMut(&Lexeme) -> bool) -> Result<Token, Error> {
        if matches!(self.peek(), Some(t) if f(&t.lexeme)) {
            Ok(self.bump())
        } else {
            Err(Error::Unexpected(self.bump()))
        }
    }

    pub fn expect_opcode(&mut self) -> Result<OpCode, Error> {
        match self.peek() {
            Some(Token {
                lexeme: Lexeme::Op(op),
                ..
            }) => {
                let op = op.clone();
                self.bump();
                Ok(op)
            }
            _ => todo!(),
        }
    }

    pub fn many_while<X>(
        &mut self,
        mut f: impl FnMut(&Lexeme) -> bool,
        mut g: impl FnMut(&mut Self) -> Result<X, Error>,
    ) -> Result<Vec<X>, Error> {
        let mut nodes = vec![];
        while matches!(self.peek(), Some(t) if f(&t.lexeme)) {
            nodes.push(g(self)?);
        }
        Ok(nodes)
    }

    fn skip_newlines(&mut self) {
        let _ = self.many_while(
            |lx| lx.is_newline(),
            |p| {
                p.bump();
                Ok(())
            },
        );
    }

    pub fn program(&mut self) -> Result<Program, Error> {
        self.skip_newlines();
        let mut program = Program {
            instrs: vec![],
            errors: vec![],
        };
        while !self.is_done() {
            match self.instruction() {
                Ok(instr) => program.instrs.push(instr),
                Err(err) => {
                    program.errors.push(err);
                    let _ = self.many_while(
                        |lx| !lx.is_newline(),
                        |p| {
                            Ok({
                                p.bump();
                            })
                        },
                    );
                }
            }
            self.skip_newlines();
        }
        Ok(program)
    }

    pub fn instruction(&mut self) -> Result<Instruction, Error> {
        let opcode = self.expect_opcode()?;
        let line = self.lexer.coord().0 as usize;
        let mut instr = Instruction {
            line,
            label: None,
            opcode,
            operands: [None; Arity::MAX],
        };

        // we just have to make sure this never exceeds `Arity::MAX`, but
        // because we've hardcoded arities into *all* bytecode ops we know we'll
        // always safe to unwrap as well as stay within array bounds
        for i in 0..opcode.arity().unwrap().as_usize() {
            instr.operands[i] = self.operand().map(Some)?
        }

        Ok(instr)
    }

    pub fn operand(&mut self) -> Result<Operand, Error> {
        match self.peek() {
            Some(Token {
                lexeme: Lexeme::Int(n),
                ..
            }) => {
                let int = Operand::Int(*n);
                self.bump();
                Ok(int)
            }
            Some(Token {
                lexeme: Lexeme::Reg(r),
                ..
            }) => {
                let reg = Operand::Reg(*r);
                self.bump();
                Ok(reg)
            }
            _ => Err(Error::ExpectedOperand(self.bump())),
        }
    }

    pub fn register(&mut self) -> Result<Reg, Error> {
        match self.peek() {
            Some(Token {
                lexeme: Lexeme::Reg(r),
                ..
            }) => {
                let reg = *r;
                self.bump();
                Ok(reg)
            }
            Some(_) => Err(Error::ExpectedRegister(self.bump())),
            None => Err(Error::UnexpectedEof),
        }
    }

    pub fn integer(&mut self) -> Result<Int, Error> {
        match self.peek() {
            Some(Token {
                lexeme: Lexeme::Int(i),
                ..
            }) => {
                let int = *i;
                self.bump();
                Ok(int)
            }
            Some(_) => Err(Error::ExpectedInteger(self.bump())),
            None => Err(Error::UnexpectedEof),
        }
    }
    pub fn label(&mut self) -> Result<Token, Error> {
        match self.peek() {
            Some(Token {
                lexeme: Lexeme::Label(_),
                ..
            }) => Ok(self.bump()),
            Some(_) => Err(Error::ExpectedLabel(self.bump())),
            None => Err(Error::UnexpectedEof),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_instr() {
        let expected = Program {
            errors: vec![],
            instrs: vec![Instruction {
                line: 1,
                label: None,
                opcode: OpCode::Load,
                operands: [
                    Some(Operand::Reg(Reg(0))),
                    Some(Operand::Int(Int(100))),
                    None,
                ],
            }],
        };
        let program = Parser::new("load $0 #100").program();
        assert_eq!(program.as_ref().map(|prog| prog.bytes().len()), Ok(4));
        assert_eq!(program, Ok(expected))
    }
}
