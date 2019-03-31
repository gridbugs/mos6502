/// Push the numbers 5, 6, 7, 8 to the stack in order, then pull twice, leaving the acculumator
/// containing 7. The top of the stack is at 0x01FF and it grows downwards.
///
/// Instructions:
/// - JMP (Absolute)
/// - LDA (Immediate)
/// - PHA
/// - PLA
use nes_samples::single_block::*;

pub fn main() {
    with_block(|b: &mut Block| {
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
    });
}
