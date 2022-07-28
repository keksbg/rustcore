extern crate macros;
use macros::{make_registers, Instruction};

#[make_registers(u32, 32)]
struct RV32I {
    pub pc: u32,
}

#[derive(Instruction)]
struct RInstruction;
