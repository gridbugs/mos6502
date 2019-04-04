/// Load the value 0x42 into the accumulator
///
/// Instructions:
/// - LDA (Immediate)
use crate::prelude::*;

pub struct LoadAccumulatorImmediate;
impl Sample for LoadAccumulatorImmediate {
    fn program(b: &mut Block) {
        b.inst(Lda(Immediate), 0x42);
    }
    fn num_steps() -> usize {
        1
    }
    fn check_result<M: MemoryReadOnly>(cpu: &Cpu, _memory: &M) {
        assert_eq!(cpu.acc, 0x42);
    }
}
