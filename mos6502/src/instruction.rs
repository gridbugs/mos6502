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

    pub struct Absolute;
    impl Trait for Absolute {
        type Operand = operand::Address;
    }
    impl ReadJumpTarget for Absolute {
        fn read_jump_target<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address {
            memory.read_u16_le(cpu.pc + 1)
        }
    }

    pub struct Indirect;
    impl Trait for Indirect {
        type Operand = operand::Address;
    }
    impl ReadJumpTarget for Indirect {
        fn read_jump_target<M: Memory>(cpu: &Cpu, memory: &mut M) -> Address {
            let address = memory.read_u16_le(cpu.pc + 1);
            memory.read_u16_le(address)
        }
    }

    pub struct Immediate;
    impl Trait for Immediate {
        type Operand = operand::Byte;
    }
    impl ReadData for Immediate {
        fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
            memory.read_u8(cpu.pc + 1)
        }
    }

    pub struct ZeroPage;
    impl Trait for ZeroPage {
        type Operand = operand::Byte;
    }
    impl ReadData for ZeroPage {
        fn read_data<M: Memory>(cpu: &Cpu, memory: &mut M) -> u8 {
            let address = memory.read_u8(cpu.pc + 1) as Address;
            memory.read_u8(address)
        }
    }
}

pub trait AssemblerInstruction {
    type AddressingMode: addressing_mode::Trait;
    fn opcode() -> u8;
}

pub mod opcode {
    pub mod jmp {
        pub const ABSOLUTE: u8 = 0x4C;
        pub const INDIRECT: u8 = 0x6C;
    }
    pub mod lda {
        pub const IMMEDIATE: u8 = 0xA9;
        pub const ZERO_PAGE: u8 = 0xA5;
        pub const ZERO_PAGE_X: u8 = 0xB5;
        pub const ABSOLUTE: u8 = 0xAD;
        pub const ABSOLUTE_X: u8 = 0xBD;
        pub const ABSOLUTE_Y: u8 = 0xB9;
        pub const INDIRECT_X: u8 = 0xA1;
        pub const INDIRECT_Y: u8 = 0xB1;
    }
}

pub mod instruction {
    use super::addressing_mode::*;
    use super::opcode;
    use super::*;
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
        pub struct Inst<A: AddressingMode>(pub A);
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
        pub fn interpret<A: AddressingMode, M: Memory>(_: A, cpu: &mut Cpu, memory: &mut M) {
            cpu.acc = A::read_data(cpu, memory);
            cpu.pc += A::instruction_bytes();
        }
    }
}

pub mod assembler_instruction {
    pub use super::addressing_mode::*;
    use super::instruction::*;
    pub use jmp::Inst as Jmp;
    pub use lda::Inst as Lda;
}
