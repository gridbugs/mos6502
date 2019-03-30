extern crate mos6502;

use mos6502::*;

const ADDRESSABLE_BYTES: usize = (1 << 16);

pub struct Assembler {
    cursor: Address,
    binary: [u8; ADDRESSABLE_BYTES],
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            binary: [0; ADDRESSABLE_BYTES],
        }
    }
    pub fn partial_binary(&self, start: Address, size: usize, buffer: &mut Vec<u8>) {
        buffer.resize(size, 0);
        buffer.copy_from_slice(&self.binary[(start as usize)..(start as usize + size)]);
    }
    pub fn org(&mut self, address: Address) {
        self.cursor = address;
    }
    pub fn declare_pc_address(&mut self, address: Address) {
        self.binary[START_PC_LO as usize] = (address & 0xff) as u8;
        self.binary[START_PC_HI as usize] = (address >> 8) as u8;
    }
}
