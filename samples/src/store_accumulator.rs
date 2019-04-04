/// Store the numbers 5, 6, 7, 8 in the first 4 bytes of RAM (starting with address 0).
///
/// Instructions:
/// - JMP (Absolute)
/// - LDA (Immediate)
/// - STA (ZeroPage)
use crate::prelude::*;

pub struct StoreAccumulator;
impl Sample for StoreAccumulator {
    fn program(b: &mut Block) {
        b.inst(Lda(Immediate), 5);
        b.inst(Sta(ZeroPage), 0x00);

        b.inst(Lda(Immediate), 6);
        b.inst(Sta(ZeroPage), 0x01);

        b.inst(Lda(Immediate), 7);
        b.inst(Sta(ZeroPage), 0x02);

        b.inst(Lda(Immediate), 8);
        b.inst(Sta(ZeroPage), 0x03);

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");
    }
    fn num_steps() -> usize {
        10
    }
    fn check_result<M: MemoryReadOnly>(_cpu: &Cpu, m: &M) {
        assert_eq!(m.read_u8_read_only(0), 5);
        assert_eq!(m.read_u8_read_only(1), 6);
        assert_eq!(m.read_u8_read_only(2), 7);
        assert_eq!(m.read_u8_read_only(3), 8);
    }
}
