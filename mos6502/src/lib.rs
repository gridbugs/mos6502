mod instruction;
mod machine;

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
    pub const START_PC_LO: Address = 0xfffc;
    pub const START_PC_HI: Address = 0xfffd;
}
