/// Repeatedly push the status register to the stack, after performing operations to change its
/// contents. Finally pushes -1 (0xFF) to make it easy to spot the end of the sequence of pushed
/// values when inspecting memory.
///
/// Instructions:
/// - JMP (Absolute)
/// - LDA (Immediate)
/// - PHP
/// - SEI
/// - SEC
/// - SED
/// - PHP
/// - PHA
use nes_samples::single_block::*;

pub fn main() {
    with_block(|b: &mut Block| {
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
    });
}
