/// Exercise instructions which operate directly on the contents of memory
///
/// Instructions:
/// INC
/// DEC
/// ASL
/// LSR
/// ROR
/// ROL
/// LDA
/// STA
/// JMP
use crate::prelude::*;

pub struct MemoryOperations;
impl Sample for MemoryOperations {
    fn program(b: &mut Block) {
        b.inst(Inc(ZeroPage), 0); // 1
        b.inst(Inc(ZeroPage), 0); // 2
        b.inst(Inc(ZeroPage), 0); // 3
        b.inst(Inc(ZeroPage), 0); // 4
        b.inst(Asl(ZeroPage), 0); // 8
        b.inst(Dec(ZeroPage), 0); // 7
        b.inst(Lda(ZeroPage), 0);
        b.inst(Sta(ZeroPage), 1);
        b.inst(Ror(ZeroPage), 1); // 3 (C=1)
        b.inst(Ror(ZeroPage), 1); // 0x81 (C=1)
        b.inst(Ror(ZeroPage), 1); // 0xC0 (C=1)
        b.inst(Lda(ZeroPage), 1);
        b.inst(Sta(ZeroPage), 2);
        b.inst(Rol(ZeroPage), 2); // 0x81 (C=1)
        b.inst(Lda(ZeroPage), 2);
        b.inst(Sta(ZeroPage), 3);
        b.inst(Lsr(ZeroPage), 3); // 0x40 (C=0)

        b.label("loop");
        b.inst(Jmp(Absolute), "loop");
    }
    fn num_steps() -> usize {
        100
    }
    fn check_result<M: MemoryReadOnly>(_cpu: &Cpu, m: &M) {
        assert_eq!(m.read_u8_read_only(0), 0x07);
        assert_eq!(m.read_u8_read_only(1), 0xC0);
        assert_eq!(m.read_u8_read_only(2), 0x81);
        assert_eq!(m.read_u8_read_only(3), 0x40);
    }
}
