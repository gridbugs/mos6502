extern crate assembler;
extern crate mos6502;
use assembler::Block;
pub use mos6502::machine::{Address, Cpu, MemoryReadOnly};

#[cfg(test)]
pub mod test;
#[cfg(test)]
pub mod test_framework;

mod arithmetic;
mod infinite_loop;
mod jump_indirect;
mod load_accumulator_immediate;
mod load_and_store_all_addressing_modes;
mod stack_basic;
mod stack_status_register;
mod store_accumulator;
pub use arithmetic::*;
pub use infinite_loop::*;
pub use jump_indirect::*;
pub use load_accumulator_immediate::*;
pub use load_and_store_all_addressing_modes::*;
pub use stack_basic::*;
pub use stack_status_register::*;
pub use store_accumulator::*;

pub const PRG_START: Address = 0xC000;

pub trait Sample {
    fn program(block: &mut Block);
    fn num_steps() -> usize;
    fn check_result<M: MemoryReadOnly>(cpu: &Cpu, memory: &M);
}

pub(crate) mod prelude {
    pub use super::{Sample, PRG_START};
    pub use assembler::*;
    pub use mos6502::addressing_mode::*;
    pub use mos6502::assembler_instruction::*;
    pub use mos6502::machine::{status, Address, Cpu, MemoryReadOnly};
}
