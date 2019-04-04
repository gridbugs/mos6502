/// Push the numbers 5, 6, 7, 8 to the stack in order, then pull twice, leaving the acculumator
/// containing 7. The top of the stack is at 0x01FF and it grows downwards.
///
/// Instructions:
/// - JMP (Absolute)
/// - LDA (Immediate)
/// - PHA
/// - PLA
use crate::prelude::*;

pub struct StackBasic;
impl Sample for StackBasic {
    fn program(b: &mut Block) {
        b.inst(Lda(Immediate), 5);
        b.inst(Pha, ());
        b.inst(Lda(Immediate), 6);
        b.inst(Pha, ());
        b.inst(Lda(Immediate), 7);
        b.inst(Pha, ());
        b.inst(Lda(Immediate), 8);
        b.inst(Pha, ());
        b.inst(Pla, ());
        b.inst(Pla, ());

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");
    }
    fn num_steps() -> usize {
        10
    }
    fn check_result<M: MemoryReadOnly>(cpu: &Cpu, m: &M) {
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.acc, 0x07);
        assert_eq!(m.read_u8_read_only(0x01FF), 5);
        assert_eq!(m.read_u8_read_only(0x01FE), 6);
        assert_eq!(m.read_u8_read_only(0x01FD), 7);
        assert_eq!(m.read_u8_read_only(0x01FC), 8);
    }
}
