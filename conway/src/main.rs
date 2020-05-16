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
    b.inst(Lda(Immediate), 0x21);
    b.inst(Sta(ZeroPage), 0);
    b.inst(Sta(ZeroPage), 2);
    b.inst(Lda(Immediate), 0x00);
    b.inst(Sta(ZeroPage), 1);
    b.inst(Sta(ZeroPage), 3);

    // enable rendering
    b.inst(Lda(Immediate), 0b00001010);
    b.inst(Sta(Absolute), Addr(0x2001)); // turn on background and left-background

    b.label("mainloop");

    b.label("vblankmain");
    b.inst(Bit(Absolute), Addr(0x2002));
    b.inst(Bpl, LabelRelativeOffset("vblankmain"));

    // update display
    b.inst(Bit(Absolute), Addr(0x2002)); // read ppu status to clear address latch

    // clear previous
    b.inst(Lda(ZeroPage), 3);
    b.inst(Ora(Immediate), 0x20);
    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Lda(ZeroPage), 0x02);
    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x2007));

    // set current
    b.inst(Lda(ZeroPage), 1);
    b.inst(Ora(Immediate), 0x20);
    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Lda(ZeroPage), 0x00);
    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Lda(Immediate), 1);
    b.inst(Sta(Absolute), Addr(0x2007));

    // set scroll
    b.inst(Bit(Absolute), Addr(0x2002)); // read ppu status to clear address latch
    b.inst(Lda(Immediate), 0x00);
    b.inst(Sta(Absolute), Addr(0x2005));
    b.inst(Sta(Absolute), Addr(0x2005));

    // update state

    // copy previous value so it can be cleared next frame
    b.inst(Lda(ZeroPage), 0);
    b.inst(Sta(ZeroPage), 2);
    b.inst(Lda(ZeroPage), 1);
    b.inst(Sta(ZeroPage), 3);

    // test if current value needs to wrap around
    // max value is 960, which is 0x03BF in hex
    b.inst(Lda(ZeroPage), 0);
    b.inst(Cmp(Immediate), 0xBF);
    b.inst(Bne, LabelRelativeOffset("inc"));
    b.inst(Lda(ZeroPage), 1);
    b.inst(Cmp(Immediate), 0x03);
    b.inst(Bne, LabelRelativeOffset("inc"));
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(ZeroPage), 0);
    b.inst(Sta(ZeroPage), 1);
    b.inst(Jmp(Absolute), "incpost");
    b.label("inc");
    b.inst(Clc, ());
    b.inst(Lda(ZeroPage), 0);
    b.inst(Adc(Immediate), 1);
    b.inst(Sta(ZeroPage), 0);
    b.inst(Lda(ZeroPage), 1);
    b.inst(Adc(Immediate), 0);
    b.inst(Sta(ZeroPage), 1);
    b.label("incpost");

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
    for i in 0..8 {
        chr_rom[16 + i] = 0xFF;
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
