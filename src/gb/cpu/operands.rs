/// 8-bit register identifiers, matching Game Boy opcode encoding
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Reg8 {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    HLRef = 6, // (HL) - memory reference
    A = 7,
}

impl Reg8 {
    /// Decode 3-bit register field from opcode
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x07 {
            0 => Reg8::B,
            1 => Reg8::C,
            2 => Reg8::D,
            3 => Reg8::E,
            4 => Reg8::H,
            5 => Reg8::L,
            6 => Reg8::HLRef,
            7 => Reg8::A,
            _ => unreachable!(),
        }
    }
}

/// 16-bit register identifiers
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reg16 {
    BC,
    DE,
    HL,
    SP,
}

impl Reg16 {
    /// Decode 2-bit register pair field from opcode (for LD rr,d16, INC/DEC rr, etc.)
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Reg16::BC,
            1 => Reg16::DE,
            2 => Reg16::HL,
            3 => Reg16::SP,
            _ => unreachable!(),
        }
    }
}

/// 16-bit register identifiers for PUSH/POP (uses AF instead of SP)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reg16Stack {
    BC,
    DE,
    HL,
    AF,
}

impl Reg16Stack {
    /// Decode 2-bit register pair field from opcode for stack operations
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Reg16Stack::BC,
            1 => Reg16Stack::DE,
            2 => Reg16Stack::HL,
            3 => Reg16Stack::AF,
            _ => unreachable!(),
        }
    }
}

/// Condition codes for conditional jumps/calls/returns
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ConditionCode {
    NZ, // Not Zero
    Z,  // Zero
    NC, // Not Carry
    C,  // Carry
}

impl ConditionCode {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => ConditionCode::NZ,
            1 => ConditionCode::Z,
            2 => ConditionCode::NC,
            3 => ConditionCode::C,
            _ => unreachable!(),
        }
    }
}
