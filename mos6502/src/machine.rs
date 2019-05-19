use crate::addressing_mode::*;
use crate::instruction::*;
pub use crate::{address, status, Address};
use crate::{opcode, UnknownOpcode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cpu {
    pub pc: Address,
    pub sp: u8,
    pub acc: u8,
    pub x: u8,
    pub y: u8,
    pub status: StatusRegister,
}

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
    pub fn nmi<M: Memory>(&mut self, memory: &mut M) {
        self.push_stack_u8(memory, address::hi(self.pc));
        self.push_stack_u8(memory, address::lo(self.pc));
        self.push_stack_u8(memory, self.status.masked_with_brk_and_expansion());
        self.pc = memory.read_u16_le(crate::interrupt_vector::NMI_LO);
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
        self.pc = memory.read_u16_le(crate::interrupt_vector::START_LO);
    }
    pub fn run_for_cycles<M: Memory>(
        &mut self,
        memory: &mut M,
        num_cycles: usize,
    ) -> Result<usize, UnknownOpcode> {
        let mut cycle_count = 0;
        while cycle_count < num_cycles {
            cycle_count += self.step(memory)? as usize;
        }
        Ok(cycle_count)
    }
    pub fn step<M: Memory>(&mut self, memory: &mut M) -> Result<u8, UnknownOpcode> {
        let opcode = memory.read_u8(self.pc);
        let cycles = match opcode {
            opcode::adc::ABSOLUTE => adc::interpret(Absolute, self, memory),
            opcode::adc::ABSOLUTE_X_INDEXED => adc::interpret(AbsoluteXIndexed, self, memory),
            opcode::adc::ABSOLUTE_Y_INDEXED => adc::interpret(AbsoluteYIndexed, self, memory),
            opcode::adc::IMMEDIATE => adc::interpret(Immediate, self, memory),
            opcode::adc::INDIRECT_Y_INDEXED => adc::interpret(IndirectYIndexed, self, memory),
            opcode::adc::X_INDEXED_INDIRECT => adc::interpret(XIndexedIndirect, self, memory),
            opcode::adc::ZERO_PAGE => adc::interpret(ZeroPage, self, memory),
            opcode::adc::ZERO_PAGE_X_INDEXED => adc::interpret(ZeroPageXIndexed, self, memory),
            opcode::alr::unofficial0::IMMEDIATE => alr::interpret(self, memory),
            opcode::anc::unofficial0::IMMEDIATE => anc::interpret(self, memory),
            opcode::anc::unofficial1::IMMEDIATE => anc::interpret(self, memory),
            opcode::and::ABSOLUTE => and::interpret(Absolute, self, memory),
            opcode::and::ABSOLUTE_X_INDEXED => and::interpret(AbsoluteXIndexed, self, memory),
            opcode::and::ABSOLUTE_Y_INDEXED => and::interpret(AbsoluteYIndexed, self, memory),
            opcode::and::IMMEDIATE => and::interpret(Immediate, self, memory),
            opcode::and::INDIRECT_Y_INDEXED => and::interpret(IndirectYIndexed, self, memory),
            opcode::and::X_INDEXED_INDIRECT => and::interpret(XIndexedIndirect, self, memory),
            opcode::and::ZERO_PAGE => and::interpret(ZeroPage, self, memory),
            opcode::and::ZERO_PAGE_X_INDEXED => and::interpret(ZeroPageXIndexed, self, memory),
            opcode::asl::ABSOLUTE => asl::interpret(Absolute, self, memory),
            opcode::asl::ABSOLUTE_X_INDEXED => asl::interpret(AbsoluteXIndexed, self, memory),
            opcode::asl::ACCUMULATOR => asl::interpret_acc(self),
            opcode::asl::ZERO_PAGE => asl::interpret(ZeroPage, self, memory),
            opcode::asl::ZERO_PAGE_X_INDEXED => asl::interpret(ZeroPageXIndexed, self, memory),
            opcode::bcc::RELATIVE => bcc::interpret(self, memory),
            opcode::bcs::RELATIVE => bcs::interpret(self, memory),
            opcode::beq::RELATIVE => beq::interpret(self, memory),
            opcode::bmi::RELATIVE => bmi::interpret(self, memory),
            opcode::bne::RELATIVE => bne::interpret(self, memory),
            opcode::bpl::RELATIVE => bpl::interpret(self, memory),
            opcode::bvc::RELATIVE => bvc::interpret(self, memory),
            opcode::bvs::RELATIVE => bvs::interpret(self, memory),
            opcode::bit::ABSOLUTE => bit::interpret(Absolute, self, memory),
            opcode::bit::ZERO_PAGE => bit::interpret(ZeroPage, self, memory),
            opcode::brk::IMPLIED => brk::interpret(self, memory),
            opcode::clc::IMPLIED => clc::interpret(self),
            opcode::cld::IMPLIED => cld::interpret(self),
            opcode::cli::IMPLIED => cli::interpret(self),
            opcode::clv::IMPLIED => clv::interpret(self),
            opcode::cmp::ABSOLUTE => cmp::interpret(Absolute, self, memory),
            opcode::cmp::ABSOLUTE_X_INDEXED => cmp::interpret(AbsoluteXIndexed, self, memory),
            opcode::cmp::ABSOLUTE_Y_INDEXED => cmp::interpret(AbsoluteYIndexed, self, memory),
            opcode::cmp::IMMEDIATE => cmp::interpret(Immediate, self, memory),
            opcode::cmp::INDIRECT_Y_INDEXED => cmp::interpret(IndirectYIndexed, self, memory),
            opcode::cmp::X_INDEXED_INDIRECT => cmp::interpret(XIndexedIndirect, self, memory),
            opcode::cmp::ZERO_PAGE => cmp::interpret(ZeroPage, self, memory),
            opcode::cmp::ZERO_PAGE_X_INDEXED => cmp::interpret(ZeroPageXIndexed, self, memory),
            opcode::cpx::ABSOLUTE => cpx::interpret(Absolute, self, memory),
            opcode::cpx::IMMEDIATE => cpx::interpret(Immediate, self, memory),
            opcode::cpx::ZERO_PAGE => cpx::interpret(ZeroPage, self, memory),
            opcode::cpy::ABSOLUTE => cpy::interpret(Absolute, self, memory),
            opcode::cpy::IMMEDIATE => cpy::interpret(Immediate, self, memory),
            opcode::cpy::ZERO_PAGE => cpy::interpret(ZeroPage, self, memory),
            opcode::dec::ABSOLUTE => dec::interpret(Absolute, self, memory),
            opcode::dec::ABSOLUTE_X_INDEXED => dec::interpret(AbsoluteXIndexed, self, memory),
            opcode::dec::ZERO_PAGE => dec::interpret(ZeroPage, self, memory),
            opcode::dec::ZERO_PAGE_X_INDEXED => dec::interpret(ZeroPageXIndexed, self, memory),
            opcode::dex::IMPLIED => dex::interpret(self),
            opcode::dey::IMPLIED => dey::interpret(self),
            opcode::eor::ABSOLUTE => eor::interpret(Absolute, self, memory),
            opcode::eor::ABSOLUTE_X_INDEXED => eor::interpret(AbsoluteXIndexed, self, memory),
            opcode::eor::ABSOLUTE_Y_INDEXED => eor::interpret(AbsoluteYIndexed, self, memory),
            opcode::eor::IMMEDIATE => eor::interpret(Immediate, self, memory),
            opcode::eor::INDIRECT_Y_INDEXED => eor::interpret(IndirectYIndexed, self, memory),
            opcode::eor::X_INDEXED_INDIRECT => eor::interpret(XIndexedIndirect, self, memory),
            opcode::eor::ZERO_PAGE => eor::interpret(ZeroPage, self, memory),
            opcode::eor::ZERO_PAGE_X_INDEXED => eor::interpret(ZeroPageXIndexed, self, memory),
            opcode::inc::ABSOLUTE => inc::interpret(Absolute, self, memory),
            opcode::inc::ABSOLUTE_X_INDEXED => inc::interpret(AbsoluteXIndexed, self, memory),
            opcode::inc::ZERO_PAGE => inc::interpret(ZeroPage, self, memory),
            opcode::inc::ZERO_PAGE_X_INDEXED => inc::interpret(ZeroPageXIndexed, self, memory),
            opcode::inx::IMPLIED => inx::interpret(self),
            opcode::iny::IMPLIED => iny::interpret(self),
            opcode::jmp::ABSOLUTE => jmp::interpret(Absolute, self, memory),
            opcode::jmp::INDIRECT => jmp::interpret(Indirect, self, memory),
            opcode::jsr::ABSOLUTE => jsr::interpret(Absolute, self, memory),
            opcode::lda::ABSOLUTE => lda::interpret(Absolute, self, memory),
            opcode::lda::ABSOLUTE_X_INDEXED => lda::interpret(AbsoluteXIndexed, self, memory),
            opcode::lda::ABSOLUTE_Y_INDEXED => lda::interpret(AbsoluteYIndexed, self, memory),
            opcode::lda::IMMEDIATE => lda::interpret(Immediate, self, memory),
            opcode::lda::INDIRECT_Y_INDEXED => lda::interpret(IndirectYIndexed, self, memory),
            opcode::lda::X_INDEXED_INDIRECT => lda::interpret(XIndexedIndirect, self, memory),
            opcode::lda::ZERO_PAGE => lda::interpret(ZeroPage, self, memory),
            opcode::lda::ZERO_PAGE_X_INDEXED => lda::interpret(ZeroPageXIndexed, self, memory),
            opcode::ldx::ABSOLUTE => ldx::interpret(Absolute, self, memory),
            opcode::ldx::ABSOLUTE_Y_INDEXED => ldx::interpret(AbsoluteYIndexed, self, memory),
            opcode::ldx::IMMEDIATE => ldx::interpret(Immediate, self, memory),
            opcode::ldx::ZERO_PAGE => ldx::interpret(ZeroPage, self, memory),
            opcode::ldx::ZERO_PAGE_Y_INDEXED => ldx::interpret(ZeroPageYIndexed, self, memory),
            opcode::ldy::ABSOLUTE => ldy::interpret(Absolute, self, memory),
            opcode::ldy::ABSOLUTE_X_INDEXED => ldy::interpret(AbsoluteXIndexed, self, memory),
            opcode::ldy::IMMEDIATE => ldy::interpret(Immediate, self, memory),
            opcode::ldy::ZERO_PAGE => ldy::interpret(ZeroPage, self, memory),
            opcode::ldy::ZERO_PAGE_X_INDEXED => ldy::interpret(ZeroPageXIndexed, self, memory),
            opcode::lsr::ABSOLUTE => lsr::interpret(Absolute, self, memory),
            opcode::lsr::ABSOLUTE_X_INDEXED => lsr::interpret(AbsoluteXIndexed, self, memory),
            opcode::lsr::ACCUMULATOR => lsr::interpret_acc(self),
            opcode::lsr::ZERO_PAGE => lsr::interpret(ZeroPage, self, memory),
            opcode::lsr::ZERO_PAGE_X_INDEXED => lsr::interpret(ZeroPageXIndexed, self, memory),
            opcode::nop::IMPLIED => nop::interpret(self),
            opcode::nop::unofficial0::IMPLIED => nop::interpret(self),
            opcode::nop::unofficial1::IMPLIED => nop::interpret(self),
            opcode::nop::unofficial2::IMPLIED => nop::interpret(self),
            opcode::nop::unofficial3::IMPLIED => nop::interpret(self),
            opcode::nop::unofficial4::IMPLIED => nop::interpret(self),
            opcode::nop::unofficial5::IMPLIED => nop::interpret(self),
            opcode::ora::ABSOLUTE => ora::interpret(Absolute, self, memory),
            opcode::ora::ABSOLUTE_X_INDEXED => ora::interpret(AbsoluteXIndexed, self, memory),
            opcode::ora::ABSOLUTE_Y_INDEXED => ora::interpret(AbsoluteYIndexed, self, memory),
            opcode::ora::IMMEDIATE => ora::interpret(Immediate, self, memory),
            opcode::ora::INDIRECT_Y_INDEXED => ora::interpret(IndirectYIndexed, self, memory),
            opcode::ora::X_INDEXED_INDIRECT => ora::interpret(XIndexedIndirect, self, memory),
            opcode::ora::ZERO_PAGE => ora::interpret(ZeroPage, self, memory),
            opcode::ora::ZERO_PAGE_X_INDEXED => ora::interpret(ZeroPageXIndexed, self, memory),
            opcode::pha::IMPLIED => pha::interpret(self, memory),
            opcode::php::IMPLIED => php::interpret(self, memory),
            opcode::pla::IMPLIED => pla::interpret(self, memory),
            opcode::plp::IMPLIED => plp::interpret(self, memory),
            opcode::rol::ABSOLUTE => rol::interpret(Absolute, self, memory),
            opcode::rol::ABSOLUTE_X_INDEXED => rol::interpret(AbsoluteXIndexed, self, memory),
            opcode::rol::ACCUMULATOR => rol::interpret_acc(self),
            opcode::rol::ZERO_PAGE => rol::interpret(ZeroPage, self, memory),
            opcode::rol::ZERO_PAGE_X_INDEXED => rol::interpret(ZeroPageXIndexed, self, memory),
            opcode::ror::ABSOLUTE => ror::interpret(Absolute, self, memory),
            opcode::ror::ABSOLUTE_X_INDEXED => ror::interpret(AbsoluteXIndexed, self, memory),
            opcode::ror::ACCUMULATOR => ror::interpret_acc(self),
            opcode::ror::ZERO_PAGE => ror::interpret(ZeroPage, self, memory),
            opcode::ror::ZERO_PAGE_X_INDEXED => ror::interpret(ZeroPageXIndexed, self, memory),
            opcode::rti::IMPLIED => rti::interpret(self, memory),
            opcode::rts::IMPLIED => rts::interpret(self, memory),
            opcode::sbc::ABSOLUTE => sbc::interpret(Absolute, self, memory),
            opcode::sbc::ABSOLUTE_X_INDEXED => sbc::interpret(AbsoluteXIndexed, self, memory),
            opcode::sbc::ABSOLUTE_Y_INDEXED => sbc::interpret(AbsoluteYIndexed, self, memory),
            opcode::sbc::IMMEDIATE => sbc::interpret(Immediate, self, memory),
            opcode::sbc::INDIRECT_Y_INDEXED => sbc::interpret(IndirectYIndexed, self, memory),
            opcode::sbc::X_INDEXED_INDIRECT => sbc::interpret(XIndexedIndirect, self, memory),
            opcode::sbc::ZERO_PAGE => sbc::interpret(ZeroPage, self, memory),
            opcode::sbc::ZERO_PAGE_X_INDEXED => sbc::interpret(ZeroPageXIndexed, self, memory),
            opcode::sbc::unofficial0::IMMEDIATE => sbc::interpret(Immediate, self, memory),
            opcode::sec::IMPLIED => sec::interpret(self),
            opcode::sed::IMPLIED => sed::interpret(self),
            opcode::sei::IMPLIED => sei::interpret(self),
            opcode::skb::unofficial0::IMMEDIATE => skb::interpret(self, memory),
            opcode::skb::unofficial1::IMMEDIATE => skb::interpret(self, memory),
            opcode::skb::unofficial2::IMMEDIATE => skb::interpret(self, memory),
            opcode::skb::unofficial3::IMMEDIATE => skb::interpret(self, memory),
            opcode::skb::unofficial4::IMMEDIATE => skb::interpret(self, memory),
            opcode::sta::ABSOLUTE => sta::interpret(Absolute, self, memory),
            opcode::sta::ABSOLUTE_X_INDEXED => sta::interpret(AbsoluteXIndexed, self, memory),
            opcode::sta::ABSOLUTE_Y_INDEXED => sta::interpret(AbsoluteYIndexed, self, memory),
            opcode::sta::INDIRECT_Y_INDEXED => sta::interpret(IndirectYIndexed, self, memory),
            opcode::sta::X_INDEXED_INDIRECT => sta::interpret(XIndexedIndirect, self, memory),
            opcode::sta::ZERO_PAGE => sta::interpret(ZeroPage, self, memory),
            opcode::sta::ZERO_PAGE_X_INDEXED => sta::interpret(ZeroPageXIndexed, self, memory),
            opcode::stx::ABSOLUTE => stx::interpret(Absolute, self, memory),
            opcode::stx::ZERO_PAGE => stx::interpret(ZeroPage, self, memory),
            opcode::stx::ZERO_PAGE_Y_INDEXED => stx::interpret(ZeroPageYIndexed, self, memory),
            opcode::sty::ABSOLUTE => sty::interpret(Absolute, self, memory),
            opcode::sty::ZERO_PAGE => sty::interpret(ZeroPage, self, memory),
            opcode::sty::ZERO_PAGE_X_INDEXED => sty::interpret(ZeroPageXIndexed, self, memory),
            opcode::tax::IMPLIED => tax::interpret(self),
            opcode::tay::IMPLIED => tay::interpret(self),
            opcode::tsx::IMPLIED => tsx::interpret(self),
            opcode::txa::IMPLIED => txa::interpret(self),
            opcode::txs::IMPLIED => txs::interpret(self),
            opcode::tya::IMPLIED => tya::interpret(self),
            _ => return Err(UnknownOpcode(opcode)),
        };
        Ok(cycles)
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

/// View of memory which never changed by reading, for use in debugging and testing
pub trait MemoryReadOnly {
    fn read_u8_read_only(&self, address: Address) -> u8;
    fn read_u16_le_read_only(&self, address: Address) -> u16 {
        let lo = self.read_u8_read_only(address);
        let hi = self.read_u8_read_only(address + 1);
        ((hi as u16) << 8) | lo as u16
    }
}

pub use status::Register as StatusRegister;
