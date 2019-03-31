/// Load the value 0x42 into the accumulator, then loop forever
///
/// Instructions:
/// - JMP (Absolute)
/// - LDA (Immediate)
use nes_samples::single_block::*;

pub fn main() {
    with_block(|b: &mut Block| {
        b.inst(Lda(Immediate), 0x42);
        b.label("loop");
        b.inst(Jmp(Absolute), "loop");
    });
}
