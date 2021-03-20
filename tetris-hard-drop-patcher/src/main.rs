use ines::Ines;
use mos6502_assembler::{Addr, Block, LabelRelativeOffset, LabelRelativeOffsetOwned};
use mos6502_model::Address;

const HINT_PATTERN_INDEX: u8 = 0xFA;
const SHAPE_TABLE: Address = 0x8A9C;
const ZP_PIECE_COORD_X: u8 = 0x40;
const ZP_PIECE_COORD_Y: u8 = 0x41;
const ZP_PIECE_SHAPE: u8 = 0x42;
const BOARD_TILES: Address = 0x0400;
const EMPTY_TILE: u8 = 0xEF;
const BOARD_HEIGHT: u8 = 20;
const CONTROLLER_STATE: u8 = 0xB6;
const CONTROLLER_BIT_UP: u8 = 0x08;

fn compute_hard_drop_distance(b: &mut Block, label: &str) {
    use mos6502_model::addressing_mode::*;
    use mos6502_model::assembler_instruction::*;

    b.label(label);

    // Multiply the shape by 12 to make an offset into the shape table, storing the result in X
    b.inst(Lda(ZeroPage), ZP_PIECE_SHAPE);
    b.inst(Clc, ());
    b.inst(Rol(Accumulator), ());
    b.inst(Rol(Accumulator), ());
    b.inst(Sta(ZeroPage), 0x20);
    b.inst(Rol(Accumulator), ());
    b.inst(Adc(ZeroPage), 0x20);
    b.inst(Tax, ());

    // Store absolute X,Y coords of each tile by reading relative coordinates from shape table
    // and adding the piece offset, storing the result in zero page 0x20..=0x27
    for i in 0..4 {
        b.inst(Lda(AbsoluteXIndexed), Addr(SHAPE_TABLE)); // 1 Y
        b.inst(Clc, ());
        b.inst(Adc(ZeroPage), ZP_PIECE_COORD_Y);
        b.inst(Sta(ZeroPage), 0x21 + (i * 2));
        b.inst(Inx, ()); // skip the tile index
        b.inst(Inx, ());
        b.inst(Lda(AbsoluteXIndexed), Addr(SHAPE_TABLE)); // 1 X
        b.inst(Clc, ());
        b.inst(Adc(ZeroPage), ZP_PIECE_COORD_X);
        b.inst(Sta(ZeroPage), 0x20 + (i * 2));
        b.inst(Inx, ());
    }

    b.inst(Ldx(Immediate), 0);
    b.label("start-hint-depth-loop");
    for i in 0..4 {
        // Increment the Y component of the coordinate
        b.inst(Inc(ZeroPage), 0x21 + (i * 2));

        // Check that we haven't gone off the bottom of the board
        b.inst(Lda(ZeroPage), 0x21 + (i * 2));
        b.inst(Cmp(Immediate), BOARD_HEIGHT);
        b.inst(Bpl, LabelRelativeOffset("end-hint-depth-loop"));

        // Multiply the Y component of the coordinate by 10 (the number of columns)
        b.inst(Asl(Accumulator), ());
        b.inst(Sta(ZeroPage), 0x28); // store Y * 2
        b.inst(Asl(Accumulator), ());
        b.inst(Asl(Accumulator), ()); // accumulator now contains Y * 8
        b.inst(Clc, ());
        b.inst(Adc(ZeroPage), 0x28); // accumulator now contains Y * 10

        // Now add the X component to get the row-major index of the cell
        b.inst(Adc(ZeroPage), 0x20 + (i * 2));

        // Load the tile at that coordinate
        b.inst(Tay, ());
        b.inst(Lda(AbsoluteYIndexed), BOARD_TILES);

        // Test whether the tile is empty, breaking out of the loop if it is not
        b.inst(Cmp(Immediate), EMPTY_TILE);
        b.inst(Bne, LabelRelativeOffset("end-hint-depth-loop"));
    }
    // Increment counter
    b.inst(Inx, ());
    b.inst(Jmp(Absolute), "start-hint-depth-loop");

    b.label("end-hint-depth-loop");

    // Return depth via accumulator
    b.inst(Txa, ());
    b.inst(Rts, ());
}

fn program_controls(b: &mut Block, label: &str, original_function_address: Address) {
    use mos6502_model::addressing_mode::*;
    use mos6502_model::assembler_instruction::*;

    b.label(label);

    b.inst(Jsr(Absolute), original_function_address);

    b.inst(Lda(ZeroPage), CONTROLLER_STATE);
    b.inst(And(Immediate), CONTROLLER_BIT_UP);
    b.inst(Beq, LabelRelativeOffset("controller-end"));

    b.inst(Jsr(Absolute), "compute-hard-drop-distance");

    // The distance will now be in the accumulator
    b.inst(Clc, ());
    b.inst(Adc(ZeroPage), ZP_PIECE_COORD_Y);
    b.inst(Sta(ZeroPage), ZP_PIECE_COORD_Y);

    b.inst(Lda(ZeroPage), 0xAF);
    b.inst(Sta(ZeroPage), 0x45);

    b.label("controller-end");

    b.inst(Rts, ());
}

fn program_oam_dma_page_update(b: &mut Block, label: &str, original_function_address: Address) {
    use mos6502_model::addressing_mode::*;
    use mos6502_model::assembler_instruction::*;

    b.label(label);

    b.inst(Jsr(Absolute), original_function_address);

    b.inst(Jsr(Absolute), "compute-hard-drop-distance");

    // The distance will now be in the accumulator
    b.inst(Beq, LabelRelativeOffset("after-render-hint"));
    b.inst(Sta(ZeroPage), 0x28);
    b.inst(Jsr(Absolute), "render-hint");
    b.label("after-render-hint");

    b.inst(Rts, ());

    program_render_hint(b, "render-hint");
}

fn program_render_hint(b: &mut Block, label: &str) {
    use mos6502_model::addressing_mode::*;
    use mos6502_model::assembler_instruction::*;
    b.label(label);
    b.inst(Lda(ZeroPage), 0x40); // X coord of current piece
    b.inst(Asl(Accumulator), ());
    b.inst(Asl(Accumulator), ());
    b.inst(Asl(Accumulator), ());
    b.inst(Adc(Immediate), 0x60);
    b.inst(Sta(ZeroPage), 0xAA);
    b.inst(Lda(ZeroPage), 0xBE);
    b.inst(Cmp(Immediate), 0x01);
    b.inst(Beq, LabelRelativeOffsetOwned(format!("{}-1", label)));
    b.inst(Lda(ZeroPage), 0xAA);
    b.inst(Sec, ());
    b.inst(Sbc(Immediate), ZP_PIECE_COORD_X);
    b.inst(Sta(ZeroPage), 0xAA);
    b.inst(Lda(ZeroPage), 0xB7);
    b.inst(Cmp(Immediate), 0x01);
    b.inst(Beq, LabelRelativeOffsetOwned(format!("{}-1", label)));
    b.inst(Lda(ZeroPage), 0xAA);
    b.inst(Adc(Immediate), 0x6F);
    b.inst(Sta(ZeroPage), 0xAA);
    b.label(format!("{}-1", label));
    b.inst(Clc, ());
    b.inst(Lda(ZeroPage), ZP_PIECE_COORD_Y); // Y coord of current piece
    b.inst(Adc(ZeroPage), 0x28); // add vertical offset
    b.inst(Rol(Accumulator), ());
    b.inst(Rol(Accumulator), ());
    b.inst(Rol(Accumulator), ());
    b.inst(Adc(Immediate), 0x2F);
    b.inst(Sta(ZeroPage), 0xAB);
    b.inst(Lda(ZeroPage), ZP_PIECE_SHAPE); // shape and rotation of current piece
    b.inst(Sta(ZeroPage), 0xAC);
    b.inst(Clc, ());
    b.inst(Lda(ZeroPage), 0xAC);
    b.inst(Rol(Accumulator), ());
    b.inst(Rol(Accumulator), ());
    b.inst(Sta(ZeroPage), 0xA8);
    b.inst(Rol(Accumulator), ());
    b.inst(Adc(ZeroPage), 0xA8);
    b.inst(Tax, ());
    b.inst(Ldy(ZeroPage), 0xB3);
    b.inst(Lda(Immediate), 0x04);
    b.inst(Sta(ZeroPage), 0xA9);
    b.label(format!("{}-3", label));
    b.inst(Lda(AbsoluteXIndexed), Addr(SHAPE_TABLE));
    b.inst(Asl(Accumulator), ());
    b.inst(Asl(Accumulator), ());
    b.inst(Asl(Accumulator), ());
    b.inst(Clc, ());
    b.inst(Adc(ZeroPage), 0xAB);
    b.inst(Sta(AbsoluteYIndexed), Addr(0x0200));
    b.inst(Sta(ZeroPage), 0xAE);
    b.inst(Inc(ZeroPage), 0xB3);
    b.inst(Iny, ());
    b.inst(Inx, ());
    b.inst(Lda(Immediate), HINT_PATTERN_INDEX);
    b.inst(Sta(AbsoluteYIndexed), Addr(0x0200));
    b.inst(Inc(ZeroPage), 0xB3);
    b.inst(Iny, ());
    b.inst(Inx, ());
    b.inst(Lda(Immediate), 0x02);
    b.inst(Sta(AbsoluteYIndexed), Addr(0x0200));
    b.inst(Lda(ZeroPage), 0xAE);
    b.inst(Cmp(Immediate), 0x2F);
    b.inst(Bcs, LabelRelativeOffsetOwned(format!("{}-2", label)));
    b.inst(Inc(ZeroPage), 0xB3);
    b.inst(Dey, ());
    b.inst(Lda(Immediate), 0xFF);
    b.inst(Sta(AbsoluteYIndexed), Addr(0x0200));
    b.inst(Iny, ());
    b.inst(Iny, ());
    b.inst(Lda(Immediate), 0x00);
    b.inst(Sta(AbsoluteYIndexed), Addr(0x0200));
    b.inst(Jmp(Absolute), format!("{}-jmp", label));
    b.label(format!("{}-2", label));
    b.inst(Inc(ZeroPage), 0xB3);
    b.inst(Iny, ());
    b.inst(Lda(AbsoluteXIndexed), Addr(SHAPE_TABLE));
    b.inst(Asl(Accumulator), ());
    b.inst(Asl(Accumulator), ());
    b.inst(Asl(Accumulator), ());
    b.inst(Clc, ());
    b.inst(Adc(ZeroPage), 0xAA);
    b.inst(Sta(AbsoluteYIndexed), Addr(0x0200));
    b.label(format!("{}-jmp", label));
    b.inst(Inc(ZeroPage), 0xB3);
    b.inst(Iny, ());
    b.inst(Inx, ());
    b.inst(Dec(ZeroPage), 0xA9);
    b.inst(Bne, LabelRelativeOffsetOwned(format!("{}-3", label)));
    b.inst(Rts, ());
}

fn add_hint_chr(ines: &mut Ines) {
    const PATTERN_TABLE_SIZE: usize = 0x1000;
    const WHICH_PATTERN_TABLE: usize = 3;
    const PATTERN_SIZE: usize = 16;
    const PATTERN_BYTE_INDEX: usize =
        (PATTERN_TABLE_SIZE * WHICH_PATTERN_TABLE) + (HINT_PATTERN_INDEX as usize * PATTERN_SIZE);
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
    const SIZE: usize = 512;
    const BASE: Address = 0xD6E0;
    const EXISTING_DMA_PAGE_UPDATE_FUNCTION: Address = 0x8A0A;
    const EXISTING_CONTROL_FUNCTION: Address = 0x89AE;
    compute_hard_drop_distance(&mut block, "compute-hard-drop-distance");
    program_oam_dma_page_update(
        &mut block,
        "oam-dma-page-update",
        EXISTING_DMA_PAGE_UPDATE_FUNCTION,
    );
    program_controls(&mut block, "controls", EXISTING_CONTROL_FUNCTION);
    let mut code_buffer = Vec::new();
    let assembled_block = block.assemble(BASE, SIZE, &mut code_buffer).unwrap();
    {
        let (code_to_replace, code_to_replace_with) = {
            use mos6502_model::addressing_mode::*;
            use mos6502_model::assembler_instruction::*;
            let mut code_to_replace_block = Block::new();
            code_to_replace_block.inst(Jsr(Absolute), EXISTING_DMA_PAGE_UPDATE_FUNCTION);
            let mut code_to_replace = Vec::new();
            code_to_replace_block
                .assemble(0, 3, &mut code_to_replace)
                .unwrap();
            let mut code_to_replace_with_block = Block::new();
            code_to_replace_with_block.inst(
                Jsr(Absolute),
                assembled_block
                    .address_of_label("oam-dma-page-update")
                    .unwrap(),
            );
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
    }
    {
        let (code_to_replace, code_to_replace_with) = {
            use mos6502_model::addressing_mode::*;
            use mos6502_model::assembler_instruction::*;
            let mut code_to_replace_block = Block::new();
            code_to_replace_block.inst(Jsr(Absolute), EXISTING_CONTROL_FUNCTION);
            let mut code_to_replace = Vec::new();
            code_to_replace_block
                .assemble(0, 3, &mut code_to_replace)
                .unwrap();
            let mut code_to_replace_with_block = Block::new();
            code_to_replace_with_block.inst(
                Jsr(Absolute),
                assembled_block.address_of_label("controls").unwrap(),
            );
            let mut code_to_replace_with = Vec::new();
            code_to_replace_with_block
                .assemble(0, 3, &mut code_to_replace_with)
                .unwrap();
            (code_to_replace, code_to_replace_with)
        };
        log::info!("Calls to redirect: {:X?}", code_to_replace);
        let addresses_of_calls_to_replace = vec![0x81CF];
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
    }
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
