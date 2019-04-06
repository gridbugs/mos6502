use crate::machine::{Cpu, MemoryReadOnly};
use crate::{Address, UnknownOpcode};
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum InstructionType {
    Adc,
    And,
    Asl,
    Bcc,
    Bcs,
    Beq,
    Bmi,
    Bne,
    Bpl,
    Bvc,
    Bvs,
    Bit,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rol,
    Ror,
    Rts,
    Sbc,
    Sec,
    Sed,
    Sei,
    Sta,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
}
#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Absolute,
    AbsoluteXIndexed,
    AbsoluteYIndexed,
    Accumulator,
    Implied,
    Immediate,
    Indirect,
    IndirectYIndexed,
    Relative,
    XIndexedIndirect,
    ZeroPage,
    ZeroPageXIndexed,
    ZeroPageYIndexed,
}
impl AddressingMode {
    fn operand_bytes(self) -> usize {
        use AddressingMode::*;
        match self {
            Absolute => 2,
            AbsoluteXIndexed => 2,
            AbsoluteYIndexed => 2,
            Implied => 0,
            Accumulator => 0,
            Immediate => 1,
            Indirect => 2,
            IndirectYIndexed => 1,
            Relative => 1,
            XIndexedIndirect => 1,
            ZeroPage => 1,
            ZeroPageXIndexed => 1,
            ZeroPageYIndexed => 1,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    instruction_type: InstructionType,
    addressing_mode: AddressingMode,
}

impl Instruction {
    fn new(instruction_type: InstructionType, addressing_mode: AddressingMode) -> Self {
        Self {
            instruction_type,
            addressing_mode,
        }
    }
    pub fn from_opcode(opcode: u8) -> Result<Self, UnknownOpcode> {
        use crate::opcode;
        use AddressingMode::*;
        use InstructionType::*;
        let (instruction_type, addressing_mode) = match opcode {
            opcode::adc::ABSOLUTE => (Adc, Absolute),
            opcode::adc::ABSOLUTE_X_INDEXED => (Adc, AbsoluteXIndexed),
            opcode::adc::ABSOLUTE_Y_INDEXED => (Adc, AbsoluteYIndexed),
            opcode::adc::IMMEDIATE => (Adc, Immediate),
            opcode::adc::INDIRECT_Y_INDEXED => (Adc, IndirectYIndexed),
            opcode::adc::X_INDEXED_INDIRECT => (Adc, XIndexedIndirect),
            opcode::adc::ZERO_PAGE => (Adc, ZeroPage),
            opcode::adc::ZERO_PAGE_X_INDEXED => (Adc, ZeroPageXIndexed),
            opcode::and::ABSOLUTE => (And, Absolute),
            opcode::and::ABSOLUTE_X_INDEXED => (And, AbsoluteXIndexed),
            opcode::and::ABSOLUTE_Y_INDEXED => (And, AbsoluteYIndexed),
            opcode::and::IMMEDIATE => (And, Immediate),
            opcode::and::INDIRECT_Y_INDEXED => (And, IndirectYIndexed),
            opcode::and::X_INDEXED_INDIRECT => (And, XIndexedIndirect),
            opcode::and::ZERO_PAGE => (And, ZeroPage),
            opcode::and::ZERO_PAGE_X_INDEXED => (And, ZeroPageXIndexed),
            opcode::asl::ABSOLUTE => (Asl, Absolute),
            opcode::asl::ABSOLUTE_X_INDEXED => (Asl, AbsoluteXIndexed),
            opcode::asl::ACCUMULATOR => (Asl, Accumulator),
            opcode::asl::ZERO_PAGE => (Asl, ZeroPage),
            opcode::asl::ZERO_PAGE_X_INDEXED => (Asl, ZeroPageXIndexed),
            opcode::bcc::RELATIVE => (Bcc, Relative),
            opcode::bcs::RELATIVE => (Bcs, Relative),
            opcode::beq::RELATIVE => (Beq, Relative),
            opcode::bmi::RELATIVE => (Bmi, Relative),
            opcode::bne::RELATIVE => (Bne, Relative),
            opcode::bpl::RELATIVE => (Bpl, Relative),
            opcode::bvc::RELATIVE => (Bvc, Relative),
            opcode::bvs::RELATIVE => (Bvs, Relative),
            opcode::bit::ABSOLUTE => (Bit, Absolute),
            opcode::bit::ZERO_PAGE => (Bit, ZeroPage),
            opcode::clc::IMPLIED => (Clc, Implied),
            opcode::cld::IMPLIED => (Cld, Implied),
            opcode::cli::IMPLIED => (Cli, Implied),
            opcode::clv::IMPLIED => (Clv, Implied),
            opcode::cmp::ABSOLUTE => (Cmp, Absolute),
            opcode::cmp::ABSOLUTE_X_INDEXED => (Cmp, AbsoluteXIndexed),
            opcode::cmp::ABSOLUTE_Y_INDEXED => (Cmp, AbsoluteYIndexed),
            opcode::cmp::IMMEDIATE => (Cmp, Immediate),
            opcode::cmp::INDIRECT_Y_INDEXED => (Cmp, IndirectYIndexed),
            opcode::cmp::X_INDEXED_INDIRECT => (Cmp, XIndexedIndirect),
            opcode::cmp::ZERO_PAGE => (Cmp, ZeroPage),
            opcode::cmp::ZERO_PAGE_X_INDEXED => (Cmp, ZeroPageXIndexed),
            opcode::dec::ABSOLUTE => (Dec, Absolute),
            opcode::dec::ABSOLUTE_X_INDEXED => (Dec, AbsoluteXIndexed),
            opcode::dec::ZERO_PAGE => (Dec, ZeroPage),
            opcode::dec::ZERO_PAGE_X_INDEXED => (Dec, ZeroPageXIndexed),
            opcode::dex::IMPLIED => (Dex, Implied),
            opcode::dey::IMPLIED => (Dey, Implied),
            opcode::eor::ABSOLUTE => (Eor, Absolute),
            opcode::eor::ABSOLUTE_X_INDEXED => (Eor, AbsoluteXIndexed),
            opcode::eor::ABSOLUTE_Y_INDEXED => (Eor, AbsoluteYIndexed),
            opcode::eor::IMMEDIATE => (Eor, Immediate),
            opcode::eor::INDIRECT_Y_INDEXED => (Eor, IndirectYIndexed),
            opcode::eor::X_INDEXED_INDIRECT => (Eor, XIndexedIndirect),
            opcode::eor::ZERO_PAGE => (Eor, ZeroPage),
            opcode::eor::ZERO_PAGE_X_INDEXED => (Eor, ZeroPageXIndexed),
            opcode::inc::ABSOLUTE => (Inc, Absolute),
            opcode::inc::ABSOLUTE_X_INDEXED => (Inc, AbsoluteXIndexed),
            opcode::inc::ZERO_PAGE => (Inc, ZeroPage),
            opcode::inc::ZERO_PAGE_X_INDEXED => (Inc, ZeroPageXIndexed),
            opcode::inx::IMPLIED => (Inx, Implied),
            opcode::iny::IMPLIED => (Iny, Implied),
            opcode::jmp::ABSOLUTE => (Jmp, Absolute),
            opcode::jmp::INDIRECT => (Jmp, Indirect),
            opcode::jsr::ABSOLUTE => (Jsr, Absolute),
            opcode::lda::ABSOLUTE => (Lda, Absolute),
            opcode::lda::ABSOLUTE_X_INDEXED => (Lda, AbsoluteXIndexed),
            opcode::lda::ABSOLUTE_Y_INDEXED => (Lda, AbsoluteYIndexed),
            opcode::lda::IMMEDIATE => (Lda, Immediate),
            opcode::lda::INDIRECT_Y_INDEXED => (Lda, IndirectYIndexed),
            opcode::lda::X_INDEXED_INDIRECT => (Lda, XIndexedIndirect),
            opcode::lda::ZERO_PAGE => (Lda, ZeroPage),
            opcode::lda::ZERO_PAGE_X_INDEXED => (Lda, ZeroPageXIndexed),
            opcode::ldx::ABSOLUTE => (Ldx, Absolute),
            opcode::ldx::ABSOLUTE_Y_INDEXED => (Ldx, AbsoluteYIndexed),
            opcode::ldx::IMMEDIATE => (Ldx, Immediate),
            opcode::ldx::ZERO_PAGE => (Ldx, ZeroPage),
            opcode::ldx::ZERO_PAGE_Y_INDEXED => (Ldx, ZeroPageYIndexed),
            opcode::ldy::ABSOLUTE => (Ldy, Absolute),
            opcode::ldy::ABSOLUTE_X_INDEXED => (Ldy, AbsoluteXIndexed),
            opcode::ldy::IMMEDIATE => (Ldy, Immediate),
            opcode::ldy::ZERO_PAGE => (Ldy, ZeroPage),
            opcode::ldy::ZERO_PAGE_X_INDEXED => (Ldy, ZeroPageXIndexed),
            opcode::lsr::ABSOLUTE => (Lsr, Absolute),
            opcode::lsr::ABSOLUTE_X_INDEXED => (Lsr, AbsoluteXIndexed),
            opcode::lsr::ACCUMULATOR => (Lsr, Accumulator),
            opcode::lsr::ZERO_PAGE => (Lsr, ZeroPage),
            opcode::lsr::ZERO_PAGE_X_INDEXED => (Lsr, ZeroPageXIndexed),
            opcode::nop::IMPLIED => (Nop, Implied),
            opcode::ora::ABSOLUTE => (Ora, Absolute),
            opcode::ora::ABSOLUTE_X_INDEXED => (Ora, AbsoluteXIndexed),
            opcode::ora::ABSOLUTE_Y_INDEXED => (Ora, AbsoluteYIndexed),
            opcode::ora::IMMEDIATE => (Ora, Immediate),
            opcode::ora::INDIRECT_Y_INDEXED => (Ora, IndirectYIndexed),
            opcode::ora::X_INDEXED_INDIRECT => (Ora, XIndexedIndirect),
            opcode::ora::ZERO_PAGE => (Ora, ZeroPage),
            opcode::ora::ZERO_PAGE_X_INDEXED => (Ora, ZeroPageXIndexed),
            opcode::pha::IMPLIED => (Pha, Implied),
            opcode::php::IMPLIED => (Php, Implied),
            opcode::pla::IMPLIED => (Pla, Implied),
            opcode::plp::IMPLIED => (Plp, Implied),
            opcode::rol::ABSOLUTE => (Rol, Absolute),
            opcode::rol::ABSOLUTE_X_INDEXED => (Rol, AbsoluteXIndexed),
            opcode::rol::ACCUMULATOR => (Rol, Accumulator),
            opcode::rol::ZERO_PAGE => (Rol, ZeroPage),
            opcode::rol::ZERO_PAGE_X_INDEXED => (Rol, ZeroPageXIndexed),
            opcode::ror::ABSOLUTE => (Ror, Absolute),
            opcode::ror::ABSOLUTE_X_INDEXED => (Ror, AbsoluteXIndexed),
            opcode::ror::ACCUMULATOR => (Ror, Accumulator),
            opcode::ror::ZERO_PAGE => (Ror, ZeroPage),
            opcode::ror::ZERO_PAGE_X_INDEXED => (Ror, ZeroPageXIndexed),
            opcode::rts::IMPLIED => (Rts, Implied),
            opcode::sbc::ABSOLUTE => (Sbc, Absolute),
            opcode::sbc::ABSOLUTE_X_INDEXED => (Sbc, AbsoluteXIndexed),
            opcode::sbc::ABSOLUTE_Y_INDEXED => (Sbc, AbsoluteYIndexed),
            opcode::sbc::IMMEDIATE => (Sbc, Immediate),
            opcode::sbc::INDIRECT_Y_INDEXED => (Sbc, IndirectYIndexed),
            opcode::sbc::X_INDEXED_INDIRECT => (Sbc, XIndexedIndirect),
            opcode::sbc::ZERO_PAGE => (Sbc, ZeroPage),
            opcode::sbc::ZERO_PAGE_X_INDEXED => (Sbc, ZeroPageXIndexed),
            opcode::sec::IMPLIED => (Sec, Implied),
            opcode::sed::IMPLIED => (Sed, Implied),
            opcode::sei::IMPLIED => (Sei, Implied),
            opcode::sta::ABSOLUTE => (Sta, Absolute),
            opcode::sta::ABSOLUTE_X_INDEXED => (Sta, AbsoluteXIndexed),
            opcode::sta::ABSOLUTE_Y_INDEXED => (Sta, AbsoluteYIndexed),
            opcode::sta::INDIRECT_Y_INDEXED => (Sta, IndirectYIndexed),
            opcode::sta::X_INDEXED_INDIRECT => (Sta, XIndexedIndirect),
            opcode::sta::ZERO_PAGE => (Sta, ZeroPage),
            opcode::sta::ZERO_PAGE_X_INDEXED => (Sta, ZeroPageXIndexed),
            opcode::tax::IMPLIED => (Tax, Implied),
            opcode::tay::IMPLIED => (Tay, Implied),
            opcode::tsx::IMPLIED => (Tsx, Implied),
            opcode::txa::IMPLIED => (Txa, Implied),
            opcode::txs::IMPLIED => (Txs, Implied),
            opcode::tya::IMPLIED => (Tya, Implied),
            _ => return Err(UnknownOpcode(opcode)),
        };
        Ok(Instruction::new(instruction_type, addressing_mode))
    }
    pub fn instruction_type(&self) -> InstructionType {
        self.instruction_type
    }
}
pub struct InstructionWithOperand {
    address: Address,
    instruction: Instruction,
    operand: Vec<u8>,
}
impl InstructionWithOperand {
    pub fn next<M: MemoryReadOnly>(cpu: &Cpu, memory: &M) -> Result<Self, UnknownOpcode> {
        let opcode = memory.read_u8_read_only(cpu.pc);
        let instruction = Instruction::from_opcode(opcode)?;
        let operand_bytes = instruction.addressing_mode.operand_bytes();
        let mut operand = Vec::new();
        for i in 0..operand_bytes {
            operand
                .push(memory.read_u8_read_only(cpu.pc.wrapping_add(i as Address).wrapping_add(1)));
        }
        Ok(Self {
            address: cpu.pc,
            instruction,
            operand,
        })
    }
}
impl fmt::Display for InstructionWithOperand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04X}  {:?}({:?}) ",
            self.address, self.instruction.instruction_type, self.instruction.addressing_mode
        )?;
        match self.operand.as_slice() {
            &[x] => write!(f, "{:02X}", x)?,
            &[x0, x1] => write!(f, "{:04X}", (x1 as u16) << 8 | x0 as u16)?,
            _ => (),
        }
        Ok(())
    }
}
