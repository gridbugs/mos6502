use crate::address;
use crate::machine::{Cpu, Memory};
use crate::operand;
use crate::Address;

pub trait Trait {
    type Operand: operand::Trait;
    fn instruction_bytes() -> u16 {
        <Self::Operand as operand::Trait>::instruction_bytes()
    }
}

pub trait ReadJumpTarget: Trait {
    fn read_jump_target<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address;
}

pub trait ReadData: Trait {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8;
}

pub trait WriteData: Trait {
    fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8);
}

pub struct Absolute;
impl Trait for Absolute {
    type Operand = operand::Address;
}
impl Absolute {
    fn address<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address {
        memory.read_u16_le(cpu.pc.wrapping_add(1))
    }
}
impl ReadJumpTarget for Absolute {
    fn read_jump_target<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address {
        Self::address(cpu, memory)
    }
}
impl ReadData for Absolute {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
        let address = Self::address(cpu, memory);
        memory.read_u8(address)
    }
}
impl WriteData for Absolute {
    fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
        let address = Self::address(cpu, memory);
        memory.write_u8(address, data)
    }
}

pub struct AbsoluteXIndexed;
impl Trait for AbsoluteXIndexed {
    type Operand = operand::Address;
}
impl AbsoluteXIndexed {
    fn address<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address {
        let base_address = memory.read_u16_le(cpu.pc.wrapping_add(1));
        base_address.wrapping_add(cpu.x as Address)
    }
    fn address_check_cross_page_boundary<M: Memory>(cpu: &Cpu, memory: &mut M) -> (Address, bool) {
        let base_address = memory.read_u16_le(cpu.pc.wrapping_add(1));
        let indexed_address = base_address.wrapping_add(cpu.x as Address);
        (
            indexed_address,
            address::on_different_pages(base_address, indexed_address),
        )
    }
    pub fn read_data_check_cross_page_boundary<M: Memory>(cpu: &Cpu, memory: &mut M) -> (u8, bool) {
        let (address, cross_page_boundary) = Self::address_check_cross_page_boundary(cpu, memory);
        (memory.read_u8(address), cross_page_boundary)
    }
}
impl ReadData for AbsoluteXIndexed {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
        let address = Self::address(cpu, memory);
        memory.read_u8(address)
    }
}
impl WriteData for AbsoluteXIndexed {
    fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
        let address = Self::address(cpu, memory);
        memory.write_u8(address, data)
    }
}

pub struct AbsoluteYIndexed;
impl Trait for AbsoluteYIndexed {
    type Operand = operand::Address;
}
impl AbsoluteYIndexed {
    fn address<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address {
        let base_address = memory.read_u16_le(cpu.pc.wrapping_add(1));
        base_address.wrapping_add(cpu.y as Address)
    }
    fn address_check_cross_page_boundary<M: Memory>(cpu: &Cpu, memory: &mut M) -> (Address, bool) {
        let base_address = memory.read_u16_le(cpu.pc.wrapping_add(1));
        let indexed_address = base_address.wrapping_add(cpu.y as Address);
        (
            indexed_address,
            address::on_different_pages(base_address, indexed_address),
        )
    }
    pub fn read_data_check_cross_page_boundary<M: Memory>(cpu: &Cpu, memory: &mut M) -> (u8, bool) {
        let (address, cross_page_boundary) = Self::address_check_cross_page_boundary(cpu, memory);
        (memory.read_u8(address), cross_page_boundary)
    }
}
impl ReadData for AbsoluteYIndexed {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
        let address = Self::address(cpu, memory);
        memory.read_u8(address)
    }
}
impl WriteData for AbsoluteYIndexed {
    fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
        let address = Self::address(cpu, memory);
        memory.write_u8(address, data)
    }
}

pub struct Accumulator;
impl Trait for Accumulator {
    type Operand = operand::None;
}

pub struct Immediate;
impl Trait for Immediate {
    type Operand = operand::Byte;
}
impl ReadData for Immediate {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
        memory.read_u8(cpu.pc.wrapping_add(1))
    }
}

pub struct Implied;
impl Trait for Implied {
    type Operand = operand::None;
}

pub struct Indirect;
impl Trait for Indirect {
    type Operand = operand::Address;
}
impl ReadJumpTarget for Indirect {
    fn read_jump_target<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address {
        let address = memory.read_u16_le(cpu.pc.wrapping_add(1));
        memory.read_u16_le(address)
    }
}

pub struct IndirectYIndexed;
impl Trait for IndirectYIndexed {
    type Operand = operand::Byte;
}
impl IndirectYIndexed {
    fn address<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address {
        let base_address = memory.read_u8(cpu.pc.wrapping_add(1)) as Address;
        memory
            .read_u16_le(base_address)
            .wrapping_add(cpu.y as Address)
    }
    fn address_check_cross_page_boundary<M: Memory>(cpu: &Cpu, memory: &mut M) -> (Address, bool) {
        let indirect_address = memory.read_u8(cpu.pc.wrapping_add(1)) as Address;
        let base_address = memory.read_u16_le(indirect_address);
        let indexed_address = base_address.wrapping_add(cpu.y as Address);
        (
            indexed_address,
            address::on_different_pages(base_address, indexed_address),
        )
    }
    pub fn read_data_check_cross_page_boundary<M: Memory>(cpu: &Cpu, memory: &mut M) -> (u8, bool) {
        let (address, cross_page_boundary) = Self::address_check_cross_page_boundary(cpu, memory);
        (memory.read_u8(address), cross_page_boundary)
    }
}
impl ReadData for IndirectYIndexed {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
        let address = Self::address(cpu, memory);
        memory.read_u8(address)
    }
}
impl WriteData for IndirectYIndexed {
    fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
        let address = Self::address(cpu, memory);
        memory.write_u8(address, data)
    }
}

pub struct Relative;
impl Trait for Relative {
    type Operand = operand::Byte;
}
impl Relative {
    pub fn read_offset<M: Memory>(cpu: &Cpu, memory: &mut M) -> i8 {
        // read from 1 before the pc as this assumes that the pc has already
        // been advanced to past the instruction containing the relative address
        memory.read_u8(cpu.pc.wrapping_sub(1)) as i8
    }
}

pub struct XIndexedIndirect;
impl Trait for XIndexedIndirect {
    type Operand = operand::Byte;
}
impl XIndexedIndirect {
    fn address<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address {
        let offset = memory.read_u8(cpu.pc.wrapping_add(1));
        memory.read_u16_le(offset.wrapping_add(cpu.x) as Address)
    }
}
impl ReadData for XIndexedIndirect {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
        let address = Self::address(cpu, memory);
        memory.read_u8(address)
    }
}
impl WriteData for XIndexedIndirect {
    fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
        let address = Self::address(cpu, memory);
        memory.write_u8(address, data)
    }
}

pub struct ZeroPage;
impl Trait for ZeroPage {
    type Operand = operand::Byte;
}
impl ReadData for ZeroPage {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
        let address = memory.read_u8(cpu.pc.wrapping_add(1)) as Address;
        memory.read_u8(address)
    }
}
impl WriteData for ZeroPage {
    fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
        let address = memory.read_u8(cpu.pc.wrapping_add(1)) as Address;
        memory.write_u8(address, data)
    }
}

pub struct ZeroPageXIndexed;
impl Trait for ZeroPageXIndexed {
    type Operand = operand::Byte;
}
impl ReadData for ZeroPageXIndexed {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
        let base_address_lo = memory.read_u8(cpu.pc.wrapping_add(1));
        let address_lo = base_address_lo.wrapping_add(cpu.x);
        memory.read_u8(address_lo as Address)
    }
}
impl WriteData for ZeroPageXIndexed {
    fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
        let base_address_lo = memory.read_u8(cpu.pc.wrapping_add(1));
        let address_lo = base_address_lo.wrapping_add(cpu.x);
        memory.write_u8(address_lo as Address, data)
    }
}

pub struct ZeroPageYIndexed;
impl Trait for ZeroPageYIndexed {
    type Operand = operand::Byte;
}
impl ReadData for ZeroPageYIndexed {
    fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
        let base_address_lo = memory.read_u8(cpu.pc.wrapping_add(1));
        let address_lo = base_address_lo.wrapping_add(cpu.y);
        memory.read_u8(address_lo as Address)
    }
}
impl WriteData for ZeroPageYIndexed {
    fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
        let base_address_lo = memory.read_u8(cpu.pc.wrapping_add(1));
        let address_lo = base_address_lo.wrapping_add(cpu.y);
        memory.write_u8(address_lo as Address, data)
    }
}
