///! NOTE: THE MACHINE IN WHICH THIS WAS WRITTEN USES BIG ENDIAN!!!!!!
///
/// Todo: maybe figure something out abt this later idk
use crate::instrs::OpCode;

#[derive(Debug)]
pub struct Vm {
    /// simulated hardware 32 registers
    regs: [i32; 32],
    /// program counter tracks which byte is being executed
    pc: usize,
    /// program bytecode being run
    code: Vec<u8>,
    /// special register holding the result (remainder) for the last division
    /// operation. since register values are already signed, we don't need to
    /// carry sign information on the remainder
    rem: u32,
    /// special register holding the result of the last comparison operation
    cmp: bool,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            regs: [0; 32],
            pc: 0,
            code: Default::default(),
            rem: 0,
            cmp: false,
        }
    }

    #[inline]
    fn is_done(&self) -> bool {
        self.pc >= self.code.len()
    }

    /// Execute one instruction, as opposed to running all instructions in the
    /// code
    pub fn tick(&mut self) {
        self.exec_instruction();
    }

    pub fn run(&mut self) {
        let mut done = self.is_done();
        while !done {
            // NOTE: we will want to take a look at optimizing this later so
            // that we don't add *another* call stack to the interpreter's loop
            done = self.exec_instruction();
        }
    }

    /// Executes the next instruction and returns whether the program is done
    /// running or not
    fn exec_instruction(&mut self) -> bool {
        // the program counter should NEVER exceed the length of the program
        // itself!!!!
        if self.is_done() {
            #[cfg(test)]
            eprintln!(
                "Uh-oh! Program counter ({}) exceeded the length of the compiled bytecode ({})",
                self.pc,
                self.code.len()
            );
            return true;
        } else {
            match self.decode_opcode() {
                OpCode::Halt => {
                    #[cfg(test)]
                    println!("encontered instruction: HALT");
                    return true;
                }
                OpCode::Bad => {
                    println!("encountered instruction: UNKNOWN");
                    return true;
                }
                OpCode::Load => {
                    // LOAD $REG #VAL
                    // next byte should contain the register we're loading into
                    let reg = self.next_8_bits() as usize;
                    // since LOAD takes 2 operands, it has a layout of
                    // 8 bits + 8 bits + 16 bits
                    // ^^^^^^   ^^^^^^   ^^^^^^^
                    // opcode  register  value
                    let val = self.next_16_bits() as u32;
                    // since our registers hold i32 values
                    self.regs[reg] = val as i32;
                    // the next 8 bits in line should be an opcode !!
                }
                OpCode::Add => {
                    // ADD (val in) R1 with (val in) R2 and store in R3
                    // get operand (reg) address and read value
                    let r1 = self.regs[self.next_8_bits() as usize];
                    // get next operand (reg) address and read value
                    let r2 = self.regs[self.next_8_bits() as usize];
                    // get last operand (reg) address and store sum
                    let r3 = self.next_8_bits() as usize;
                    self.regs[r3] = r1 + r2;
                }
                OpCode::Sub => {
                    let r1 = self.regs[self.next_8_bits() as usize];
                    let r2 = self.regs[self.next_8_bits() as usize];
                    let r3 = self.next_8_bits() as usize;
                    self.regs[r3] = r1 - r2;
                }
                OpCode::Mul => {
                    let r1 = self.regs[self.next_8_bits() as usize];
                    let r2 = self.regs[self.next_8_bits() as usize];
                    let r3 = self.next_8_bits() as usize;
                    self.regs[r3] = r1 * r2;
                }
                // since division is not algebraically closed over integers we
                // could store floats elsewhere, but instead we'll store
                // *remainders* and keep things integer based.
                //
                // recall that for integers `a, b, q, r`, we have `a / b = q +
                // r` where q is the *quotient* and r is the *remainder*
                //
                // so what do? store quotient in register and store remainder
                // separately in the VM's `rem` field
                OpCode::Div => {
                    let r1 = self.regs[self.next_8_bits() as usize];
                    let r2 = self.regs[self.next_8_bits() as usize];
                    let r3 = self.next_8_bits() as usize;
                    // integer division
                    self.regs[r3] = r1 / r2;
                    self.rem = (r1 % r2) as u32;
                }
                OpCode::Jump => {
                    let dest = self.regs[self.next_8_bits() as usize];
                    self.pc = dest as usize;
                }
                OpCode::JumpF => {
                    let dest = self.regs[self.next_8_bits() as usize];
                    self.pc += dest as usize;
                }
                OpCode::JumpB => {
                    let dest = self.regs[self.next_8_bits() as usize];
                    self.pc -= dest as usize;
                }
                OpCode::Eq => {
                    let r1 = self.regs[self.next_8_bits() as usize];
                    let r2 = self.regs[self.next_8_bits() as usize];
                    // update the special comparison register to hold the result
                    self.cmp = r1 == r2;
                    // then proceed with the next 8 bits?
                    self.next_8_bits();
                }
                OpCode::NotEq => {
                    let r1 = self.regs[self.next_8_bits() as usize];
                    let r2 = self.regs[self.next_8_bits() as usize];
                    // update the special comparison register to hold the result
                    self.cmp = r1 != r2;
                    // then proceed with the next 8 bits?
                    self.next_8_bits();
                }
                OpCode::Greater => {
                    let r1 = self.regs[self.next_8_bits() as usize];
                    let r2 = self.regs[self.next_8_bits() as usize];
                    // update the special comparison register to hold the result
                    self.cmp = r1 > r2;
                    // then proceed with the next 8 bits?
                    self.next_8_bits();
                }
                OpCode::Less => {
                    let r1 = self.regs[self.next_8_bits() as usize];
                    let r2 = self.regs[self.next_8_bits() as usize];
                    // update the special comparison register to hold the result
                    self.cmp = r1 < r2;
                    // then proceed with the next 8 bits?
                    self.next_8_bits();
                }
                OpCode::GreaterEq => {
                    let r1 = self.regs[self.next_8_bits() as usize];
                    let r2 = self.regs[self.next_8_bits() as usize];
                    // update the special comparison register to hold the result
                    self.cmp = r1 >= r2;
                    // then proceed with the next 8 bits?
                    self.next_8_bits();
                }
                OpCode::LessEq => {
                    let r1 = self.regs[self.next_8_bits() as usize];
                    let r2 = self.regs[self.next_8_bits() as usize];
                    // update the special comparison register to hold the result
                    self.cmp = r1 <= r2;
                    // then proceed with the next 8 bits?
                    self.next_8_bits();
                }
                OpCode::JumpEq => {
                    let reg = self.next_8_bits() as usize;
                    let dest = self.regs[reg];
                    if self.cmp {
                        self.pc = dest as usize
                    }
                }
            };
        }
        self.pc >= self.code.len()
    }

    fn decode_opcode(&mut self) -> OpCode {
        let opcode = OpCode::from(self.code[self.pc]);
        self.pc += 1;
        opcode
    }

    fn next_8_bits(&mut self) -> u8 {
        let byte = self.code[self.pc];
        self.pc += 1;
        byte
    }

    fn next_16_bits(&mut self) -> u16 {
        let dword = ((self.code[self.pc] as u16) << 8) | self.code[self.pc + 1] as u16;
        // increment twice, since the pc increments *bytes*
        self.pc += 2;
        dword
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_new_vm() {
        let vm = Vm::new();
        assert_eq!(vm.regs[0], 0i32);
    }

    #[test]
    fn test_bit_stuff() {
        let three_bytes = vec![200u8, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0];
        let b1 = (three_bytes[0] as u16) << 8;
        assert_eq!(b1.to_be_bytes(), [200u8, 0]);
        assert_eq!(b1.to_le_bytes(), [0u8, 200]);
        let b2 = three_bytes[8] as u16;
        assert_eq!(b2.to_be_bytes(), [0u8, 5]);
        assert_eq!(b2.to_le_bytes(), [5u8, 0]);
        let b = b1 | b2;
        assert_eq!(b.to_be_bytes(), [200u8, 5]);
        assert_eq!(b.to_le_bytes(), [5u8, 200]);
    }

    #[test]
    fn test_opcode_halt() {
        let mut vm = Vm::new();
        let code = vec![0, 0, 0, 0];
        vm.code = code;
        vm.run();
        assert_eq!(vm.pc, 1)
    }

    #[test]
    fn test_opcode_bad() {
        let mut vm = Vm::new();
        let code = vec![200, 0, 0, 0];
        vm.code = code;
        vm.run();
        assert_eq!(vm.pc, 1)
    }

    #[test]
    fn test_opcode_load() {
        let mut vm = Vm::new();
        // represent 500 using LE u8
        vm.code = vec![0, 0, 1, 244];
        vm.exec_instruction();
        assert_eq!(vm.regs[0], 500)
    }

    #[test]
    fn test_opcode_add() {
        let mut vm = Vm::new();
        // represent 500 using BE u8, so 500 = [1, 244]
        vm.code = vec![
            OpCode::Load as u8,
            0,
            1,
            244, // load 500 into $0
            OpCode::Load as u8,
            1,
            1,
            244, // load 500 into $1
            OpCode::Add as u8,
            0,
            1,
            2, // add $0 $1 $2
        ];
        vm.run();
        println!("{:?}", &vm);
        assert_eq!(vm.regs[2], 1000)
    }

    #[test]
    fn test_opcode_mul() {
        let mut vm = Vm::new();
        vm.code = vec![
            OpCode::Load as u8,
            0,
            1,
            1,
            OpCode::Load as u8,
            1,
            2,
            2,
            OpCode::Mul as u8,
            0,
            1,
            2,
        ];
        vm.run();
        println!("{}", vm.regs[2]);
    }

    #[test]
    fn test_opcode_jump() {
        let mut vm = Vm::new();
        // manually store `1` in r0, so that when we jump to it the program
        // counter is set to this value
        vm.regs[0] = 1;
        vm.code = vec![OpCode::Jump as u8, 0, 0, 0];
        vm.tick();
        assert_eq!(vm.pc, 1)
    }

    #[test]
    fn test_opcode_jumpf() {
        let mut vm = Vm::new();
        vm.regs[0] = 2;
        // uwu i think this would cause an infinite loop
        vm.code = vec![OpCode::JumpF as u8, 0, 0, 0, OpCode::Jump as u8, 0, 0, 0];
        vm.tick();
        assert_eq!(vm.pc, 4)
    }

    #[test]
    fn test_opcode_eq() {
        let mut vm = Vm::new();
        // let's set the values of 2 registers equal
        vm.regs[0] = 10;
        vm.regs[1] = 10;
        vm.code = vec![OpCode::Eq as u8, 0, 1, 0, OpCode::Eq as u8, 0, 1, 0];
        vm.tick();
        // 10 == 10
        assert!(vm.cmp);
        // now let's change one of the registers so that they're no longer equal
        vm.regs[1] = 20;
        vm.tick();
        // 10 != 20
        assert!(!vm.cmp)
    }

    #[test]
    fn test_opcode_jeq() {
        let mut vm = Vm::new();
        vm.regs[0] = 7;
        vm.cmp = true;
        vm.code = vec![OpCode::JumpEq as u8, 0, 0, 0, 17, 0, 0, 0, 17, 0, 0, 0];
        vm.tick();
        assert_eq!(vm.pc, 7);
        println!("{:?}", &vm)
    }
}
