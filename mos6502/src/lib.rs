pub mod addressing_mode;
pub mod assembler_instruction;
pub mod debug;
mod instruction;
mod machine;
pub mod opcode;
pub mod operand;

pub use addressing_mode::Trait as AddressingMode;
pub use assembler_instruction::Trait as AssemblerInstruction;
pub use instruction::*;
pub use machine::*;

pub type Address = u16;

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
