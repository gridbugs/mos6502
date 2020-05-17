use assembler::{Addr, Block, LabelRelativeOffset};
use ines::Ines;
use mos6502::{interrupt_vector, Address};

pub const PRG_START: Address = 0xC000;
pub const INTERRUPT_VECTOR_START_PC_OFFSET: Address = interrupt_vector::START_LO - PRG_START;
pub const INTERRUPT_VECTOR_NMI_OFFSET: Address = interrupt_vector::NMI_LO - PRG_START;

fn program(b: &mut Block) {
    use mos6502::addressing_mode::*;
    use mos6502::assembler_instruction::*;

    b.label("reset");

    b.inst(Sei, ());
    b.inst(Cld, ());

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
    let universal_background = 0x2C;
    let background_colour_1 = 0x14;
    b.inst(Bit(Absolute), Addr(0x2002)); // read ppu status to clear address latch
    b.inst(Lda(Immediate), 0x3F);
    b.inst(Sta(Absolute), Addr(0x2006)); // write high byte of 0x3F00
    b.inst(Lda(Immediate), 0x00);
    b.inst(Sta(Absolute), Addr(0x2006)); // write low byte of 0x3F00
    b.inst(Lda(Immediate), universal_background);
    b.inst(Sta(Absolute), Addr(0x2007));
    b.inst(Lda(Immediate), background_colour_1);
    b.inst(Sta(Absolute), Addr(0x2007));

    // initialize state
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(ZeroPage), 0);
    b.inst(Lda(Immediate), 120);
    b.inst(Sta(ZeroPage), 1);
    for i in 0..30 {
        if i % 2 == 0 {
            b.inst(Lda(Immediate), 0x55);
        } else {
            b.inst(Lda(Immediate), 0xAA);
        }
        for j in 0..4 {
            b.inst(Sta(ZeroPage), i * 4 + j + 2);
        }
    }
    b.inst(Lda(Immediate), 60);
    b.inst(Sta(ZeroPage), 122);
    b.inst(Lda(Immediate), 60);
    b.inst(Sta(ZeroPage), 123);
    b.inst(Lda(Immediate), 0x55);
    for i in 0..15 {
        b.inst(Lda(Immediate), 0xFF);
        for j in 0..4 {
            b.inst(Sta(ZeroPage), i * 4 + j + 124);
        }
    }
    b.inst(Lda(Immediate), 0xFF);
    b.inst(Sta(ZeroPage), 184);

    // enable rendering
    b.inst(Lda(Immediate), 0b00001010);
    b.inst(Sta(Absolute), Addr(0x2001)); // turn on background and left-background

    b.label("mainloop");

    b.label("vblankmain");
    b.inst(Bit(Absolute), Addr(0x2002));
    b.inst(Bpl, LabelRelativeOffset("vblankmain"));

    // update ppu memory
    b.inst(Bit(Absolute), Addr(0x2002)); // read ppu status to clear address latch

    let gas = 24;
    b.inst(Lda(Immediate), gas);
    b.inst(Sta(ZeroPage), 255); // store gas

    b.inst(Ldx(Immediate), 0); // initialize x register

    b.label("update-ppu-start");
    b.inst(Lda(ZeroPageXIndexed), 0);
    b.inst(Bmi, LabelRelativeOffset("update-ppu-end"));

    b.inst(Sta(ZeroPage), 254);

    b.inst(Lda(Immediate), 0x04);
    b.inst(Asl(ZeroPageXIndexed), 0);
    b.inst(Rol(Accumulator), ());
    b.inst(Asl(ZeroPageXIndexed), 0);
    b.inst(Rol(Accumulator), ());
    b.inst(Asl(ZeroPageXIndexed), 0);
    b.inst(Rol(Accumulator), ());

    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Lda(ZeroPageXIndexed), 0);
    b.inst(Sta(Absolute), Addr(0x2006)); // set ppu addr for current run

    b.inst(Inx, ());
    b.inst(Ldy(ZeroPageXIndexed), 0); // read length (bytes) of run into y register

    b.label("byte-run-start");
    b.inst(Dey, ());
    b.inst(Bmi, LabelRelativeOffset("byte-run-end"));

    b.inst(Dec(ZeroPage), 255); // spend gas
    b.inst(Bne, LabelRelativeOffset("post-vblank-wait"));
    b.inst(Lda(Immediate), gas);
    b.inst(Sta(ZeroPage), 255); // reset gas

    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x2005));
    b.inst(Sta(Absolute), Addr(0x2005)); // fix scroll

    b.label("vblankmain-gas");
    b.inst(Bit(Absolute), Addr(0x2002));
    b.inst(Bpl, LabelRelativeOffset("vblankmain-gas"));

    b.inst(Bit(Absolute), Addr(0x2002)); // read ppu status to clear address latch

    b.inst(Lda(ZeroPage), 254);
    b.inst(Pha, ()); // back up offset
    b.inst(Lda(Immediate), 0x04);
    b.inst(Asl(ZeroPage), 254);
    b.inst(Rol(Accumulator), ());
    b.inst(Asl(ZeroPage), 254);
    b.inst(Rol(Accumulator), ());
    b.inst(Asl(ZeroPage), 254);
    b.inst(Rol(Accumulator), ());

    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Lda(ZeroPage), 254);
    b.inst(Sta(Absolute), Addr(0x2006)); // restore ppuaddr
    b.inst(Pla, ());
    b.inst(Sta(ZeroPage), 254); // restore offset

    b.label("post-vblank-wait");

    b.inst(Inc(ZeroPage), 254); // increment offset

    b.inst(Inx, ());
    b.inst(Lda(ZeroPageXIndexed), 0); // read next byte into accumulator
    for i in 0..8 {
        b.inst(Sta(Absolute), Addr(0x2007));
        if i < 7 {
            b.inst(Ror(Accumulator), ());
        }
    }
    b.inst(Jmp(Absolute), "byte-run-start");
    b.label("byte-run-end");

    b.inst(Inx, ());
    b.inst(Jmp(Absolute), "update-ppu-start");
    b.label("update-ppu-end");

    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x2005));
    b.inst(Sta(Absolute), Addr(0x2005)); // fix scroll

    b.inst(Lda(Immediate), 0xFF);
    b.inst(Sta(ZeroPage), 0); // clear update buffer

    b.inst(Jmp(Absolute), "mainloop");

    b.infinite_loop();

    b.label("nmi");
    b.inst(Rti, ());

    b.set_offset(INTERRUPT_VECTOR_START_PC_OFFSET);
    b.label_offset_le("reset");
    b.set_offset(INTERRUPT_VECTOR_NMI_OFFSET);
    b.label_offset_le("nmi");
}

fn chr_rom() -> Vec<u8> {
    let mut chr_rom = vec![0; ines::CHR_ROM_BLOCK_BYTES];
    for tile_index in 0..256 {
        if tile_index % 2 == 1 {
            let byte_index = tile_index * 16;
            for pixel_offset in 0..8 {
                chr_rom[byte_index + pixel_offset] = 0xFF;
            }
        }
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
        .write(&encoded)
        .expect("Failed to write encoded rom to stdout");
}
