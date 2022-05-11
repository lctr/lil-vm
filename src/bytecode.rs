#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arity(pub usize);

impl Arity {
    pub const MAX: usize = 3;

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl PartialEq<usize> for Arity {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Arity> for usize {
    fn eq(&self, other: &Arity) -> bool {
        self == &other.0
    }
}

impl PartialOrd<usize> for Arity {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<Arity> for usize {
    fn partial_cmp(&self, other: &Arity) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

// TRYIN OUT SOMETHIN NEW also: what are the (perf) differences between
// handcoding out each opcode value vs using discriminants and u8 repr vs
// indexing (modulo max for safety reasons) into a predefined array of
// instances?
// ok well more ops for indexing so probably not that great
stringy::stringy! {
    /// ## Bytecode
    /// The following enum variants comprise the `Instruction` set modeling the
    /// assembly language simulated by the VM.
    ///
    /// Note that in the documentation below for each variant, the opcode names
    /// used in the `syntax` sections may not exactly line up with the `OpCode`
    /// variant name. This is *purely* for aesthetic reasons (the author doesn't
    /// care for enums in all caps).
    ///
    /// ## Size and alignment
    /// Instructions will be 32-bits long and have the following possible forms:
    ///
    /// 1. op_code (`8` bits)
    /// 2. op_code (`8` bits), operand (`24` bits)
    /// 3. op_code (`8` bits), operand (`8` bits), operand (`16` bits)
    /// 4. op_code (`8` bits), operand (`8` bits) x 3 (= `24` bits)
    OpCode { arity: Arity }
        =
        /// Load into register R the value X
        ///
        /// __syntax:__ `LOAD $R #X`
        Load "load" | "LOAD" { Arity(2) }
        /// Adds the values found in the first two registers and stores it in
        /// the third register
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
        Add "add" | "ADD" { Arity(2) }
        Sub "sub" | "SUB" { Arity(2) }
        Mul "mul" | "MUL" { Arity(2) }
        /// Unlike `ADD`, `SUB`, or `MUL`, this operation is not algebraically
        /// closed over the integers (which is the type of values stored in
        /// registers), so it will need special care
        Div "div" | "DIV" { Arity(2) }
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
        Jump "jmp" | "JMP" { Arity(1) }
        /// Relative jump in the FORWARD direction. The argument is the register
        /// number in which the number of bytes to move forward is stored.
        JumpF "jmpf" | "JMPF" { Arity(1) }
        /// Relative jump in the BACKWARD direction. The argument is the
        /// register index in which the number of bytes to move backward is
        /// stored.
        JumpB "jmpb" | "JMPB" { Arity(1) }

        /* COMPARISONS */
        /// Equality comparison; checks the values in both registers given and
        /// tests for equality.
        ///
        /// __syntax:__ `EQ $0 $1`
        ///
        /// Note: where is the result stored? we can require a 3rd operand to
        /// define where to store the result, OR we can store the result in a
        /// similar manner as we do with remainders, i.e., in their own special
        /// register (read: field) within the VM struct.
        ///
        /// The result of this is stored in its own special register, which
        /// CANNOT be loaded or used for anything outside of the instructions
        /// that rely on it, such as `Eq`, `JumpEq`, etc.
        ///
        Eq "eq" | "EQ" { Arity(2) }
        NotEq "neq" | "NEQ" { Arity(2) }
        Greater "gt" | "GT" { Arity(2) }
        Less "lt" | "LT" { Arity(2) }
        GreaterEq "gte" | "GTE" { Arity(2) }
        LessEq "lte" | "LTE" { Arity(2) }
        /// Conditional branching, aka `jump if equal`. It takes a register
        /// address as the argument and will jump to the value stored in that
        /// register IF the VM's `cmp` flag is set to `true`.
        JumpEq "jmpe" | "JMPE" { Arity(1) }
        JumpNeq "jmpne" | "JMPNE" { Arity(1) }
        /* IDK LOL */
        /// Halts the program
        Halt "halt" | "HALT" { Arity(0) }
        /// INVALID opcode; stops VM with an error
        Bad "bad" | "BAD" { Arity(1) }
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        OpCode::VARIANTS[byte as usize]
    }
}

#[test]
fn stringything() {
    let byte = 9u8;
    let u = OpCode::VARIANTS[byte as usize];
    assert_eq!(u.as_usize(), byte as usize)
}
