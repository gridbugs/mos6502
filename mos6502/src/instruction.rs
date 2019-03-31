pub mod operand {
    pub trait Operand {
        fn instruction_bytes() -> u16;
    }

    pub struct Address;
    impl Operand for Address {
        fn instruction_bytes() -> u16 {
            3
        }
    }

    pub struct Byte;
    impl Operand for Byte {
        fn instruction_bytes() -> u16 {
            2
        }
    }
}

pub mod addressing_mode {
    use super::operand::*;

    pub trait AddressingMode {
        type Operand: Operand;
        fn instruction_bytes() -> u16 {
            <Self::Operand as Operand>::instruction_bytes()
        }
    }

    pub struct Immediate;
    impl AddressingMode for Immediate {
        type Operand = Byte;
    }

    pub struct Absolute;
    impl AddressingMode for Absolute {
        type Operand = Address;
    }

    pub struct Indirect;
    impl AddressingMode for Indirect {
        type Operand = Address;
    }
}

use crate::machine::*;
use addressing_mode::*;

pub trait Instruction {
    type AddressingMode: AddressingMode;
    fn instruction_bytes() -> u16 {
        <Self::AddressingMode as AddressingMode>::instruction_bytes()
    }
    fn opcode() -> u8;
    fn interpret<D: MemoryMappedDevices>(cpu: &mut Cpu, devices: &mut D);
}

pub fn interpret<I: Instruction, D: MemoryMappedDevices>(cpu: &mut Cpu, devices: &mut D) {
    I::interpret(cpu, devices);
}

pub mod opcode {
    pub mod jmp {
        pub const ABSOLUTE: u8 = 0x4c;
        pub const INDIRECT: u8 = 0x6c;
    }
    pub mod lda {
        pub const IMMEDIATE: u8 = 0xa9;
    }
}

pub struct Jmp<M>(pub M);
impl Instruction for Jmp<Absolute> {
    type AddressingMode = Absolute;
    fn opcode() -> u8 {
        opcode::jmp::ABSOLUTE
    }
    fn interpret<D: MemoryMappedDevices>(cpu: &mut Cpu, devices: &mut D) {
        cpu.pc = devices.read_u16_le(cpu.pc + 1);
    }
}
impl Instruction for Jmp<Indirect> {
    type AddressingMode = Indirect;
    fn opcode() -> u8 {
        opcode::jmp::INDIRECT
    }
    fn interpret<D: MemoryMappedDevices>(cpu: &mut Cpu, devices: &mut D) {
        let address = devices.read_u16_le(cpu.pc + 1);
        cpu.pc = devices.read_u16_le(address);
    }
}

pub struct Lda<M>(pub M);
impl Instruction for Lda<Immediate> {
    type AddressingMode = Immediate;
    fn opcode() -> u8 {
        opcode::lda::IMMEDIATE
    }
    fn interpret<D: MemoryMappedDevices>(cpu: &mut Cpu, devices: &mut D) {
        cpu.acc = devices.read_u8(cpu.pc + 1);
        cpu.pc += Self::instruction_bytes();
    }
}
