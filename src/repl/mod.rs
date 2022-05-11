use stringy::stringy;

use crate::vm::Vm;
use std::{
    io::{self, Write},
    num::ParseIntError,
};

const STARTUP_MSG: &'static str = "Hello! I'm a machine.";
const PROMPT: &'static str = ">> ";

stringy! { Cmd =
    Quit ":quit" | ":q" | ":Q"
    History ":history" | ":h" | ":hist"
    Program ":program" | ":prog"
    Registers ":registers" | ":r"
}

pub struct Repl {
    vm: Vm,
    log: Vec<String>,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            vm: Vm::new(),
            log: vec![],
        }
    }

    /// Stores the input in the logs if it is unique
    pub fn save_input(&mut self, input: String) {
        if !self.log.contains(&input) {
            self.log.push(input.into())
        }
    }

    pub fn run(&mut self) {
        println!("{}", STARTUP_MSG);

        loop {
            let mut buf = String::new();

            // block call until user provides input (remember to flush this out)
            let stdin = std::io::stdin();
            print!("{}", PROMPT);
            io::stdout().flush().unwrap();
            stdin
                .read_line(&mut buf)
                .expect("unable to read user input");

            let buf = buf.trim();
            match Cmd::from_str(buf) {
                Some(c) => match c {
                    Cmd::Quit => {
                        print!("k bye");
                        std::process::exit(0);
                    }
                    Cmd::History => {
                        println!("history {{");
                        for line in &self.log[..] {
                            println!("{}", line)
                        }
                        println!("}}")
                    }
                    Cmd::Program => {
                        println!("code {{");
                        for op in self.vm.instructions() {
                            println!("    {:?}", op)
                        }
                        println!("}}");
                    }
                    Cmd::Registers => {
                        println!("registers {{");
                        for (a, r) in self.vm.regs.iter().enumerate() {
                            println!("\t0x{:x}\t{:?}", a, r)
                        }
                        println!("}}")
                    }
                },
                None => {
                    match parse_hex(buf) {
                        Ok(bytes) => {
                            self.save_input(buf.into());
                            for byte in bytes {
                                self.vm.add_byte(byte)
                            }
                        }
                        Err(_) => {
                            println!("Invalid hex string provided. Expected 4 groups of 2 hex characters (0-9a-fA-F).");
                            continue;
                        }
                    };
                    self.vm.tick()
                }
            }
        }
    }
}

/// Parse a hexadecimal string without the leading hex prefix `0x`.
pub fn parse_hex(input: &str) -> Result<Vec<u8>, ParseIntError> {
    let mut bytes = vec![];
    for chunk in input.split(" ") {
        match u8::from_str_radix(&chunk, 16) {
            Ok(byte) => bytes.push(byte),
            Err(err) => return Err(err),
        }
    }
    Ok(bytes)
}
