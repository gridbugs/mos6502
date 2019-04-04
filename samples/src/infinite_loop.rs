/// Loop forever, keeping the program counter at 0xc000
///
/// Instructions:
/// - JMP (Absolute)
use crate::prelude::*;

pub struct InfiniteLoop;
impl Sample for InfiniteLoop {
    fn program(b: &mut Block) {
        b.label("start");
        b.inst(Jmp(Absolute), "start");
    }
    fn num_steps() -> usize {
        10
    }
    fn check_result<M: MemoryReadOnly>(cpu: &Cpu, _memory: &M) {
        assert_eq!(cpu.pc, PRG_START);
    }
}
