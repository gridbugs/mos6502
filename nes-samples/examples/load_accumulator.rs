/// Load the value 0x42 into the accumulator, then loop forever
///
/// Instructions:
/// - JMP (Absolute)
/// - LDA (Immediate)
use nes_samples::single_block::*;

pub fn main() {
    with_block(|b: &mut Block| {
        b.label("start");
        b.inst(Lda(Immediate), 0x42u8);
        b.label("loop");
        b.inst(Jmp(Absolute), "loop");
        b.set_offset(INTERRUPT_VECTOR_START_PC_OFFSET);
        b.label_offset_le("start");
    });
}
