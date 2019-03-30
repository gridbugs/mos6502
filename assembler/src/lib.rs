extern crate mos6502;

use mos6502::*;
use std::collections::HashMap;

enum Data {
    LiteralByte(u8),
    LabelOffsetLe(String),
    LiteralOffsetLe(Address),
}

struct DataAtOffset {
    data: Data,
    offset: Address,
}

pub struct Block {
    cursor_offset: Address,
    program: Vec<DataAtOffset>,
    labels: HashMap<String, Address>,
}

pub enum AddressOperand {
    LabelOffset(&'static str),
    LiteralOffset(Address),
}

pub trait ArgOperand {
    type Operand: Operand;
    fn program(self, block: &mut Block);
}

impl ArgOperand for AddressOperand {
    type Operand = operand::Address;
    fn program(self, block: &mut Block) {
        block.address_operand(self);
    }
}

impl ArgOperand for u8 {
    type Operand = operand::Byte;
    fn program(self, block: &mut Block) {
        block.literal_byte(self);
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    OffsetOutOfBounds,
    UndeclaredLabel(String),
}

impl Block {
    pub fn new() -> Self {
        Self {
            cursor_offset: 0,
            program: Vec::new(),
            labels: HashMap::new(),
        }
    }
    pub fn set_offset(&mut self, offset: Address) {
        self.cursor_offset = offset;
    }
    pub fn literal_byte(&mut self, byte: u8) {
        self.program.push(DataAtOffset {
            data: Data::LiteralByte(byte),
            offset: self.cursor_offset,
        });
        self.cursor_offset += 1;
    }
    pub fn literal_offset_le(&mut self, offset: Address) {
        self.program.push(DataAtOffset {
            data: Data::LiteralOffsetLe(offset),
            offset: self.cursor_offset,
        });
        self.cursor_offset += 2;
    }
    pub fn label_offset_le<S: AsRef<str>>(&mut self, label: S) {
        let string = label.as_ref().to_string();
        self.program.push(DataAtOffset {
            data: Data::LabelOffsetLe(string),
            offset: self.cursor_offset,
        });
        self.cursor_offset += 2;
    }
    pub fn label<S: AsRef<str>>(&mut self, s: S) {
        let string = s.as_ref().to_string();
        self.labels.insert(string, self.cursor_offset);
    }
    pub fn address_operand(&mut self, operand: AddressOperand) {
        match operand {
            AddressOperand::LabelOffset(label) => {
                self.label_offset_le(label);
            }
            AddressOperand::LiteralOffset(offset) => {
                self.literal_offset_le(offset);
            }
        }
    }
    pub fn inst<I: Instruction, A: ArgOperand<Operand = I::Operand>>(&mut self, inst: I, arg: A) {
        self.literal_byte(inst.opcode());
        arg.program(self);
    }
    pub fn assemble(&self, base: Address, size: usize, buffer: &mut Vec<u8>) -> Result<(), Error> {
        buffer.resize(size, 0);
        for &DataAtOffset { offset, ref data } in self.program.iter() {
            match data {
                &Data::LiteralByte(byte) => {
                    if offset as usize >= size {
                        return Err(Error::OffsetOutOfBounds);
                    }
                    buffer[offset as usize] = byte;
                }
                Data::LabelOffsetLe(label) => {
                    if let Some(&label_offset) = self.labels.get(label) {
                        if offset as usize + 1 >= size {
                            return Err(Error::OffsetOutOfBounds);
                        }
                        let address = label_offset + base;
                        buffer[offset as usize] = address::lo(address);
                        buffer[offset as usize + 1] = address::hi(address);
                    } else {
                        return Err(Error::UndeclaredLabel(label.clone()));
                    }
                }
                Data::LiteralOffsetLe(literal_offset) => {
                    if offset as usize + 1 >= size {
                        return Err(Error::OffsetOutOfBounds);
                    }
                    let address = literal_offset + base;
                    buffer[offset as usize] = address::lo(address);
                    buffer[offset as usize + 1] = address::hi(address);
                }
            }
        }
        Ok(())
    }
}
