extern crate assembler;
extern crate ines;
extern crate mos6502;
extern crate nes;

use std::io::{self, Write};

pub use assembler::*;
pub use mos6502::*;
pub use nes::*;

use ines::*;

pub fn assemble_ines_file_to_stdout(assembler: &Assembler) {
    let mut prg_rom = Vec::new();
    assembler.partial_binary(nrom::PRG_START, ines::PRG_ROM_BLOCK_BYTES, &mut prg_rom);
    let ines = Ines {
        header: Header {
            num_prg_rom_blocks: 1,
            num_chr_rom_blocks: 0,
        },
        prg_rom,
        chr_rom: Vec::new(),
    };
    let mut output = Vec::new();
    ines.encode(&mut output);
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write(&output).expect("Failed to write output");
}

pub fn with_assembler<F: FnOnce(&mut Assembler)>(f: F) {
    let mut a = Assembler::new();
    f(&mut a);
    assemble_ines_file_to_stdout(&a);
}
