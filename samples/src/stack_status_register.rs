/// Repeatedly push the status register to the stack, after performing operations to change its
/// contents. Finally pushes -1 (0xFF) to make it easy to spot the end of the sequence of pushed
/// values when inspecting memory.
///
/// Instructions:
/// - JMP (Absolute)
/// - LDA (Immediate)
/// - PHP
/// - CLI
/// - SEI
/// - SEC
/// - SED
/// - PHP
/// - PHA
use crate::prelude::*;

pub struct StackStatusRegister;
impl Sample for StackStatusRegister {
    fn program(b: &mut Block) {
        b.inst(Cli, ());
        b.inst(Php, ());
        b.inst(Sei, ());
        b.inst(Php, ());
        b.inst(Sec, ());
        b.inst(Php, ());
        b.inst(Sed, ());
        b.inst(Php, ());
        b.inst(Lda(Immediate), 0);
        b.inst(Php, ());
        b.inst(Clc, ());
        b.inst(Php, ());
        b.inst(Lda(Immediate), -1);
        b.inst(Php, ());
        b.inst(Pha, ());

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");
    }
    fn num_steps() -> usize {
        100
    }
    fn check_result<M: MemoryReadOnly>(_cpu: &Cpu, m: &M) {
        use status::flag::*;
        let mask = EXPANSION | BRK;
        assert_eq!(m.read_u8_read_only(0x01FF), 0x00 | mask);
        assert_eq!(m.read_u8_read_only(0x01FE), 0x04 | mask);
        assert_eq!(m.read_u8_read_only(0x01FD), 0x05 | mask);
        assert_eq!(m.read_u8_read_only(0x01FC), 0x0D | mask);
        assert_eq!(m.read_u8_read_only(0x01FB), 0x0F | mask);
        assert_eq!(m.read_u8_read_only(0x01FA), 0x0E | mask);
        assert_eq!(m.read_u8_read_only(0x01F9), 0x8C | mask);
        assert_eq!(m.read_u8_read_only(0x01F8), 0xFF | mask);
    }
}
