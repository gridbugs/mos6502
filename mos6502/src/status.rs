pub mod bit {
    pub const CARRY: u8 = 0;
    pub const ZERO: u8 = 1;
    pub const INTERRUPT_DISABLE: u8 = 2;
    pub const DECIMAL: u8 = 3;
    pub const BRK: u8 = 4;
    pub const EXPANSION: u8 = 5;
    pub const OVERFLOW: u8 = 6;
    pub const NEGATIVE: u8 = 7;
}
pub mod flag {
    use super::bit;
    pub const CARRY: u8 = 1 << bit::CARRY;
    pub const ZERO: u8 = 1 << bit::ZERO;
    pub const INTERRUPT_DISABLE: u8 = 1 << bit::INTERRUPT_DISABLE;
    pub const DECIMAL: u8 = 1 << bit::DECIMAL;
    pub const BRK: u8 = 1 << bit::BRK;
    pub const EXPANSION: u8 = 1 << bit::EXPANSION;
    pub const OVERFLOW: u8 = 1 << bit::OVERFLOW;
    pub const NEGATIVE: u8 = 1 << bit::NEGATIVE;
}
const MASK: u8 = !(flag::BRK | flag::EXPANSION);
#[derive(Clone, Serialize, Deserialize)]
pub struct Register {
    raw: u8,
}
impl Register {
    pub fn new() -> Self {
        Self {
            raw: flag::INTERRUPT_DISABLE,
        }
    }
    pub fn masked_with_brk_and_expansion(&self) -> u8 {
        self.raw | flag::BRK | flag::EXPANSION
    }
    pub fn set(&mut self, value: u8) {
        self.raw = value & MASK;
    }
    pub fn set_carry(&mut self) {
        self.raw |= flag::CARRY;
    }
    pub fn clear_carry(&mut self) {
        self.raw &= !flag::CARRY;
    }
    pub fn set_carry_to(&mut self, value: bool) {
        self.raw = ((value as u8) << bit::CARRY) | (self.raw & !flag::CARRY);
    }
    pub fn is_carry(&self) -> bool {
        self.raw & flag::CARRY != 0
    }
    pub fn carry_value(&self) -> u8 {
        (self.raw & flag::CARRY) >> bit::CARRY
    }
    pub fn set_decimal(&mut self) {
        self.raw |= flag::DECIMAL;
    }
    pub fn clear_decimal(&mut self) {
        self.raw &= !flag::DECIMAL;
    }
    pub fn is_decimal(&self) -> bool {
        self.raw & flag::DECIMAL != 0
    }
    pub fn set_zero_from_value(&mut self, value: u8) {
        self.raw = (((value == 0) as u8) << bit::ZERO) | (self.raw & !flag::ZERO);
    }
    pub fn is_zero(&self) -> bool {
        self.raw & flag::ZERO != 0
    }
    pub fn clear_overflow(&mut self) {
        self.raw &= !flag::OVERFLOW;
    }
    pub fn is_overflow(&self) -> bool {
        self.raw & flag::OVERFLOW != 0
    }
    pub fn set_overflow_to(&mut self, value: bool) {
        self.raw = ((value as u8) << bit::OVERFLOW) | (self.raw & !flag::OVERFLOW);
    }
    pub fn clear_negative(&mut self) {
        self.raw &= !flag::NEGATIVE;
    }
    pub fn is_negative(&self) -> bool {
        self.raw & flag::NEGATIVE != 0
    }
    pub fn set_negative_from_value(&mut self, value: u8) {
        self.raw = (value & flag::NEGATIVE) | (self.raw & !flag::NEGATIVE);
    }
    pub fn set_interrupt_disable(&mut self) {
        self.raw |= flag::INTERRUPT_DISABLE;
    }
    pub fn clear_interrupt_disable(&mut self) {
        self.raw &= !flag::INTERRUPT_DISABLE;
    }
    pub fn is_interrupt_disable(&self) -> bool {
        self.raw & flag::INTERRUPT_DISABLE != 0
    }
}
use std::fmt;
impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[N={:?},V={:?},D={:?},I:{:?},Z:{:?},C:{:?}]",
            self.is_negative() as u8,
            self.is_overflow() as u8,
            self.is_decimal() as u8,
            self.is_interrupt_disable() as u8,
            self.is_zero() as u8,
            self.is_carry() as u8,
        )
    }
}
