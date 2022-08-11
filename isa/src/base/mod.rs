use macros::Instruction;

pub mod rv128i;
pub mod rv32i;
pub mod rv64i;

pub trait Instruction {
    fn from_u32(input: u32) -> Self;
}

// this would be nice:
// https://internals.rust-lang.org/t/pre-rfc-arbitrary-bit-width-integers/15603
// TODO: arbitrary-width integers

#[derive(Instruction)]
/// R-type instruction
///
/// Useful for register-register operations.
/// Sources from registers `rs1` and `rs2`.
/// Stores its output into register `rd`.
pub struct RInstruction {
    #[bits = 7]
    pub opcode: u8,
    /// Destination
    #[bits = 5]
    pub rd: u8,
    #[bits = 3]
    pub funct3: u8,
    /// Source
    #[bits = 5]
    pub rs1: u8,
    /// Source
    #[bits = 5]
    pub rs2: u8,
    #[bits = 7]
    pub funct7: u8,
}

#[derive(Instruction)]
/// I-type instruction
///
/// Useful for register-immediate operations.
/// Sources from register `rs1` and immediate `imm`.
/// Stores its output into register `rd`.
pub struct IInstruction {
    #[bits = 7]
    pub opcode: u8,
    /// Destination
    #[bits = 5]
    pub rd: u8,
    #[bits = 3]
    pub funct3: u8,
    /// Source
    #[bits = 5]
    pub rs1: u8,
    #[bits = 12]
    pub imm: u16,
}

#[derive(Instruction)]
/// S-type instruction
pub struct SInstruction {
    #[bits = 7]
    pub opcode: u8,
    #[bits = 5]
    pub imm0: u8,
    #[bits = 3]
    pub funct3: u8,
    /// Source
    #[bits = 5]
    pub rs1: u8,
    /// Source
    #[bits = 5]
    pub rs2: u8,
    #[bits = 12]
    pub imm1: u16,
}

#[derive(Instruction)]
/// B-type instruction
pub struct BInstruction {
    #[bits = 7]
    pub opcode: u8,
    #[bits = 2]
    pub imm0: u8,
    #[bits = 2]
    pub imm1: u8,
    #[bits = 3]
    pub funct3: u8,
    /// Source
    #[bits = 5]
    pub rs1: u8,
    /// Source
    #[bits = 5]
    pub rs2: u8,
    #[bits = 8]
    pub imm2: u8,
}

#[derive(Instruction)]
/// U-type instruction
pub struct UInstruction {
    #[bits = 7]
    pub opcode: u8,
    /// Destination
    #[bits = 5]
    pub rd: u8,
    #[bits = 20]
    pub imm: u32,
}

#[derive(Instruction)]
/// J-type instruction
pub struct JInstruction {
    #[bits = 7]
    pub opcode: u8,
    /// Destination
    #[bits = 5]
    pub rd: u8,
    #[bits = 8]
    pub imm0: u8,
    #[bits = 1]
    pub imm1: u8,
    #[bits = 10]
    pub imm2: u16,
    #[bits = 1]
    pub imm3: u8,
}
