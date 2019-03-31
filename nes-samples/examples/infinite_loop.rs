/// Loop forever, keeping the program counter at 0xc000
///
/// Instructions:
/// - JMP (Absolute)
use nes_samples::single_block::*;

pub fn main() {
    with_block(|b: &mut Block| {
        b.label("start");
        b.inst(Jmp(Absolute), "start");
    });
}
