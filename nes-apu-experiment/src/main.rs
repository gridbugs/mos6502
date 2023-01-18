use ines::Ines;
use mos6502_assembler::{Addr, Block, LabelRelativeOffset, LabelRelativeOffsetOwned};
use mos6502_model::{interrupt_vector, Address};

const PRG_START: Address = 0xC000;
const INTERRUPT_VECTOR_START_PC_OFFSET: Address = interrupt_vector::START_LO - PRG_START;
const INTERRUPT_VECTOR_NMI_OFFSET: Address = interrupt_vector::NMI_LO - PRG_START;

fn program(b: &mut Block) {
    use mos6502_model::addressing_mode::*;
    use mos6502_model::assembler_instruction::*;

    b.label("reset");

    b.inst(Sei, ()); // Enable interrupts
    b.inst(Cld, ()); // Disable decimal mode

    b.inst(Ldx(Immediate), 0x40);
    b.inst(Stx(Absolute), Addr(0x4017)); // disable APU frame IRQ

    b.inst(Ldx(Immediate), 0xFF);
    b.inst(Tsx, ()); // initialize stack

    b.inst(Ldx(Immediate), 0x00);
    b.inst(Stx(Absolute), Addr(0x2000)); // disable vblank nmi
    b.inst(Stx(Absolute), Addr(0x2001)); // disable rendering
    b.inst(Stx(Absolute), Addr(0x4010)); // disable DMC IRQs

    b.inst(Bit(Absolute), Addr(0x2002)); // read ppu status to clear vblank

    // wait for 2 vblanks to occur to make sure ppu has stabilized

    b.label("vblankwait1");
    b.inst(Bit(Absolute), Addr(0x2002));
    b.inst(Bpl, LabelRelativeOffset("vblankwait1"));

    b.label("vblankwait2");
    b.inst(Bit(Absolute), Addr(0x2002));
    b.inst(Bpl, LabelRelativeOffset("vblankwait2"));

    // set up palette
    let universal_background = 0x0d;
    let colours = [
        0x01, 0x31, 0x33, 0xFF, /* 4th is ignored */
        0x04, 0x34, 0x36, 0xFF, /* 8th is ignored */
        0x08, 0x38, 0x39, 0xFF, /* 12th is ignored */
        0x09, 0x39, 0x3c,
    ];
    b.inst(Bit(Absolute), Addr(0x2002)); // read ppu status to clear address latch

    b.inst(Sta(Absolute), Addr(0x2001)); // turn on background and left-background

    // write address of the palette in video memory (0x3F00) to the PPU address register
    b.inst(Lda(Immediate), 0x3F);
    b.inst(Sta(Absolute), Addr(0x2006)); // write high byte of 0x3F00
    b.inst(Lda(Immediate), 0x00);
    b.inst(Sta(Absolute), Addr(0x2006)); // write low byte of 0x3F00

    // now write the colours (the address increments automatically)
    b.inst(Lda(Immediate), universal_background);
    b.inst(Sta(Absolute), Addr(0x2007));
    for &c in &colours {
        b.inst(Lda(Immediate), c);
        b.inst(Sta(Absolute), Addr(0x2007));
    }

    // done setting up the palette

    // enable rendering
    b.inst(Lda(Immediate), 0b00001010);
    b.inst(Sta(Absolute), Addr(0x2001)); // turn on background and left-background

    b.label("mainloop");

    b.label("vblankmain");
    b.inst(Bit(Absolute), Addr(0x2002));
    b.inst(Bpl, LabelRelativeOffset("vblankmain"));

    b.inst(Jsr(Absolute), "controller-to-0");

    // draw some debugging tiles based on controller input
    b.inst(Lda(ZeroPage), 0);
    b.inst(Sta(ZeroPage), 1);
    fn controll_debug(b: &mut Block, name: &str, name_table_address: u16) {
        b.inst(Ldx(Immediate), 1);
        b.inst(Ror(ZeroPage), 1);
        b.inst(
            Bcc,
            LabelRelativeOffsetOwned(format!("controller-debug-dpad-{}", name)),
        );
        b.inst(Ldx(Immediate), 2);
        b.label(format!("controller-debug-dpad-{}", name));

        b.inst(Lda(Immediate), (name_table_address >> 8) as u8);
        b.inst(Sta(Absolute), Addr(0x2006));
        b.inst(Lda(Immediate), name_table_address as u8);
        b.inst(Sta(Absolute), Addr(0x2006));
        b.inst(Stx(Absolute), Addr(0x2007));
    }
    controll_debug(b, "right", 0x21CA);
    controll_debug(b, "left", 0x21C8);
    controll_debug(b, "down", 0x21E9);
    controll_debug(b, "up", 0x21A9);
    controll_debug(b, "start", 0x21AD);
    controll_debug(b, "select", 0x21AC);
    controll_debug(b, "b", 0x21D0);
    controll_debug(b, "a", 0x21CF);

    // end of debugging tiles

    b.inst(Lda(Immediate), 1 << 7);
    b.inst(Bit(ZeroPage), 0);
    b.inst(Beq, LabelRelativeOffset("end-of-audio"));

    // enable triangle channel
    b.inst(Lda(Immediate), 1 << 2);
    b.inst(Sta(Absolute), Addr(0x4015));
    b.inst(Lda(Immediate), 0x7F);
    b.inst(Sta(Absolute), Addr(0x4008));
    b.inst(Lda(Immediate), 0xFF);
    b.inst(Sta(Absolute), Addr(0x400A));
    b.inst(Lda(Immediate), 0x1E << 3);
    b.inst(Sta(Absolute), Addr(0x400B));

    b.label("end-of-audio");

    // fix scroll

    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x2005));
    b.inst(Sta(Absolute), Addr(0x2005)); // fix scroll

    b.inst(Jmp(Absolute), "mainloop");

    // start of function
    b.label("controller-to-0");
    const CONTROLLER_REG: Addr = Addr(0x4016);

    // toggle the controller strobe bit to copy its current value into shift register
    b.inst(Lda(Immediate), 1);
    b.inst(Sta(Absolute), CONTROLLER_REG); // set controller strobe
    b.inst(Sta(ZeroPage), 0); // store a 1 at 0 - used to check when all bits are read
    b.inst(Lsr(Accumulator), ()); // clear accumulator
    b.inst(Sta(Absolute), CONTROLLER_REG); // clear controller strobe

    // shift each of the 8 bits of controller state from the shift register into address 0
    b.label("controller-to-0-loop");
    b.inst(Lda(Absolute), CONTROLLER_REG); // load single bit into LBS of acculumator
    b.inst(Lsr(Accumulator), ()); // shift bit into carry flag
    b.inst(Rol(ZeroPage), 0); // shift carry flag into 0, and MSB of 0 into carry flag

    // if that set the carry flag, this was the 8th iteration
    b.inst(Bcc, LabelRelativeOffset("controller-to-0-loop"));

    b.inst(Rts, ());
    // end of function

    b.label("nmi");
    b.inst(Rti, ());

    b.set_offset(INTERRUPT_VECTOR_START_PC_OFFSET);
    b.label_offset_le("reset");
    b.set_offset(INTERRUPT_VECTOR_NMI_OFFSET);
    b.label_offset_le("nmi");
}

fn chr_rom() -> Vec<u8> {
    let mut chr_rom = vec![0; ines::CHR_ROM_BLOCK_BYTES];
    // Leave tile 0 as all 0s to make it transparent.
    // Tile 1 will be all 1s
    let tile_1_byte_index = 16 * 1;
    for row in 0..8 {
        chr_rom[tile_1_byte_index + row] = 0xFF;
    }
    // Tile 2 will be all 2s
    let tile_2_byte_index = 16 * 2;
    for row in 0..8 {
        // add 8 because we're updating the second pane
        chr_rom[8 + tile_2_byte_index + row] = 0xFF;
    }
    // Tile 3 will be all 3s
    let tile_3_byte_index = 16 * 3;
    for row in 0..16 {
        chr_rom[tile_3_byte_index + row] = 0xFF;
    }
    chr_rom
}

fn prg_rom() -> Vec<u8> {
    let mut block = Block::new();
    program(&mut block);
    let mut prg_rom = Vec::new();
    block
        .assemble(PRG_START, ines::PRG_ROM_BLOCK_BYTES, &mut prg_rom)
        .expect("Failed to assemble");
    prg_rom
}

fn main() {
    use std::io::Write;
    let ines = Ines {
        header: ines::Header {
            num_prg_rom_blocks: 1,
            num_chr_rom_blocks: 1,
            mapper: ines::Mapper::Nrom,
            mirroring: ines::Mirroring::Vertical,
            four_screen_vram: false,
        },
        prg_rom: prg_rom(),
        chr_rom: chr_rom(),
    };
    let mut encoded = Vec::new();
    ines.encode(&mut encoded);
    std::io::stdout()
        .lock()
        .write_all(&encoded)
        .expect("Failed to write encoded rom to stdout");
}
