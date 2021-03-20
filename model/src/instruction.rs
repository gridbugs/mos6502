use crate::address;
use crate::addressing_mode::{self, *};
use crate::assembler_instruction::Trait as AssemblerInstruction;
use crate::machine::*;
use crate::opcode;
use crate::Address;

pub struct DataWithCycles {
    data: u8,
    cycles: u8,
}

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
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 5u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for XIndexedIndirect {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 6,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        if cpu.status.is_decimal() {
            log::warn!("decimal addition attempted");
        }
        adc_common(cpu, data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
    }
}
pub mod ahx {
    use super::*;
    use opcode::ahx::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn address_and_num_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> (Address, u8);
    }
    impl AddressingMode for IndirectYIndexed {
        fn address_and_num_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> (Address, u8) {
            let (address, cross_page_boundary) =
                Self::address_check_cross_page_boundary(cpu, memory);
            (address, 5u8.wrapping_add(cross_page_boundary as u8))
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn address_and_num_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> (Address, u8) {
            let (address, cross_page_boundary) =
                Self::address_check_cross_page_boundary(cpu, memory);
            (address, 4u8.wrapping_add(cross_page_boundary as u8))
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            unofficial0::INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let (target_address, num_cycles) = A::address_and_num_cycles(cpu, memory);
        let value = cpu.x & cpu.acc & address::hi(target_address).wrapping_add(1);
        let target_address = address::from_u8_lo_hi(address::lo(target_address), value);
        memory.write_u8(target_address, value);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        num_cycles
    }
}
pub mod alr {
    use super::*;
    use opcode::alr::*;
    pub trait AddressingMode: addressing_mode::Trait {}
    impl AddressingMode for Immediate {}
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            unofficial0::IMMEDIATE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = Immediate::read_data(cpu, memory);
        cpu.acc &= data;
        let carry = cpu.acc & 1 != 0;
        cpu.acc = cpu.acc.wrapping_shr(1);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.clear_negative();
        cpu.pc = cpu.pc.wrapping_add(Immediate::instruction_bytes());
        2
    }
}
pub mod arr {
    use super::*;
    use opcode::arr::*;
    pub trait AddressingMode: addressing_mode::Trait {}
    impl AddressingMode for Immediate {}
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            unofficial0::IMMEDIATE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = Immediate::read_data(cpu, memory);
        cpu.acc &= data;
        cpu.acc = cpu.acc.wrapping_shr(1) | cpu.status.carry_value().wrapping_shl(7);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        let carry = (cpu.acc & (1 << 6)) != 0;
        cpu.status.set_carry_to(carry);
        cpu.status
            .set_overflow_to(carry ^ ((cpu.acc & (1 << 5)) != 0));
        cpu.pc = cpu.pc.wrapping_add(Immediate::instruction_bytes());
        2
    }
}
pub mod anc {
    use super::*;
    use opcode::anc::*;
    pub trait AddressingMode: addressing_mode::Trait {}
    impl AddressingMode for Immediate {}
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            unofficial0::IMMEDIATE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = Immediate::read_data(cpu, memory);
        cpu.acc &= data;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.status.set_carry_to(cpu.status.is_negative());
        cpu.pc = cpu.pc.wrapping_add(Immediate::instruction_bytes());
        2
    }
}
pub mod and {
    use super::*;
    use opcode::and::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8 + page_boundary_cross as u8,
            }
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8 + page_boundary_cross as u8,
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 5,
            }
        }
    }
    impl AddressingMode for XIndexedIndirect {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 6,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        cpu.acc &= data;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
    }
}
pub mod asl {
    use super::*;
    use opcode::asl::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn num_cycles() -> u8;
    }
    pub trait MemoryAddressingMode: AddressingMode + ReadData + WriteData {}
    impl AddressingMode for Accumulator {
        fn num_cycles() -> u8 {
            2
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl MemoryAddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl MemoryAddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl MemoryAddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
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
    pub fn interpret<A: MemoryAddressingMode, M: Memory>(
        _: A,
        cpu: &mut Cpu,
        memory: &mut M,
    ) -> u8 {
        let data = A::read_data(cpu, memory);
        let carry = data & (1 << 7) != 0;
        let data = data.wrapping_shl(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(data);
        cpu.status.set_negative_from_value(data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
    pub fn interpret_acc(cpu: &mut Cpu) -> u8 {
        let carry = cpu.acc & (1 << 7) != 0;
        cpu.acc = cpu.acc.wrapping_shl(1);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Accumulator::instruction_bytes());
        Accumulator::num_cycles()
    }
}
pub mod axs {
    use super::*;
    use opcode::axs::*;
    pub trait AddressingMode: addressing_mode::Trait {}
    impl AddressingMode for Immediate {}
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            unofficial0::IMMEDIATE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = Immediate::read_data(cpu, memory);
        let (x, borrow) = (cpu.acc & cpu.x).overflowing_sub(data);
        cpu.x = x;
        cpu.status.set_zero_from_value(x);
        cpu.status.set_negative_from_value(x);
        cpu.status.set_carry_to(!borrow);
        cpu.pc = cpu.pc.wrapping_add(Immediate::instruction_bytes());
        2
    }
}
fn branch_next_pc_with_cycles(pc: Address, offset: i8) -> (Address, u8) {
    let next_pc = ((pc as i16).wrapping_add(offset as i16)) as Address;
    let cycles = 3 + address::on_different_pages(pc, next_pc) as u8;
    (next_pc, cycles)
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if !cpu.status.is_carry() {
            let offset = Relative::read_offset(cpu, memory);
            let (pc, cycles) = branch_next_pc_with_cycles(cpu.pc, offset);
            cpu.pc = pc;
            cycles
        } else {
            2
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if cpu.status.is_carry() {
            let offset = Relative::read_offset(cpu, memory);
            let (pc, cycles) = branch_next_pc_with_cycles(cpu.pc, offset);
            cpu.pc = pc;
            cycles
        } else {
            2
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if cpu.status.is_zero() {
            let offset = Relative::read_offset(cpu, memory);
            let (pc, cycles) = branch_next_pc_with_cycles(cpu.pc, offset);
            cpu.pc = pc;
            cycles
        } else {
            2
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if cpu.status.is_negative() {
            let offset = Relative::read_offset(cpu, memory);
            let (pc, cycles) = branch_next_pc_with_cycles(cpu.pc, offset);
            cpu.pc = pc;
            cycles
        } else {
            2
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if !cpu.status.is_zero() {
            let offset = Relative::read_offset(cpu, memory);
            let (pc, cycles) = branch_next_pc_with_cycles(cpu.pc, offset);
            cpu.pc = pc;
            cycles
        } else {
            2
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if !cpu.status.is_negative() {
            let offset = Relative::read_offset(cpu, memory);
            let (pc, cycles) = branch_next_pc_with_cycles(cpu.pc, offset);
            cpu.pc = pc;
            cycles
        } else {
            2
        }
    }
}
pub mod brk {
    use super::*;
    use opcode::brk::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let pc_to_save = cpu.pc.wrapping_add(2);
        cpu.push_stack_u8(memory, address::hi(pc_to_save));
        cpu.push_stack_u8(memory, address::lo(pc_to_save));
        cpu.push_stack_u8(memory, cpu.status.masked_with_brk_and_expansion());
        cpu.status.set_interrupt_disable();
        cpu.pc = memory.read_u16_le(crate::interrupt_vector::IRQ_LO);
        7
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if !cpu.status.is_overflow() {
            let offset = Relative::read_offset(cpu, memory);
            let (pc, cycles) = branch_next_pc_with_cycles(cpu.pc, offset);
            cpu.pc = pc;
            cycles
        } else {
            2
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.pc = cpu.pc.wrapping_add(Relative::instruction_bytes());
        if cpu.status.is_overflow() {
            let offset = Relative::read_offset(cpu, memory);
            let (pc, cycles) = branch_next_pc_with_cycles(cpu.pc, offset);
            cpu.pc = pc;
            cycles
        } else {
            2
        }
    }
}
pub mod bit {
    use super::*;
    use opcode::bit::*;
    pub trait AddressingMode: ReadData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            3
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            4
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory);
        let value = cpu.acc & data;
        cpu.status.set_zero_from_value(value);
        cpu.status.set_negative_from_value(data);
        cpu.status.set_overflow_to(data & (1 << 6) != 0);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.status.clear_carry();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.status.clear_decimal();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.status.clear_interrupt_disable();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.status.clear_overflow();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
    }
}
pub mod cmp {
    use super::*;
    use opcode::cmp::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 5u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for XIndexedIndirect {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 6,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        let (diff, borrow) = cpu.acc.overflowing_sub(data);
        cpu.status.set_zero_from_value(diff);
        cpu.status.set_negative_from_value(diff);
        cpu.status.set_carry_to(!borrow);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
    }
}
pub mod cpx {
    use super::*;
    use opcode::cpx::*;
    pub trait AddressingMode: ReadData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            4
        }
    }
    impl AddressingMode for Immediate {
        fn num_cycles() -> u8 {
            2
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            3
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory);
        let (diff, borrow) = cpu.x.overflowing_sub(data);
        cpu.status.set_zero_from_value(diff);
        cpu.status.set_negative_from_value(diff);
        cpu.status.set_carry_to(!borrow);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod cpy {
    use super::*;
    use opcode::cpy::*;
    pub trait AddressingMode: ReadData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            4
        }
    }
    impl AddressingMode for Immediate {
        fn num_cycles() -> u8 {
            2
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            3
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            ABSOLUTE
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory);
        let (diff, borrow) = cpu.y.overflowing_sub(data);
        cpu.status.set_zero_from_value(diff);
        cpu.status.set_negative_from_value(diff);
        cpu.status.set_carry_to(!borrow);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod dcp {
    use super::*;
    use opcode::dcp::*;
    pub trait AddressingMode: ReadData + WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for XIndexedIndirect {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            unofficial0::X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            unofficial0::INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory).wrapping_sub(1);
        A::write_data(cpu, memory, data);
        let (diff, borrow) = cpu.acc.overflowing_sub(data);
        cpu.status.set_zero_from_value(diff);
        cpu.status.set_negative_from_value(diff);
        cpu.status.set_carry_to(!borrow);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod dec {
    use super::*;
    use opcode::dec::*;
    pub trait AddressingMode: ReadData + WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory).wrapping_sub(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_negative_from_value(data);
        cpu.status.set_zero_from_value(data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.x = cpu.x.wrapping_sub(1);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.status.set_zero_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.y = cpu.y.wrapping_sub(1);
        cpu.status.set_negative_from_value(cpu.y);
        cpu.status.set_zero_from_value(cpu.y);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
    }
}
pub mod eor {
    use super::*;
    use opcode::eor::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 5u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for XIndexedIndirect {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 6,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        cpu.acc ^= data;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
    }
}
pub mod ign {
    use super::*;
    use opcode::ign::*;
    pub trait AddressingMode: ReadData {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8 + page_boundary_cross as u8,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data: _, cycles } = A::read_data_with_cycles(cpu, memory);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
    }
}
pub mod inc {
    use super::*;
    use opcode::inc::*;
    pub trait AddressingMode: ReadData + WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory).wrapping_add(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_negative_from_value(data);
        cpu.status.set_zero_from_value(data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.x = cpu.x.wrapping_add(1);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.status.set_zero_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
    }
}
pub mod iny {
    use super::*;
    use opcode::iny::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.y = cpu.y.wrapping_add(1);
        cpu.status.set_negative_from_value(cpu.y);
        cpu.status.set_zero_from_value(cpu.y);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
    }
}
pub mod isc {
    use super::*;
    use opcode::isc::*;
    pub trait AddressingMode: ReadData + WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for XIndexedIndirect {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            unofficial0::X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            unofficial0::INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory).wrapping_add(1);
        A::write_data(cpu, memory, data);
        adc_common(cpu, !data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod jmp {
    use super::*;
    use opcode::jmp::*;
    pub trait AddressingMode: ReadJumpTarget {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            3
        }
    }
    impl AddressingMode for Indirect {
        fn num_cycles() -> u8 {
            5
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.pc = A::read_jump_target(cpu, memory);
        A::num_cycles()
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let return_address = cpu.pc.wrapping_add(2);
        cpu.push_stack_u8(memory, address::hi(return_address));
        cpu.push_stack_u8(memory, address::lo(return_address));
        cpu.pc = A::read_jump_target(cpu, memory);
        6
    }
}
pub mod lax {
    use super::*;
    use opcode::lax::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for XIndexedIndirect {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 6,
            }
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 5u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<Immediate> {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            unofficial0::IMMEDIATE
        }
    }
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            unofficial0::X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            unofficial0::INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageYIndexed> {
        type AddressingMode = ZeroPageYIndexed;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        cpu.acc = data;
        cpu.x = data;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
    }
}
pub mod lda {
    use super::*;
    use opcode::lda::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 5u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for XIndexedIndirect {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 6,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        cpu.acc = data;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
    }
}
pub mod ldx {
    use super::*;
    use opcode::ldx::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }

    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        cpu.x = data;
        cpu.status.set_zero_from_value(cpu.x);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
    }
}
pub mod ldy {
    use super::*;
    use opcode::ldy::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        cpu.y = data;
        cpu.status.set_zero_from_value(cpu.y);
        cpu.status.set_negative_from_value(cpu.y);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
    }
}
pub mod lsr {
    use super::*;
    use opcode::lsr::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn num_cycles() -> u8;
    }
    pub trait MemoryAddressingMode: AddressingMode + ReadData + WriteData {}
    impl AddressingMode for Accumulator {
        fn num_cycles() -> u8 {
            2
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl MemoryAddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl MemoryAddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl MemoryAddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
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
    pub fn interpret<A: MemoryAddressingMode, M: Memory>(
        _: A,
        cpu: &mut Cpu,
        memory: &mut M,
    ) -> u8 {
        let data = A::read_data(cpu, memory);
        let carry = data & 1 != 0;
        let data = data.wrapping_shr(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(data);
        cpu.status.clear_negative();
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
    pub fn interpret_acc(cpu: &mut Cpu) -> u8 {
        let carry = cpu.acc & 1 != 0;
        cpu.acc = cpu.acc.wrapping_shr(1);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.clear_negative();
        cpu.pc = cpu.pc.wrapping_add(Accumulator::instruction_bytes());
        Accumulator::num_cycles()
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
    }
}
pub mod ora {
    use super::*;
    use opcode::ora::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8 + page_boundary_cross as u8,
            }
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8 + page_boundary_cross as u8,
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 5,
            }
        }
    }
    impl AddressingMode for XIndexedIndirect {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 6,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        cpu.acc |= data;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.push_stack_u8(memory, cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        3
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.push_stack_u8(memory, cpu.status.masked_with_brk_and_expansion());
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        3
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        cpu.acc = cpu.pop_stack_u8(memory);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        4
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let status = cpu.pop_stack_u8(memory);
        cpu.status.set(status);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        4
    }
}
pub mod rla {
    use super::*;
    use opcode::rla::*;
    pub trait AddressingMode: ReadData + WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for XIndexedIndirect {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            unofficial0::X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            unofficial0::INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory);
        let carry = data & (1 << 7) != 0;
        let data = data.wrapping_shl(1) | cpu.status.carry_value();
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.acc &= data;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod rol {
    use super::*;
    use opcode::rol::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn num_cycles() -> u8;
    }
    pub trait MemoryAddressingMode: AddressingMode + ReadData + WriteData {}
    impl AddressingMode for Accumulator {
        fn num_cycles() -> u8 {
            2
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl MemoryAddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl MemoryAddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl MemoryAddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
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
    pub fn interpret<A: MemoryAddressingMode, M: Memory>(
        _: A,
        cpu: &mut Cpu,
        memory: &mut M,
    ) -> u8 {
        let data = A::read_data(cpu, memory);
        let carry = data & (1 << 7) != 0;
        let data = data.wrapping_shl(1) | cpu.status.carry_value();
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(data);
        cpu.status.set_negative_from_value(data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
    pub fn interpret_acc(cpu: &mut Cpu) -> u8 {
        let carry = cpu.acc & (1 << 7) != 0;
        cpu.acc = cpu.acc.wrapping_shl(1) | cpu.status.carry_value();
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Accumulator::instruction_bytes());
        Accumulator::num_cycles()
    }
}
pub mod ror {
    use super::*;
    use opcode::ror::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn num_cycles() -> u8;
    }
    pub trait MemoryAddressingMode: AddressingMode + ReadData + WriteData {}
    impl AddressingMode for Accumulator {
        fn num_cycles() -> u8 {
            2
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl MemoryAddressingMode for Absolute {}
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl MemoryAddressingMode for AbsoluteXIndexed {}
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl MemoryAddressingMode for ZeroPage {}
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
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
    pub fn interpret<A: MemoryAddressingMode, M: Memory>(
        _: A,
        cpu: &mut Cpu,
        memory: &mut M,
    ) -> u8 {
        let data = A::read_data(cpu, memory);
        let carry = data & 1 != 0;
        let data = data.wrapping_shr(1) | cpu.status.carry_value().wrapping_shl(7);
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(data);
        cpu.status.set_negative_from_value(data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
    pub fn interpret_acc(cpu: &mut Cpu) -> u8 {
        let carry = cpu.acc & 1 != 0;
        cpu.acc = cpu.acc.wrapping_shr(1) | cpu.status.carry_value().wrapping_shl(7);
        cpu.status.set_carry_to(carry);
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Accumulator::instruction_bytes());
        Accumulator::num_cycles()
    }
}
pub mod rra {
    use super::*;
    use opcode::rra::*;
    pub trait AddressingMode: ReadData + WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for XIndexedIndirect {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            unofficial0::X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            unofficial0::INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory);
        let carry = data & 1 != 0;
        let data = data.wrapping_shr(1) | cpu.status.carry_value().wrapping_shl(7);
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        adc_common(cpu, data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod rti {
    use super::*;
    use opcode::rti::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Implied;
        fn opcode() -> u8 {
            IMPLIED
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let status = cpu.pop_stack_u8(memory);
        let return_address_lo = cpu.pop_stack_u8(memory);
        let return_address_hi = cpu.pop_stack_u8(memory);
        cpu.status.set(status);
        cpu.pc = address::from_u8_lo_hi(return_address_lo, return_address_hi);
        6
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
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let return_address_lo = cpu.pop_stack_u8(memory);
        let return_address_hi = cpu.pop_stack_u8(memory);
        cpu.pc = address::from_u8_lo_hi(return_address_lo, return_address_hi).wrapping_add(1);
        6
    }
}
pub mod sax {
    use super::*;
    use opcode::sax::*;
    pub trait AddressingMode: WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for XIndexedIndirect {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            3
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            4
        }
    }
    impl AddressingMode for ZeroPageYIndexed {
        fn num_cycles() -> u8 {
            4
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            unofficial0::X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<Absolute> {
        type AddressingMode = Absolute;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageYIndexed> {
        type AddressingMode = ZeroPageYIndexed;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        A::write_data(cpu, memory, cpu.acc & cpu.x);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod sbc {
    use super::*;
    use opcode::sbc::*;
    pub trait AddressingMode: addressing_mode::Trait {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles;
    }
    impl AddressingMode for Absolute {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 4u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for Immediate {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 2,
            }
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            let (data, page_boundary_cross) =
                Self::read_data_check_cross_page_boundary(cpu, memory);
            DataWithCycles {
                data,
                cycles: 5u8.wrapping_add(page_boundary_cross as u8),
            }
        }
    }
    impl AddressingMode for XIndexedIndirect {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 6,
            }
        }
    }
    impl AddressingMode for ZeroPage {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 3,
            }
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn read_data_with_cycles<M: Memory>(cpu: &Cpu, memory: &mut M) -> DataWithCycles {
            DataWithCycles {
                data: Self::read_data(cpu, memory),
                cycles: 4,
            }
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let DataWithCycles { data, cycles } = A::read_data_with_cycles(cpu, memory);
        if cpu.status.is_decimal() {
            log::warn!("decimal subtraction attempted");
        }
        adc_common(cpu, !data);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        cycles
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.status.set_carry();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.status.set_decimal();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.status.set_interrupt_disable();
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
    }
}
pub mod skb {
    use super::*;
    use opcode::skb::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = Immediate;
        fn opcode() -> u8 {
            unofficial0::IMMEDIATE
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let _ = Immediate::read_data(cpu, memory);
        cpu.pc = cpu.pc.wrapping_add(Immediate::instruction_bytes());
        2
    }
}
pub mod slo {
    use super::*;
    use opcode::slo::*;
    pub trait AddressingMode: ReadData + WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for XIndexedIndirect {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            unofficial0::X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            unofficial0::INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory);
        let carry = data & (1 << 7) != 0;
        let data = data.wrapping_shl(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.acc |= data;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod sre {
    use super::*;
    use opcode::sre::*;
    pub trait AddressingMode: ReadData + WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for XIndexedIndirect {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn num_cycles() -> u8 {
            8
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn num_cycles() -> u8 {
            7
        }
    }
    pub struct Inst<A: AddressingMode>(pub A);
    impl AssemblerInstruction for Inst<XIndexedIndirect> {
        type AddressingMode = XIndexedIndirect;
        fn opcode() -> u8 {
            unofficial0::X_INDEXED_INDIRECT
        }
    }
    impl AssemblerInstruction for Inst<ZeroPage> {
        type AddressingMode = ZeroPage;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE
        }
    }
    impl AssemblerInstruction for Inst<IndirectYIndexed> {
        type AddressingMode = IndirectYIndexed;
        fn opcode() -> u8 {
            unofficial0::INDIRECT_Y_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            unofficial0::ZERO_PAGE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteXIndexed> {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_X_INDEXED
        }
    }
    impl AssemblerInstruction for Inst<AbsoluteYIndexed> {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        let data = A::read_data(cpu, memory);
        let carry = data & 1 != 0;
        let data = data.wrapping_shr(1);
        A::write_data(cpu, memory, data);
        cpu.status.set_carry_to(carry);
        cpu.acc ^= data;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod sta {
    use super::*;
    use opcode::sta::*;
    pub trait AddressingMode: WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            4
        }
    }
    impl AddressingMode for AbsoluteXIndexed {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for AbsoluteYIndexed {
        fn num_cycles() -> u8 {
            5
        }
    }
    impl AddressingMode for IndirectYIndexed {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for XIndexedIndirect {
        fn num_cycles() -> u8 {
            6
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            3
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            4
        }
    }
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
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        A::write_data(cpu, memory, cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod stx {
    use super::*;
    use opcode::stx::*;
    pub trait AddressingMode: WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            4
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            3
        }
    }
    impl AddressingMode for ZeroPageYIndexed {
        fn num_cycles() -> u8 {
            4
        }
    }
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
    impl AssemblerInstruction for Inst<ZeroPageYIndexed> {
        type AddressingMode = ZeroPageYIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_Y_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        A::write_data(cpu, memory, cpu.x);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
    }
}
pub mod sty {
    use super::*;
    use opcode::sty::*;
    pub trait AddressingMode: WriteData {
        fn num_cycles() -> u8;
    }
    impl AddressingMode for Absolute {
        fn num_cycles() -> u8 {
            4
        }
    }
    impl AddressingMode for ZeroPage {
        fn num_cycles() -> u8 {
            3
        }
    }
    impl AddressingMode for ZeroPageXIndexed {
        fn num_cycles() -> u8 {
            4
        }
    }
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
    impl AssemblerInstruction for Inst<ZeroPageXIndexed> {
        type AddressingMode = ZeroPageXIndexed;
        fn opcode() -> u8 {
            ZERO_PAGE_X_INDEXED
        }
    }
    pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) -> u8 {
        A::write_data(cpu, memory, cpu.y);
        cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
        A::num_cycles()
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.x = cpu.acc;
        cpu.status.set_zero_from_value(cpu.x);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
    }
}
pub mod sxa {
    use super::*;
    use opcode::sxa::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = AbsoluteYIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_Y_INDEXED
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let target_address = AbsoluteYIndexed::address(cpu, memory);
        let value = cpu.x & address::hi(target_address).wrapping_add(1);
        let target_address = address::from_u8_lo_hi(address::lo(target_address), value);
        memory.write_u8(target_address, value);
        cpu.pc = cpu.pc.wrapping_add(AbsoluteYIndexed::instruction_bytes());
        5
    }
}
pub mod sya {
    use super::*;
    use opcode::sya::*;
    pub struct Inst;
    impl AssemblerInstruction for Inst {
        type AddressingMode = AbsoluteXIndexed;
        fn opcode() -> u8 {
            unofficial0::ABSOLUTE_X_INDEXED
        }
    }
    pub fn interpret<M: Memory>(cpu: &mut Cpu, memory: &mut M) -> u8 {
        let target_address = AbsoluteXIndexed::address(cpu, memory);
        let value = cpu.y & address::hi(target_address).wrapping_add(1);
        let target_address = address::from_u8_lo_hi(address::lo(target_address), value);
        memory.write_u8(target_address, value);
        cpu.pc = cpu.pc.wrapping_add(AbsoluteXIndexed::instruction_bytes());
        5
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.y = cpu.acc;
        cpu.status.set_zero_from_value(cpu.y);
        cpu.status.set_negative_from_value(cpu.y);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.x = cpu.sp;
        cpu.status.set_zero_from_value(cpu.x);
        cpu.status.set_negative_from_value(cpu.x);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.acc = cpu.x;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.sp = cpu.x;
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
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
    pub fn interpret(cpu: &mut Cpu) -> u8 {
        cpu.acc = cpu.y;
        cpu.status.set_zero_from_value(cpu.acc);
        cpu.status.set_negative_from_value(cpu.acc);
        cpu.pc = cpu.pc.wrapping_add(Implied::instruction_bytes());
        2
    }
}
