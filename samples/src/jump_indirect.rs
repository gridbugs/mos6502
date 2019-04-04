/// Loop forever by repeatedly indirectly jumping, writing 0x42 to address 0 after the first
/// iteration. The program is padded with 0xFF bytes to make it easier to read the assembled cdoe.
///
/// Instructions:
/// - JMP (Absolute)
/// - JMP (Indirect)
/// - LDA (Immediate)
/// - STA (ZeroPage)
use crate::prelude::*;

pub struct JumpIndirect;
impl Sample for JumpIndirect {
    fn program(b: &mut Block) {
        b.inst(Jmp(Absolute), "start");
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.label("jump_target_location");
        b.label_offset_le("jump_target");
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.label("jump_target");
        b.inst(Lda(Immediate), 0x42);
        b.inst(Sta(ZeroPage), 0);
        b.inst(Jmp(Indirect), "jump_target_location");
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.literal_byte(0xFF);
        b.label("start");
        b.inst(Jmp(Indirect), "jump_target_location");
    }
    fn num_steps() -> usize {
        100
    }
    fn check_result<M: MemoryReadOnly>(_cpu: &Cpu, memory: &M) {
        assert_eq!(memory.read_u8_read_only(0), 0x42);
    }
}
