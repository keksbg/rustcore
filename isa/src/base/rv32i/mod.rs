extern crate macros;
use macros::make_registers;

#[make_registers(u32, 32)]
struct RV32I {
    pub pc: u32,
}
