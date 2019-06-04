extern crate assembler;
extern crate ines;
extern crate mos6502;
extern crate nes;
extern crate samples;

pub mod single_block {
    pub use addressing_mode::*;
    pub use assembler::*;
    pub use assembler_instruction::*;
    pub use mos6502::*;
    pub use nes::*;
    pub use samples::*;
    use std::io::{self, Write};

    use ines::*;

    pub const PRG_START: Address = nrom::PRG_START_HI;
    pub const INTERRUPT_VECTOR_START_PC_OFFSET: Address = interrupt_vector::START_LO - PRG_START;

    pub fn assemble_ines_file_to_stdout(block: &Block) {
        let mut prg_rom = Vec::new();
        block
            .assemble(nrom::PRG_START_HI, ines::PRG_ROM_BLOCK_BYTES, &mut prg_rom)
            .expect("Failed to assemble");
        let ines = Ines {
            header: Header {
                num_prg_rom_blocks: 1,
                num_chr_rom_blocks: 0,
                mapper: Mapper::Nrom,
                mirroring: Mirroring::Vertical,
                four_screen_vram: false,
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

    pub fn with_block<F: FnOnce(&mut Block)>(f: F) {
        let mut b = Block::new();
        f(&mut b);
        b.set_offset(INTERRUPT_VECTOR_START_PC_OFFSET);
        b.literal_offset_le(0);
        assemble_ines_file_to_stdout(&b);
    }

    pub fn with_sample<S: Sample>(_: S) {
        with_block(|b| S::program(b));
    }
}
