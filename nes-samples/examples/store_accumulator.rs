/// Store the numbers 5, 6, 7, 8 in the first 4 bytes of RAM (starting with address 0).
///
/// Instructions:
/// - JMP (Absolute)
/// - LDA (Immediate)
/// - STA (ZeroPage)
use nes_samples::single_block::*;

pub fn main() {
    with_block(|b: &mut Block| {
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
    });
}
