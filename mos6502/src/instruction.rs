use crate::address;
use crate::addressing_mode::*;
use crate::assembler_instruction::Trait as AssemblerInstruction;
use crate::machine::*;
use crate::opcode;
use crate::Address;

fn adc_common(cpu: &mut Cpu, value: u8) {
    let carry_value = cpu.status.carry_value();
    let (sum, carry0) = cpu.acc.overflowing_add(value);
    let (sum, carry1) = sum.overflowing_add(carry_value);
    let overflow_candidate = !(cpu.acc ^ value); // sign bits must match for overflow to occur
    let overflow_if_candidate = cpu.acc ^ sum; // the sign bit changing indicates an overflow
    let overflow = (overflow_candidate & overflow_if_candidate) & (1 << 7) != 0;
    cpu.acc = sum;
    cpu.status.set_overflow_to(overflow);
    cpu.status.set_carry_to(carry0 || carry1);
    cpu.status.set_zero_from_value(cpu.acc);
    cpu.status.set_negative_from_value(cpu.acc);
}
pub mod adc {
    use super::*;
    use opcode::adc::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for AbsoluteYIndexed {}
    impl AddressingMode for Immediate {}
    impl AddressingMode for IndirectYIndexed {}
    impl AddressingMode for XIndexedIndirect {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let value = A::read_data(cpu, memory);
        if cpu.status.is_decimal() {
            panic!("decimal addition not implemented");
        } else {
            adc_common(cpu, value);
        }
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod and {
    use super::*;
    use opcode::and::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for AbsoluteYIndexed {}
    impl AddressingMode for Immediate {}
    impl AddressingMode for IndirectYIndexed {}
    impl AddressingMode for XIndexedIndirect {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let value = A::read_data(cpu, memory);
        cpu.acc &= value;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod asl {
    use super::*;
    use opcode::asl::*;
    pub trait AddressingMode: Trait {}
    pub trait MemoryAddressingMode: ReadData + WriteData + AddressingMode {}
    impl AddressingMode for Accumulator {}
    impl AddressingMode for Absolute {}
    impl MemoryAddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl MemoryAddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {}
    impl MemoryAddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    impl MemoryAddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Accumulator> {
        type AddressingMode = Accumulator;
        fn opcode() -> u8 {
            ACCUMULATOR
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: MemoryAddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let data = A::read_data(cpu, memory);
        let carry = data & (1 << 7) != 0;
        let data = data.wrapping_shl(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
    pub fn interpret_acc(cpu: &mut Cpu) {
        let carry = cpu.acc & (1 << 7) != 0;
        cpu.acc = cpu.acc.wrapping_shl(1);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Accumulator::instruction_bytes());
    }
}
pub mod bcc {
    use super::*;
    use opcode::bcc::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Relative;
        fn opcode() -> u8 {
            RELATIVE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if !cpu.status.is_carry() {
            let offset = Relative::read_offset(cpu, memory);
            cpu.pc = ((cpu.pc as i16).wrapping_add(offset as i16)) as Address;
        }
    }
}
pub mod bcs {
    use super::*;
    use opcode::bcs::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Relative;
        fn opcode() -> u8 {
            RELATIVE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if cpu.status.is_carry() {
            let offset = Relative::read_offset(cpu, memory);
            cpu.pc = ((cpu.pc as i16).wrapping_add(offset as i16)) as Address;
        }
    }
}
pub mod beq {
    use super::*;
    use opcode::beq::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Relative;
        fn opcode() -> u8 {
            RELATIVE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if cpu.status.is_zero() {
            let offset = Relative::read_offset(cpu, memory);
            cpu.pc = ((cpu.pc as i16).wrapping_add(offset as i16)) as Address;
        }
    }
}
pub mod bmi {
    use super::*;
    use opcode::bmi::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Relative;
        fn opcode() -> u8 {
            RELATIVE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if cpu.status.is_negative() {
            let offset = Relative::read_offset(cpu, memory);
            cpu.pc = ((cpu.pc as i16).wrapping_add(offset as i16)) as Address;
        }
    }
}
pub mod bne {
    use super::*;
    use opcode::bne::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Relative;
        fn opcode() -> u8 {
            RELATIVE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if !cpu.status.is_zero() {
            let offset = Relative::read_offset(cpu, memory);
            println!("jumping relative {:x?}", offset);
            cpu.pc = ((cpu.pc as i16).wrapping_add(offset as i16)) as Address;
        }
    }
}
pub mod bpl {
    use super::*;
    use opcode::bpl::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Relative;
        fn opcode() -> u8 {
            RELATIVE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if !cpu.status.is_negative() {
            let offset = Relative::read_offset(cpu, memory);
            cpu.pc = ((cpu.pc as i16).wrapping_add(offset as i16)) as Address;
        }
    }
}
pub mod bvc {
    use super::*;
    use opcode::bvc::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Relative;
        fn opcode() -> u8 {
            RELATIVE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if !cpu.status.is_overflow() {
            let offset = Relative::read_offset(cpu, memory);
            cpu.pc = ((cpu.pc as i16).wrapping_add(offset as i16)) as Address;
        }
    }
}
pub mod bvs {
    use super::*;
    use opcode::bvs::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Relative;
        fn opcode() -> u8 {
            RELATIVE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if cpu.status.is_overflow() {
            let offset = Relative::read_offset(cpu, memory);
            cpu.pc = ((cpu.pc as i16).wrapping_add(offset as i16)) as Address;
        }
    }
}
pub mod bit {
    use super::*;
    use opcode::bit::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for Absolute {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let data = A::read_data(cpu, memory);
        let value = cpu.acc & data;
        cpu.status.set_zero_from_value(value);
        cpu.status.set_negative_from_value(value);
        cpu.status.set_overflow_to(value & (1 << 6) != 0);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod clc {
    use super::*;
    use opcode::clc::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.status.clear_carry();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod cld {
    use super::*;
    use opcode::cld::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.status.clear_decimal();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod cli {
    use super::*;
    use opcode::cli::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.status.clear_interrupt_disable();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod clv {
    use super::*;
    use opcode::cli::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.status.clear_overflow();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod cmp {
    use super::*;
    use opcode::cmp::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for AbsoluteYIndexed {}
    impl AddressingMode for Immediate {}
    impl AddressingMode for IndirectYIndexed {}
    impl AddressingMode for XIndexedIndirect {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let value = A::read_data(cpu, memory);
        let (diff, borrow) = cpu.acc.overflowing_sub(value);
        cpu.status.set_zero_from_value(diff);
        cpu.status.set_negative_from_value(diff);
        cpu.status.set_carry_to(!borrow);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod dec {
    use super::*;
    use opcode::dec::*;
    pub trait AddressingMode: ReadData + WriteData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let data = A::read_data(cpu, memory).wrapping_sub(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_negative_from_value(data);
        cpu.status.set_zero_from_value(data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod dex {
    use super::*;
    use opcode::dex::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.x = cpu.x.wrapping_sub(1);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.status.set_zero_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod dey {
    use super::*;
    use opcode::dey::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.y = cpu.y.wrapping_sub(1);
        cpu.status.set_negative_from_value(cpu.y);
        cpu.status.set_zero_from_value(cpu.y);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod eor {
    use super::*;
    use opcode::eor::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for AbsoluteYIndexed {}
    impl AddressingMode for Immediate {}
    impl AddressingMode for IndirectYIndexed {}
    impl AddressingMode for XIndexedIndirect {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let value = A::read_data(cpu, memory);
        cpu.acc ^= value;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod inc {
    use super::*;
    use opcode::inc::*;
    pub trait AddressingMode: ReadData + WriteData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let data = A::read_data(cpu, memory).wrapping_add(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_negative_from_value(data);
        cpu.status.set_zero_from_value(data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod inx {
    use super::*;
    use opcode::inx::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.x = cpu.x.wrapping_add(1);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.status.set_zero_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod iny {
    use super::*;
    use opcode::dey::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.y = cpu.y.wrapping_add(1);
        cpu.status.set_negative_from_value(cpu.y);
        cpu.status.set_zero_from_value(cpu.y);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod jmp {
    use super::*;
    use opcode::jmp::*;
    pub trait AddressingMode: ReadJumpTarget {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for Indirect {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<Indirect> {
        type AddressingMode = Indirect;
        fn opcode() -> u8 {
            INDIRECT
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        cpu.pc = A::read_jump_target(cpu, memory);
    }
}
pub mod jsr {
    use super::*;
    use opcode::jsr::*;
    pub trait AddressingMode: ReadJumpTarget {}
    impl AddressingMode for Absolute {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let return_address = cpu.pc.wrapping_add(2);
        cpu.push_stack_u8(memory, address::hi(return_address));
        cpu.push_stack_u8(memory, address::lo(return_address));
        cpu.pc = A::read_jump_target(cpu, memory);
    }
}
pub mod lda {
    use super::*;
    use opcode::lda::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for Immediate {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for AbsoluteYIndexed {}
    impl AddressingMode for XIndexedIndirect {}
    impl AddressingMode for IndirectYIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        cpu.acc = A::read_data(cpu, memory);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod ldx {
    use super::*;
    use opcode::ldx::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteYIndexed {}
    impl AddressingMode for Immediate {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageYIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageYIndexed> {
        type AddressingMode = ZeroPageYIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        cpu.x = A::read_data(cpu, memory);
        cpu.status.set_zero_from_value(cpu.x);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod ldy {
    use super::*;
    use opcode::ldy::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for Immediate {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        cpu.y = A::read_data(cpu, memory);
        cpu.status.set_zero_from_value(cpu.y);
        cpu.status.set_negative_from_value(cpu.y);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod lsr {
    use super::*;
    use opcode::lsr::*;
    pub trait AddressingMode: Trait {}
    pub trait MemoryAddressingMode: ReadData + WriteData + AddressingMode {}
    impl AddressingMode for Accumulator {}
    impl AddressingMode for Absolute {}
    impl MemoryAddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl MemoryAddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {}
    impl MemoryAddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    impl MemoryAddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Accumulator> {
        type AddressingMode = Accumulator;
        fn opcode() -> u8 {
            ACCUMULATOR
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: MemoryAddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let data = A::read_data(cpu, memory);
        let carry = data & 1 != 0;
        let data = data.wrapping_shr(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.clear_negative();
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
    pub fn interpret_acc(cpu: &mut Cpu) {
        let carry = cpu.acc & 1 != 0;
        cpu.acc = cpu.acc.wrapping_shr(1);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.clear_negative();
        cpu.pc = cpu.pc.wrapping_add(Accumulator::instruction_bytes());
    }
}
pub mod nop {
    use super::*;
    use opcode::nop::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod ora {
    use super::*;
    use opcode::ora::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for AbsoluteYIndexed {}
    impl AddressingMode for Immediate {}
    impl AddressingMode for IndirectYIndexed {}
    impl AddressingMode for XIndexedIndirect {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let value = A::read_data(cpu, memory);
        cpu.acc |= value;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod pha {
    use super::*;
    use opcode::pha::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.push_stack_u8(memory, cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod php {
    use super::*;
    use opcode::php::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.push_stack_u8(memory, cpu.status.raw);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod pla {
    use super::*;
    use opcode::pla::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        cpu.acc = cpu.pop_stack_u8(memory);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod plp {
    use super::*;
    use opcode::plp::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        let status_raw = cpu.pop_stack_u8(memory);
        cpu.status.raw = status_raw;
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod rol {
    use super::*;
    use opcode::rol::*;
    pub trait AddressingMode: Trait {}
    pub trait MemoryAddressingMode: ReadData + WriteData + AddressingMode {}
    impl AddressingMode for Accumulator {}
    impl AddressingMode for Absolute {}
    impl MemoryAddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl MemoryAddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {}
    impl MemoryAddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    impl MemoryAddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Accumulator> {
        type AddressingMode = Accumulator;
        fn opcode() -> u8 {
            ACCUMULATOR
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: MemoryAddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let data = A::read_data(cpu, memory);
        let carry = data & (1 << 7) != 0;
        let data = data.wrapping_shl(1) | cpu.status.carry_value();
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
    pub fn interpret_acc(cpu: &mut Cpu) {
        let carry = cpu.acc & (1 << 7) != 0;
        cpu.acc = cpu.acc.wrapping_shl(1) | cpu.status.carry_value();
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Accumulator::instruction_bytes());
    }
}
pub mod ror {
    use super::*;
    use opcode::ror::*;
    pub trait AddressingMode: Trait {}
    pub trait MemoryAddressingMode: ReadData + WriteData + AddressingMode {}
    impl AddressingMode for Accumulator {}
    impl AddressingMode for Absolute {}
    impl MemoryAddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl MemoryAddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {}
    impl MemoryAddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    impl MemoryAddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Accumulator> {
        type AddressingMode = Accumulator;
        fn opcode() -> u8 {
            ACCUMULATOR
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: MemoryAddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let data = A::read_data(cpu, memory);
        let carry = data & 1 != 0;
        let data = data.wrapping_shr(1) | cpu.status.carry_value().wrapping_shl(7);
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
    pub fn interpret_acc(cpu: &mut Cpu) {
        let carry = cpu.acc & 1 != 0;
        cpu.acc = cpu.acc.wrapping_shr(1) | cpu.status.carry_value().wrapping_shl(7);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Accumulator::instruction_bytes());
    }
}
pub mod rts {
    use super::*;
    use opcode::rts::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) {
        let return_address_lo = cpu.pop_stack_u8(memory);
        let return_address_hi = cpu.pop_stack_u8(memory);
        cpu.pc = address::from_u8_lo_hi(return_address_lo, return_address_hi).wrapping_add(1);
    }
}
pub mod sbc {
    use super::*;
    use opcode::sbc::*;
    pub trait AddressingMode: ReadData {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for AbsoluteYIndexed {}
    impl AddressingMode for Immediate {}
    impl AddressingMode for IndirectYIndexed {}
    impl AddressingMode for XIndexedIndirect {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        let value = A::read_data(cpu, memory);
        if cpu.status.is_decimal() {
            panic!("decimal subtraction not implemented");
        } else {
            adc_common(cpu, !value);
        }
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod sec {
    use super::*;
    use opcode::sec::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.status.set_carry();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod sed {
    use super::*;
    use opcode::sed::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.status.set_decimal();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod sei {
    use super::*;
    use opcode::sei::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.status.set_interrupt_disable();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod sta {
    use super::*;
    use opcode::sta::*;
    pub trait AddressingMode: WriteData {}
    impl AddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {}
    impl AddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for AbsoluteYIndexed {}
    impl AddressingMode for XIndexedIndirect {}
    impl AddressingMode for IndirectYIndexed {}
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
        A::write_data(cpu, memory, cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
    }
}
pub mod tax {
    use super::*;
    use opcode::tax::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.x = cpu.acc;
        cpu.status.set_zero_from_value(cpu.x);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod tay {
    use super::*;
    use opcode::tay::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.y = cpu.acc;
        cpu.status.set_zero_from_value(cpu.y);
        cpu.status.set_negative_from_value(cpu.y);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod tsx {
    use super::*;
    use opcode::tsx::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.x = cpu.sp;
        cpu.status.set_zero_from_value(cpu.x);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod txa {
    use super::*;
    use opcode::txa::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.acc = cpu.x;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod txs {
    use super::*;
    use opcode::txs::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.sp = cpu.x;
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
pub mod tya {
    use super::*;
    use opcode::tya::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) {
        cpu.acc = cpu.y;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
    }
}
