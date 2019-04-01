use crate::machine::*;
use crate::Address;

pub mod operand {
    pub trait Trait {
        fn instruction_bytes() -> u16;
    }

    pub struct Address;
    impl Trait for Address {
        fn instruction_bytes() -> u16 {
            3
        }
    }

    pub struct Byte;
    impl Trait for Byte {
        fn instruction_bytes() -> u16 {
            2
        }
    }

    pub struct None;
    impl Trait for None {
        fn instruction_bytes() -> u16 {
            1
        }
    }
}

pub mod addressing_mode {
    use super::operand;
    use super::{Address, Cpu, Memory};

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
    impl ReadData for AbsoluteXIndexed {
        fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
            let base_address = memory.read_u16_le(cpu.pc.wrapping_add(1));
            let address = base_address.wrapping_add(cpu.x as Address);
            memory.read_u8(address)
        }
    }
    impl WriteData for AbsoluteXIndexed {
        fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
            let base_address = memory.read_u16_le(cpu.pc.wrapping_add(1));
            let address = base_address.wrapping_add(cpu.x as Address);
            memory.write_u8(address, data)
        }
    }

    pub struct AbsoluteYIndexed;
    impl Trait for AbsoluteYIndexed {
        type Operand = operand::Address;
    }
    impl ReadData for AbsoluteYIndexed {
        fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
            let base_address = memory.read_u16_le(cpu.pc.wrapping_add(1));
            let address = base_address.wrapping_add(cpu.y as Address);
            memory.read_u8(address)
        }
    }
    impl WriteData for AbsoluteYIndexed {
        fn write_data<M: Memory>(cpu: &Cpu, memory: &mut M, data: u8) {
            let base_address = memory.read_u16_le(cpu.pc.wrapping_add(1));
            let address = base_address.wrapping_add(cpu.y as Address);
            memory.write_u8(address, data)
        }
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
            memory.read_u16_le(base_address) + cpu.y as Address
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
}

pub trait AssemblerInstruction {
    type AddressingMode: addressing_mode::Trait;
    fn opcode() -> u8;
}

pub mod opcode {
    pub mod clc {
        pub const IMPLIED: u8 = 0x18;
    }
    pub mod cld {
        pub const IMPLIED: u8 = 0xD8;
    }
    pub mod cli {
        pub const IMPLIED: u8 = 0x58;
    }
    pub mod jmp {
        pub const ABSOLUTE: u8 = 0x4C;
        pub const INDIRECT: u8 = 0x6C;
    }
    pub mod lda {
        pub const ABSOLUTE: u8 = 0xAD;
        pub const ABSOLUTE_X_INDEXED: u8 = 0xBD;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0xB9;
        pub const IMMEDIATE: u8 = 0xA9;
        pub const INDIRECT_Y_INDEXED: u8 = 0xB1;
        pub const X_INDEXED_INDIRECT: u8 = 0xA1;
        pub const ZERO_PAGE: u8 = 0xA5;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0xB5;
    }
    pub mod ldx {
        pub const IMMEDIATE: u8 = 0xA2;
    }
    pub mod ldy {
        pub const IMMEDIATE: u8 = 0xA0;
    }
    pub mod pha {
        pub const IMPLIED: u8 = 0x48;
    }
    pub mod php {
        pub const IMPLIED: u8 = 0x08;
    }
    pub mod pla {
        pub const IMPLIED: u8 = 0x68;
    }
    pub mod plp {
        pub const IMPLIED: u8 = 0x28;
    }
    pub mod sec {
        pub const IMPLIED: u8 = 0x38;
    }
    pub mod sed {
        pub const IMPLIED: u8 = 0xF8;
    }
    pub mod sei {
        pub const IMPLIED: u8 = 0x78;
    }
    pub mod sta {
        pub const ABSOLUTE: u8 = 0x8D;
        pub const ABSOLUTE_X_INDEXED: u8 = 0x9D;
        pub const ABSOLUTE_Y_INDEXED: u8 = 0x99;
        pub const INDIRECT_Y_INDEXED: u8 = 0x91;
        pub const X_INDEXED_INDIRECT: u8 = 0x81;
        pub const ZERO_PAGE: u8 = 0x85;
        pub const ZERO_PAGE_X_INDEXED: u8 = 0x95;
    }
}

pub mod instruction {
    use super::addressing_mode::*;
    use super::opcode;
    use super::*;
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
            cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
            cpu.status.set_zero_from_value(cpu.acc);
            cpu.status.set_negative_from_value(cpu.acc);
        }
    }
    pub mod ldx {
        use super::*;
        use opcode::ldx::*;
        pub trait AddressingMode: ReadData {}
        impl AddressingMode for Immediate {}
        pub struct Inst<A: AddressingMode>(pub A);
        impl AssemblerInstruction for Inst<Immediate> {
            type AddressingMode = Immediate;
            fn opcode() -> u8 {
                IMMEDIATE
            }
        }
        pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
            cpu.x = A::read_data(cpu, memory);
            cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
            cpu.status.set_zero_from_value(cpu.x);
            cpu.status.set_negative_from_value(cpu.x);
        }
    }
    pub mod ldy {
        use super::*;
        use opcode::ldy::*;
        pub trait AddressingMode: ReadData {}
        impl AddressingMode for Immediate {}
        pub struct Inst<A: AddressingMode>(pub A);
        impl AssemblerInstruction for Inst<Immediate> {
            type AddressingMode = Immediate;
            fn opcode() -> u8 {
                IMMEDIATE
            }
        }
        pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
            cpu.y = A::read_data(cpu, memory);
            cpu.pc = cpu.pc.wrapping_add(A::instruction_bytes());
            cpu.status.set_zero_from_value(cpu.y);
            cpu.status.set_negative_from_value(cpu.y);
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
}

pub mod assembler_instruction {
    pub use super::addressing_mode::*;
    use super::instruction::*;
    pub use clc::Inst as Clc;
    pub use cld::Inst as Cld;
    pub use cli::Inst as Cli;
    pub use jmp::Inst as Jmp;
    pub use lda::Inst as Lda;
    pub use ldx::Inst as Ldx;
    pub use ldy::Inst as Ldy;
    pub use pha::Inst as Pha;
    pub use php::Inst as Php;
    pub use pla::Inst as Pla;
    pub use plp::Inst as Plp;
    pub use sec::Inst as Sec;
    pub use sed::Inst as Sed;
    pub use sei::Inst as Sei;
    pub use sta::Inst as Sta;
}

pub mod debug {
    use crate::machine::UnknownOpcode;
    use crate::{Address, Cpu};
    use std::fmt;

    #[derive(Debug, Clone, Copy)]
    pub enum InstructionType {
        Clc,
        Cld,
        Cli,
        Jmp,
        Lda,
        Ldx,
        Ldy,
        Pha,
        Php,
        Pla,
        Plp,
        Sec,
        Sed,
        Sei,
        Sta,
    }
    #[derive(Debug, Clone, Copy)]
    pub enum AddressingMode {
        Absolute,
        AbsoluteXIndexed,
        AbsoluteYIndexed,
        Implied,
        Immediate,
        Indirect,
        IndirectYIndexed,
        XIndexedIndirect,
        ZeroPage,
        ZeroPageXIndexed,
    }
    impl AddressingMode {
        fn operand_bytes(self) -> usize {
            use AddressingMode::*;
            match self {
                Absolute => 2,
                AbsoluteXIndexed => 2,
                AbsoluteYIndexed => 2,
                Implied => 0,
                Immediate => 1,
                Indirect => 2,
                IndirectYIndexed => 1,
                XIndexedIndirect => 1,
                ZeroPage => 1,
                ZeroPageXIndexed => 1,
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
                opcode::clc::IMPLIED => (Clc, Implied),
                opcode::cld::IMPLIED => (Cld, Implied),
                opcode::cli::IMPLIED => (Cli, Implied),
                opcode::jmp::ABSOLUTE => (Jmp, Absolute),
                opcode::jmp::INDIRECT => (Jmp, Indirect),
                opcode::lda::ABSOLUTE => (Lda, Absolute),
                opcode::lda::ABSOLUTE_X_INDEXED => (Lda, AbsoluteXIndexed),
                opcode::lda::ABSOLUTE_Y_INDEXED => (Lda, AbsoluteYIndexed),
                opcode::lda::IMMEDIATE => (Lda, Immediate),
                opcode::lda::INDIRECT_Y_INDEXED => (Lda, IndirectYIndexed),
                opcode::lda::X_INDEXED_INDIRECT => (Lda, XIndexedIndirect),
                opcode::lda::ZERO_PAGE => (Lda, ZeroPage),
                opcode::lda::ZERO_PAGE_X_INDEXED => (Lda, ZeroPageXIndexed),
                opcode::ldx::IMMEDIATE => (Ldx, Immediate),
                opcode::ldy::IMMEDIATE => (Ldy, Immediate),
                opcode::pha::IMPLIED => (Pha, Implied),
                opcode::php::IMPLIED => (Php, Implied),
                opcode::pla::IMPLIED => (Pla, Implied),
                opcode::plp::IMPLIED => (Plp, Implied),
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
    pub trait MemoryDebug {
        fn read_u8_debug(&self, address: Address) -> u8;
    }
    impl InstructionWithOperand {
        pub fn next<M: MemoryDebug>(cpu: &Cpu, memory: &M) -> Result<Self, UnknownOpcode> {
            let opcode = memory.read_u8_debug(cpu.pc);
            let instruction = Instruction::from_opcode(opcode)?;
            let operand_bytes = instruction.addressing_mode.operand_bytes();
            let mut operand = Vec::new();
            for i in 0..operand_bytes {
                operand
                    .push(memory.read_u8_debug(cpu.pc.wrapping_add(i as Address).wrapping_add(1)));
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
}
