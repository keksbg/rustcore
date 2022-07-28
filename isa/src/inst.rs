// Instruction definitions

/// Length (in bits) of one parcel. This is the "unit" length
/// for all instructions, mostly useful for alternative ISAs with
/// differing lengths from the official ISA. Naturally, this is
/// also the boundary for alignment.
const PARCEL: u8 = 16;
/// Length (in bits) of the boundary for aligning instruction
/// addresses. In the base ISA this is always 32 bits, though in
/// other ISA extensions (like the compressed ISA), this may be
/// relaxed to 16 bits. No other values are permitted.
const IALIGN: u8 = PARCEL * 2;
/// Maximum length (in bits) of any one instruction. For our
/// implementation (which only has the base ISA) this is fixed
/// at 32 bits. This must be a multiple of `IALIGN`.
const ILEN: u8 = IALIGN * 1;
