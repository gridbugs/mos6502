pub mod jmp {
    pub mod absolute {
        pub const OPCODE: u8 = 0x4c;
        pub const NUM_BYTES: u8 = 3;
        pub const NUM_CYCLES: u8 = 3;
        pub struct I;
        impl crate::Instruction for I {
            type Operand = crate::operand::Address;
            fn num_bytes(&self) -> usize {
                NUM_BYTES as usize
            }
            fn opcode(&self) -> u8 {
                OPCODE
            }
            fn num_cycles(&self) -> usize {
                NUM_CYCLES as usize
            }
        }
    }
}

pub mod lda {
    pub mod immediate {
        pub const OPCODE: u8 = 0xa9;
        pub const NUM_BYTES: u8 = 2;
        pub const NUM_CYCLES: u8 = 2;
        pub struct I;
        impl crate::Instruction for I {
            type Operand = crate::operand::Byte;
            fn num_bytes(&self) -> usize {
                NUM_BYTES as usize
            }
            fn opcode(&self) -> u8 {
                OPCODE
            }
            fn num_cycles(&self) -> usize {
                NUM_CYCLES as usize
            }
        }
    }
}
