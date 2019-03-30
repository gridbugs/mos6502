use nes_samples::*;

pub fn main() {
    with_assembler(|a| {
        a.org(nrom::PRG_START);
        a.org(START_PC_LO)
    });
}
