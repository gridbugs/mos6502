use crate::Address;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub pc: Address,
    pub acc: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct UnknownOpcode(u8);

impl Cpu {
    pub fn new() -> Self {
        Self { pc: 0, acc: 0 }
    }
    pub fn start<D: MemoryMappedDevices>(&mut self, devices: &mut D) {
        self.pc = devices.read_u16_le(crate::interrupt_vector::START_PC_LO);
    }
    pub fn step<D: MemoryMappedDevices>(&mut self, devices: &mut D) -> Result<(), UnknownOpcode> {
        let opcode = devices.read_u8(self.pc);
        use crate::instruction::{addressing_mode::*, *};
        match opcode {
            opcode::jmp::ABSOLUTE => <Jmp<Absolute> as Instruction>::interpret(self, devices),
            opcode::jmp::INDIRECT => <Jmp<Indirect> as Instruction>::interpret(self, devices),
            opcode::lda::IMMEDIATE => <Lda<Immediate> as Instruction>::interpret(self, devices),
            _ => return Err(UnknownOpcode(opcode)),
        }
        Ok(())
    }
}

pub trait MemoryMappedDevices {
    fn read_u8(&mut self, address: Address) -> u8;
    fn read_u16_le(&mut self, address: Address) -> u16 {
        let lo = self.read_u8(address);
        let hi = self.read_u8(address + 1);
        ((hi as u16) << 8) | lo as u16
    }
    fn write_u8(&mut self, address: Address, data: u8);
}
