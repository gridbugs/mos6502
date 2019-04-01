use crate::Address;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub pc: Address,
    pub sp: u8,
    pub acc: u8,
    pub x: u8,
    pub y: u8,
    pub status: StatusRegister,
}

#[derive(Debug, Clone, Copy)]
pub struct UnknownOpcode(pub u8);

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0xff,
            acc: 0,
            x: 0,
            y: 0,
            status: StatusRegister::new(),
        }
    }
    pub fn sp_address(&self) -> Address {
        (0x01 << 8) | (self.sp as Address)
    }
    pub fn push_stack_u8<M: Memory>(&mut self, memory: &mut M, value: u8) {
        let address = self.sp_address();
        memory.write_u8(address, value);
        self.sp = self.sp.wrapping_sub(1);
    }
    pub fn pop_stack_u8<M: Memory>(&mut self, memory: &mut M) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let address = self.sp_address();
        memory.read_u8(address)
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
            opcode::ldx::IMMEDIATE => ldx::interpret(Immediate, self, memory),
            opcode::ldy::IMMEDIATE => ldy::interpret(Immediate, self, memory),
            opcode::lda::IMMEDIATE => lda::interpret(Immediate, self, memory),
            opcode::lda::ZERO_PAGE => lda::interpret(ZeroPage, self, memory),
            opcode::lda::ZERO_PAGE_X_INDEXED => lda::interpret(ZeroPageXIndexed, self, memory),
            opcode::lda::ABSOLUTE => lda::interpret(Absolute, self, memory),
            opcode::lda::ABSOLUTE_X_INDEXED => lda::interpret(AbsoluteXIndexed, self, memory),
            opcode::lda::ABSOLUTE_Y_INDEXED => lda::interpret(AbsoluteYIndexed, self, memory),
            opcode::lda::X_INDEXED_INDIRECT => lda::interpret(XIndexedIndirect, self, memory),
            opcode::lda::INDIRECT_Y_INDEXED => lda::interpret(IndirectYIndexed, self, memory),
            opcode::sta::ZERO_PAGE => sta::interpret(ZeroPage, self, memory),
            opcode::sta::ZERO_PAGE_X_INDEXED => sta::interpret(ZeroPageXIndexed, self, memory),
            opcode::sta::ABSOLUTE => sta::interpret(Absolute, self, memory),
            opcode::sta::ABSOLUTE_X_INDEXED => sta::interpret(AbsoluteXIndexed, self, memory),
            opcode::sta::ABSOLUTE_Y_INDEXED => sta::interpret(AbsoluteYIndexed, self, memory),
            opcode::sta::X_INDEXED_INDIRECT => sta::interpret(XIndexedIndirect, self, memory),
            opcode::sta::INDIRECT_Y_INDEXED => sta::interpret(IndirectYIndexed, self, memory),
            opcode::php::IMPLIED => php::interpret(self, memory),
            opcode::plp::IMPLIED => plp::interpret(self, memory),
            opcode::sec::IMPLIED => sec::interpret(self),
            opcode::clc::IMPLIED => clc::interpret(self),
            opcode::sed::IMPLIED => sed::interpret(self),
            opcode::cld::IMPLIED => cld::interpret(self),
            opcode::sei::IMPLIED => sei::interpret(self),
            opcode::cli::IMPLIED => cli::interpret(self),
            opcode::pha::IMPLIED => pha::interpret(self, memory),
            opcode::pla::IMPLIED => pla::interpret(self, memory),
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

#[derive(Clone)]
pub struct StatusRegister {
    pub raw: u8,
}

const STATUS_CARRY: u8 = 1 << 0;
const STATUS_ZERO_BIT: u8 = 1;
const STATUS_ZERO: u8 = 1 << STATUS_ZERO_BIT;
const STATUS_INTERRUPT_DISABLE: u8 = 1 << 2;
const STATUS_DECIMAL: u8 = 1 << 3;
const STATUS_BRK: u8 = 1 << 4;
const STATUS_EXPANSION: u8 = 1 << 5;
const STATUS_OVERFLOW: u8 = 1 << 6;
const STATUS_NEGATIVE: u8 = 1 << 7;

impl StatusRegister {
    pub fn new() -> Self {
        Self {
            raw: STATUS_EXPANSION,
        }
    }
    pub fn set_carry(&mut self) {
        self.raw |= STATUS_CARRY;
    }
    pub fn clear_carry(&mut self) {
        self.raw &= !STATUS_CARRY;
    }
    pub fn is_carry(&self) -> bool {
        self.raw & STATUS_CARRY != 0
    }
    pub fn set_decimal(&mut self) {
        self.raw |= STATUS_DECIMAL;
    }
    pub fn clear_decimal(&mut self) {
        self.raw &= !STATUS_DECIMAL;
    }
    pub fn is_decimal(&self) -> bool {
        self.raw & STATUS_CARRY != 0
    }
    pub fn set_zero(&mut self) {
        self.raw |= STATUS_ZERO;
    }
    pub fn set_zero_from_value(&mut self, value: u8) {
        self.raw = (((value == 0) as u8) << STATUS_ZERO_BIT) | (self.raw & !STATUS_ZERO);
    }
    pub fn clear_zero(&mut self) {
        self.raw &= !STATUS_ZERO;
    }
    pub fn is_zero(&self) -> bool {
        self.raw & STATUS_ZERO != 0
    }
    pub fn set_brk(&mut self) {
        self.raw |= STATUS_BRK;
    }
    pub fn clear_brk(&mut self) {
        self.raw &= !STATUS_BRK;
    }
    pub fn is_brk(&self) -> bool {
        self.raw & STATUS_BRK != 0
    }
    pub fn set_overflow(&mut self) {
        self.raw |= STATUS_OVERFLOW;
    }
    pub fn clear_overflow(&mut self) {
        self.raw &= !STATUS_OVERFLOW;
    }
    pub fn is_overflow(&self) -> bool {
        self.raw & STATUS_OVERFLOW != 0
    }
    pub fn set_negative(&mut self) {
        self.raw |= STATUS_NEGATIVE;
    }
    pub fn clear_negative(&mut self) {
        self.raw &= !STATUS_NEGATIVE;
    }
    pub fn is_negative(&self) -> bool {
        self.raw & STATUS_NEGATIVE != 0
    }
    pub fn set_negative_from_value(&mut self, value: u8) {
        self.raw = (value & STATUS_NEGATIVE) | (self.raw & !STATUS_NEGATIVE);
    }
    pub fn set_interrupt_disable(&mut self) {
        self.raw |= STATUS_INTERRUPT_DISABLE;
    }
    pub fn clear_interrupt_disable(&mut self) {
        self.raw &= !STATUS_INTERRUPT_DISABLE;
    }
    pub fn is_interrupt_disable(&self) -> bool {
        self.raw & STATUS_INTERRUPT_DISABLE != 0
    }
    pub fn is_expansion(&self) -> bool {
        self.raw & STATUS_EXPANSION != 0
    }
}

impl fmt::Debug for StatusRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[N={:?},V={:?},E={:?},B={:?},D={:?},I:{:?},Z:{:?},C:{:?}]",
            self.is_negative() as u8,
            self.is_overflow() as u8,
            self.is_expansion() as u8,
            self.is_brk() as u8,
            self.is_decimal() as u8,
            self.is_interrupt_disable() as u8,
            self.is_zero() as u8,
            self.is_carry() as u8,
        )
    }
}
