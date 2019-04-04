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
        (address >> 8) as u8
    }
}

pub mod interrupt_vector {
    use super::*;
    pub const START_PC_LO: Address = 0xFFFC;
    pub const START_PC_HI: Address = 0xFFFD;
}
