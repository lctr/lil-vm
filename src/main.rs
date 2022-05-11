pub mod assembler;
pub mod bytecode;
pub mod data;
pub mod repl;
pub mod vm;

fn main() {
    repl::Repl::new().run()
}
