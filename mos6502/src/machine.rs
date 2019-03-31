use crate::Address;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub pc: Address,
    pub acc: u8,
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct UnknownOpcode(u8);

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: 0,
            acc: 0,
            x: 0,
            y: 0,
        }
    }
    pub fn start<M: Memory>(&mut self, memory: &mut M) {
        self.pc = memory.read_u16_le(crate::interrupt_vector::START_PC_LO);
    }
    pub fn step<M: Memory>(&mut self, memory: &mut M) -> Result<(), UnknownOpcode> {
        let opcode = memory.read_u8(self.pc);
        use crate::instruction::{addressing_mode::*, instruction::*, opcode};
        match opcode {
            opcode::jmp::ABSOLUTE => jmp::interpret(Absolute, self, memory),
            opcode::jmp::INDIRECT => jmp::interpret(Indirect, self, memory),
            opcode::lda::IMMEDIATE => lda::interpret(Immediate, self, memory),
            _ => return Err(UnknownOpcode(opcode)),
        }
        Ok(())
    }
}

pub trait Memory {
    fn read_u8(&mut self, address: Address) -> u8;
    fn read_u16_le(&mut self, address: Address) -> u16 {
        let lo = self.read_u8(address);
        let hi = self.read_u8(address + 1);
        ((hi as u16) << 8) | lo as u16
    }
    fn write_u8(&mut self, address: Address, data: u8);
}
