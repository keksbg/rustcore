use macros::Instruction;

pub mod rv128i;
pub mod rv32i;
pub mod rv64i;

pub trait Instruction {}

// this would be nice:
// https://internals.rust-lang.org/t/pre-rfc-arbitrary-bit-width-integers/15603
// TODO: arbitrary-width integers instead of boolean arrays
#[derive(Instruction)]
#[repr(align(32))]
/// R-type instruction
///
/// Useful for register-register operations.
/// Sources from registers `rs1` and `rs2`.
/// Stores its output into register `rd`
pub struct RInstruction {
    pub opcode: [bool; 7],
    /// Destination
    pub rd: [bool; 5],
    pub funct3: [bool; 3],
    /// Source
    pub rs1: [bool; 5],
    /// Source
    pub rs2: [bool; 5],
    pub funct7: [bool; 7],
}

#[derive(Instruction)]
#[repr(align(32))]
/// I-type instruction
///
/// Useful for register-immediate operations.
/// Sources from register `rs1` and immediate `imm`.
/// Stores its output into register `rd`.
pub struct IInstruction {
    pub opcode: [bool; 7],
    /// Destination
    pub rd: [bool; 5],
    pub funct3: [bool; 3],
    /// Source
    pub rs1: [bool; 5],
    pub imm: [bool; 12],
}

#[derive(Instruction)]
#[repr(align(32))]
/// S-type instruction
pub struct SInstruction {
    pub opcode: [bool; 7],
    pub imm0: [bool; 5],
    pub funct3: [bool; 3],
    /// Source
    pub rs1: [bool; 5],
    /// Source
    pub rs2: [bool; 5],
    pub imm1: [bool; 12],
}

#[derive(Instruction)]
#[repr(align(32))]
/// B-type instruction
pub struct BInstruction {
    pub opcode: [bool; 7],
    pub imm0: [bool; 2],
    pub imm1: [bool; 2],
    pub funct3: [bool; 3],
    /// Source
    pub rs1: [bool; 5],
    /// Source
    pub rs2: [bool; 5],
    pub imm2: [bool; 8],
}

#[derive(Instruction)]
#[repr(align(32))]
/// U-type instruction
pub struct UInstruction {
    pub opcode: [bool; 7],
    /// Destination
    pub rd: [bool; 5],
    pub imm: [bool; 20],
}

#[derive(Instruction)]
#[repr(align(32))]
/// J-type instruction
pub struct JInstruction {
    pub opcode: [bool; 7],
    /// Destination
    pub rd: [bool; 5],
    pub imm0: [bool; 8],
    pub imm1: [bool; 1],
    pub imm2: [bool; 10],
    pub imm3: [bool; 1],
}
