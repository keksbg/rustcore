extern crate macros;
use macros::make_registers;

#[make_registers(u32, 32)]
/// RV32I Base Integer Instruction Set
///
/// x0: hardwired to 0;
/// x1: return address;
/// x2: stack pointer;
/// x5: link register
struct RV32I {
    pub pc: u32,
}
