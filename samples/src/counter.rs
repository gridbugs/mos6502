/// Stores the numbers from 0..=0x42 in the first 43 addresses, and then the numbers
/// from 0x41..=0 in the next 42 addresses.
///
/// Instructions:
/// Lda
/// Ldx
/// Sta
/// Adc
/// Sbc
/// Inx
/// Cmp
/// Bne
/// Beq
/// Jmp
use crate::prelude::*;

pub struct Counter;
impl Sample for Counter {
    fn program(b: &mut Block) {
        b.inst(Lda(Immediate), 0);
        b.inst(Ldx(Immediate), 0);

        b.label("a");
        b.inst(Sta(ZeroPageXIndexed), 0);
        b.inst(Adc(Immediate), 1);
        b.inst(Inx, ());
        b.inst(Cmp(Immediate), 0x42 + 1);
        b.inst(Bne, LabelRelativeOffset("a"));

        b.inst(Sbc(Immediate), 2);

        b.label("b");
        b.inst(Sta(ZeroPageXIndexed), 0);
        b.inst(Inx, ());
        b.inst(Sbc(Immediate), 1);
        b.inst(Beq, LabelRelativeOffset("loop"));
        b.inst(Jmp(Absolute), "b");

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");
    }
    fn num_steps() -> usize {
        1000
    }
    fn check_result<M: MemoryReadOnly>(_cpu: &Cpu, m: &M) {
        for i in 0..0x42 {
            assert_eq!(m.read_u8_read_only(i as u16), i);
        }
        for i in 0..=0x42 {
            assert_eq!(m.read_u8_read_only(0x42 + i as u16), 0x42 - i);
        }
    }
}
