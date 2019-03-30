/// Loop forever, keeping the program counter at 0xc000
use nes_samples::single_block::*;

pub fn main() {
    with_block(|b: &mut Block| {
        b.label("start");
        b.inst(jmp::absolute::I, "start");
        b.set_offset(INTERRUPT_VECTOR_START_PC_OFFSET);
        b.label_offset_le("start");
    });
}
