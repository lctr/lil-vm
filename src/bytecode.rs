/// ## Bytecode
/// The following enum variants comprise the `Instruction` set modeling the
/// assembly language simulated by the VM.
///
/// Note that in the documentation below for each variant, the opcode names used
/// in the `syntax` sections may not exactly line up with the `OpCode` variant
/// name. This is *purely* for aesthetic reasons (the author doesn't care for
/// enums in all caps).
///
/// ## Size and alignment
/// Instructions will be 32-bits long and have the following possible forms:
///
/// 1. op_code (`8` bits)
/// 2. op_code (`8` bits), operand (`24` bits)
/// 3. op_code (`8` bits), operand (`8` bits), operand (`16` bits)
/// 4. op_code (`8` bits), operand (`8` bits) x 3 (= `24` bits)
#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    /// Load into register R the value X
    ///
    /// __syntax:__ `LOAD $R #X`
    Load = 0,
    /// Adds the values found in the first two registers and stores it in the
    /// third register
    ///
    /// __syntax:__ `ADD $REG $REG $REG`
    ///
    /// ### Example
    /// ```txt
    /// LOAD $0 #10
    /// LOAD $1 #15
    /// ADD $0 $1 $2
    /// ```
    /* ARITHMETIC */
    Add = 1,
    /// Substracts the values found in the first two registers and stores the
    /// result in the third register
    ///
    /// __syntax:__ `SUB $REG $REG $REG`
    Sub,
    Mul,
    /// Unlike `ADD`, `SUB`, or `MUL`, this operation is not algebraically
    /// closed over the integers (which is the type of values stored in
    /// registers), so it will need special care
    Div,
    /// Absolute jump; will modify the program counter to point to the
    /// INSTRUCTION AT THE GIVEN BYTE INDEX
    ///
    /// __syntax:__ `JMP $CODE_IDX`
    ///
    /// The following example would cause an infinite loop! this is because
    /// at instruction 0, we're told to jump back to instruction 0
    /// ```txt
    /// LOAD $0 #0
    /// JMP $0
    /// ```
    /* CONTROL FLOW */
    Jump,
    /// Relative jump in the FORWARD direction. The argument is the register
    /// number in which the number of bytes to move forward is stored.
    JumpF,
    /// Relative jump in the BACKWARD direction. The argument is the register
    /// number in which the number of bytes to move backward is stored.
    JumpB,

    /* COMPARISONS */
    /// Equality comparison; checks the values in both registers given and tests
    /// for equality.
    ///
    /// __syntax:__ `EQ $0 $1`
    ///
    /// Note: where is the result stored? we can require a 3rd operand to define
    /// where to store the result, OR we can store the result in a similar
    /// manner as we do with remainders, i.e., in their own special register
    /// (read: field) within the VM struct.
    ///
    /// The result of this is stored in its own special register, which CANNOT
    /// be loaded or used for anything outside of the instructions that rely on
    /// it, such as `Eq`, `JumpEq`, etc.
    ///
    Eq,
    /// Exactly what you'd think the opposite of `Eq` would be
    ///
    /// __syntax:__ `NEQ $0 $1`
    NotEq,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    /// Conditional branching, aka `jump if equal`. It takes a register address
    /// as the argument and will jump to the value stored in that register IF
    /// the VM's `cmp` flag is set to `true`.
    JumpEq,

    /* IDK LOL */
    /// Halts the program
    Halt,
    /// INVALID opcode; stops VM with an error
    Bad,
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        Self::from_byte(byte)
    }
}

impl OpCode {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => OpCode::Load,
            1 => OpCode::Add,
            2 => OpCode::Sub,
            3 => OpCode::Mul,
            4 => OpCode::Div,
            5 => OpCode::Jump,
            6 => OpCode::JumpF,
            7 => OpCode::JumpB,
            8 => OpCode::Eq,
            9 => OpCode::NotEq,
            10 => OpCode::Greater,
            11 => OpCode::Less,
            12 => OpCode::GreaterEq,
            13 => OpCode::LessEq,
            14 => OpCode::JumpEq,
            254 => OpCode::Halt,
            _ => OpCode::Bad,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    opcode: OpCode,
}

impl Instruction {
    pub fn new(opcode: OpCode) -> Self {
        Self { opcode }
    }
}
