pub mod instruction;
mod machine;

pub use machine::*;

pub type Address = u16;
pub trait Operand {}

pub mod operand {
    use super::*;
    pub enum Address {}
    impl Operand for Address {}
    pub enum Byte {}
    impl Operand for Byte {}
}

pub trait Instruction {
    type Operand: Operand;
    fn num_bytes(&self) -> usize;
    fn opcode(&self) -> u8;
    fn num_cycles(&self) -> usize;
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
    pub const START_PC_LO: Address = 0xfffc;
    pub const START_PC_HI: Address = 0xfffd;
}
