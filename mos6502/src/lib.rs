pub mod addressing_mode;
pub mod assembler_instruction;
pub mod debug;
pub mod instruction;
pub mod machine;
pub mod opcode;
pub mod operand;

pub use addressing_mode::Trait as AddressingMode;
pub use assembler_instruction::Trait as AssemblerInstruction;

use std::fmt;

pub type Address = u16;

#[derive(Clone, Copy)]
pub struct UnknownOpcode(pub u8);

impl fmt::Debug for UnknownOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UnknownOpcode({:02X})", self.0)
    }
}

pub mod address {
    use super::*;
    pub fn lo(address: Address) -> u8 {
        address as u8
    }
    pub fn hi(address: Address) -> u8 {
        address.wrapping_shr(8) as u8
    }
    pub fn from_u8_lo_hi(lo: u8, hi: u8) -> Address {
        (hi as Address).wrapping_shl(8) | lo as Address
    }
}

pub mod interrupt_vector {
    use super::*;
    pub const NMI_LO: Address = 0xFFFA;
    pub const NMI_HI: Address = 0xFFFB;
    pub const START_LO: Address = 0xFFFC;
    pub const START_HI: Address = 0xFFFD;
    pub const IRQ_LO: Address = 0xFFFE;
    pub const IRQ_HI: Address = 0xFFFF;
}
