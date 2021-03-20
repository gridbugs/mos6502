use ines::Ines;
use mos6502_assembler::{Addr, Block, LabelRelativeOffset, LabelRelativeOffsetOwned};
use mos6502_model::{address, interrupt_vector, Address};

const HINT_PATTERN_INDEX: usize = 0xFA;

fn program_oam_dma_page_update(b: &mut Block, original_function_address: Address) {
    use mos6502_model::addressing_mode::*;
    use mos6502_model::assembler_instruction::*;
    b.inst(Jsr(Absolute), original_function_address);
    b.inst(Ldx(Immediate), 32);
    b.inst(Lda(Immediate), 0x80);
    b.inst(Sta(AbsoluteXIndexed), Addr(0x200));
    b.inst(Inx, ());
    b.inst(Lda(Immediate), 0xFA);
    b.inst(Sta(AbsoluteXIndexed), Addr(0x200));
    b.inst(Inx, ());
    b.inst(Lda(Immediate), 0x02);
    b.inst(Sta(AbsoluteXIndexed), Addr(0x200));
    b.inst(Inx, ());
    b.inst(Lda(Immediate), 0x80);
    b.inst(Sta(AbsoluteXIndexed), Addr(0x200));
    b.inst(Inx, ());
    b.inst(Lda(Immediate), 0x80);
    b.inst(Sta(AbsoluteXIndexed), Addr(0x200));
    b.inst(Inx, ());
    b.inst(Lda(Immediate), 0xFA);
    b.inst(Sta(AbsoluteXIndexed), Addr(0x200));
    b.inst(Inx, ());
    b.inst(Lda(Immediate), 0x02);
    b.inst(Sta(AbsoluteXIndexed), Addr(0x200));
    b.inst(Inx, ());
    b.inst(Lda(Immediate), 0x88);
    b.inst(Sta(AbsoluteXIndexed), Addr(0x200));
    b.inst(Inx, ());
    b.inst(Rts, ());
}

fn add_hint_chr(ines: &mut Ines) {
    const PATTERN_TABLE_SIZE: usize = 0x1000;
    const WHICH_PATTERN_TABLE: usize = 3;
    const PATTERN_SIZE: usize = 16;
    const PATTERN_BYTE_INDEX: usize =
        (PATTERN_TABLE_SIZE * WHICH_PATTERN_TABLE) + (HINT_PATTERN_INDEX * PATTERN_SIZE);
    let chr_slice = &mut ines.chr_rom[PATTERN_BYTE_INDEX..(PATTERN_BYTE_INDEX + PATTERN_SIZE)];
    chr_slice[0] = 0b10101010;
    chr_slice[1] = 0b00000000;
    chr_slice[2] = 0b10000010;
    chr_slice[3] = 0b00000000;
    chr_slice[4] = 0b10000010;
    chr_slice[5] = 0b00000000;
    chr_slice[6] = 0b10101010;
    chr_slice[7] = 0b00000000;
}

fn modify_rom(ines: &mut Ines) {
    let mut block = Block::new();
    const SIZE: usize = 256;
    const BASE: Address = 0xD6E0;
    const FUNCTION_TO_REDIRECT: Address = 0x8A0A;
    let (code_to_replace, code_to_replace_with) = {
        use mos6502_model::addressing_mode::*;
        use mos6502_model::assembler_instruction::*;
        let mut code_to_replace_block = Block::new();
        code_to_replace_block.inst(Jsr(Absolute), FUNCTION_TO_REDIRECT);
        let mut code_to_replace = Vec::new();
        code_to_replace_block
            .assemble(0, 3, &mut code_to_replace)
            .unwrap();
        let mut code_to_replace_with_block = Block::new();
        code_to_replace_with_block.inst(Jsr(Absolute), BASE);
        let mut code_to_replace_with = Vec::new();
        code_to_replace_with_block
            .assemble(0, 3, &mut code_to_replace_with)
            .unwrap();
        (code_to_replace, code_to_replace_with)
    };
    log::info!("Calls to redirect: {:X?}", code_to_replace);
    let addresses_of_calls_to_replace = vec![0x8192, 0x817A];
    for &address in &addresses_of_calls_to_replace {
        let base = address as usize - 0x8000;
        assert!(&ines.prg_rom[base..(base + code_to_replace.len())] == &code_to_replace);
        &mut ines.prg_rom[base..(base + code_to_replace.len())]
            .copy_from_slice(&code_to_replace_with);
        log::info!(
            "Replacing call at 0x{:X} with {:X?}",
            address,
            code_to_replace_with
        );
    }
    program_oam_dma_page_update(&mut block, FUNCTION_TO_REDIRECT);
    let mut code_buffer = Vec::new();
    block.assemble(BASE, SIZE, &mut code_buffer).unwrap();
    let prg_start = BASE as usize - 0x8000;
    let to_replace_slice = &mut ines.prg_rom[prg_start..(prg_start + SIZE)];
    to_replace_slice.copy_from_slice(&code_buffer);
    add_hint_chr(ines);
}

fn main() {
    use std::io::{self, Read, Write};
    env_logger::init();
    let mut buffer = Vec::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle
        .read_to_end(&mut buffer)
        .expect("Failed to read rom from stdin");
    let mut ines = Ines::parse(&buffer).unwrap();
    modify_rom(&mut ines);
    let mut encoded = Vec::new();
    ines.encode(&mut encoded);
    std::io::stdout()
        .lock()
        .write_all(&encoded)
        .expect("Failed to write encoded rom to stdout");
}
