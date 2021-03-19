use ines::Ines;
use mos6502_assembler::{Addr, Block, LabelRelativeOffset, LabelRelativeOffsetOwned};
use mos6502_model::{address, interrupt_vector, Address};

const PRG_START: Address = 0xC000;
const INTERRUPT_VECTOR_START_PC_OFFSET: Address = interrupt_vector::START_LO - PRG_START;
const INTERRUPT_VECTOR_NMI_OFFSET: Address = interrupt_vector::NMI_LO - PRG_START;
const OFFSET_TABLE_START: Address = 0xFC00;
const OFFSET_TABLE_START_OFFSET: Address = OFFSET_TABLE_START - PRG_START;

fn program(b: &mut Block) {
    use mos6502_model::addressing_mode::*;
    use mos6502_model::assembler_instruction::*;

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
    let universal_background = 0x10;
    let background_colour_1 = 0x11;
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
    b.inst(Jsr(Absolute), "rng-init");
    b.inst(Jsr(Absolute), "rng-new-state");

    // enable rendering
    b.inst(Lda(Immediate), 0b00001010);
    b.inst(Sta(Absolute), Addr(0x2001)); // turn on background and left-background

    b.inst(Lda(Immediate), 0);
    b.inst(Sta(ZeroPage), 252); // direction

    b.label("mainloop");

    b.inst(Jsr(Absolute), "rng-increment");

    b.inst(Jsr(Absolute), "controller-to-255");

    b.inst(Lda(ZeroPage), 255);
    b.inst(Beq, LabelRelativeOffset("vblankmain"));
    b.inst(Jsr(Absolute), "rng-new-state");

    b.label("vblankmain");
    b.inst(Bit(Absolute), Addr(0x2002));
    b.inst(Bpl, LabelRelativeOffset("vblankmain"));

    // update ppu memory
    b.inst(Bit(Absolute), Addr(0x2002)); // read ppu status to clear address latch

    let gas = 16;
    b.inst(Lda(Immediate), gas);
    b.inst(Sta(ZeroPage), 255); // store gas

    b.inst(Ldx(Immediate), 0); // initialize x register

    b.label("update-ppu-start");
    b.inst(Lda(ZeroPageXIndexed), 0);
    b.inst(Bmi, LabelRelativeOffset("update-ppu-end")); // a negative tile index indicates the end of the draw queue

    b.inst(Sta(ZeroPage), 254); //  store current tile index

    b.inst(Clc, ());
    b.inst(Lda(Immediate), 0x04);
    b.inst(Asl(ZeroPageXIndexed), 0);
    b.inst(Rol(Accumulator), ());
    b.inst(Asl(ZeroPageXIndexed), 0);
    b.inst(Rol(Accumulator), ());
    b.inst(Asl(ZeroPageXIndexed), 0);
    b.inst(Rol(Accumulator), ()); // compute the ppu address for the current run

    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Lda(ZeroPageXIndexed), 0);
    b.inst(Sta(Absolute), Addr(0x2006)); // set ppu addr for current run

    b.inst(Inx, ());
    b.inst(Ldy(ZeroPageXIndexed), 0); // read length (bytes) of run into y register

    b.label("byte-run-start");
    b.inst(Dey, ());
    b.inst(Bmi, LabelRelativeOffset("byte-run-end")); // if y was decremented to negative, the run is over

    b.inst(Dec(ZeroPage), 255); // spend gas
    b.inst(Bne, LabelRelativeOffset("post-vblank-wait"));
    b.inst(Lda(Immediate), gas);
    b.inst(Sta(ZeroPage), 255); // reset gas

    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x2005));
    b.inst(Sta(Absolute), Addr(0x2005)); // fix scroll

    b.label("vblankmain-gas");
    b.inst(Bit(Absolute), Addr(0x2002));
    b.inst(Bpl, LabelRelativeOffset("vblankmain-gas")); // gas has run out, so wait until start of next vblank to continue

    b.inst(Bit(Absolute), Addr(0x2002)); // read ppu status to clear address latch

    b.inst(Clc, ());
    b.inst(Lda(ZeroPage), 254);
    b.inst(Pha, ()); // back up offset
    b.inst(Lda(Immediate), 0x04); // start at 0x04 so when multiplied by 8 ends up as 0x20XX
    b.inst(Asl(ZeroPage), 254);
    b.inst(Rol(Accumulator), ());
    b.inst(Asl(ZeroPage), 254);
    b.inst(Rol(Accumulator), ());
    b.inst(Asl(ZeroPage), 254);
    b.inst(Rol(Accumulator), ()); // multiply by 8 to get byte address

    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Lda(ZeroPage), 254);
    b.inst(Sta(Absolute), Addr(0x2006)); // restore ppuaddr
    b.inst(Pla, ());
    b.inst(Sta(ZeroPage), 254); // restore offset which was corrupted during above multiply

    b.label("post-vblank-wait");

    b.inst(Inc(ZeroPage), 254); // increment offset

    b.inst(Inx, ());
    b.inst(Lda(ZeroPageXIndexed), 0); // read next byte into accumulator
    for i in 0..8 {
        b.inst(Sta(Absolute), Addr(0x2007));
        if i < 7 {
            b.inst(Lsr(Accumulator), ());
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

    fn conway_update(b: &mut Block, from: u16, to: u16, prefix: &str) {
        b.inst(Lda(Immediate), address::lo(OFFSET_TABLE_START));
        b.inst(Sta(ZeroPage), 254);
        b.inst(Lda(Immediate), address::hi(OFFSET_TABLE_START));
        b.inst(Sta(ZeroPage), 255); // store offset table address at top of zero page

        b.inst(Lda(Immediate), 0);
        b.inst(Sta(ZeroPage), 253); // 253 will be offset into state

        b.label(format!("{}-life-update-start", prefix));
        b.inst(Lda(ZeroPage), 253);
        b.inst(Cmp(Immediate), 120);
        b.inst(
            Bne,
            LabelRelativeOffsetOwned(format!("{}-skip-life-update-end", prefix)),
        );

        b.inst(Jmp(Absolute), format!("{}-life-update-end", prefix));

        b.label(format!("{}-skip-life-update-end", prefix));

        // zero-out counters
        b.inst(Lda(Immediate), 0);
        for i in 0..8 {
            b.inst(Sta(ZeroPage), i);
        }

        b.inst(Ldy(Immediate), 0); // Y will be offset into offset table

        // top
        b.inst(Lda(IndirectYIndexed), 254);
        b.inst(Iny, ());
        b.inst(Tax, ());
        b.inst(Lda(AbsoluteXIndexed), from);
        b.inst(Tax, ()); // backup in X

        for i in 0..=7 {
            if i != 0 {
                // no point copying when i == 0 as A will already contain value
                b.inst(Txa, ());
            }
            b.inst(And(Immediate), 1 << i);
            b.inst(
                Beq,
                LabelRelativeOffsetOwned(format!("{}-skip-top-{}", prefix, i)),
            );
            if i != 0 {
                b.inst(Inc(ZeroPage), i - 1);
            }
            b.inst(Inc(ZeroPage), i);
            if i != 7 {
                b.inst(Inc(ZeroPage), i + 1);
            }
            b.label(format!("{}-skip-top-{}", prefix, i));
        }

        // bottom
        b.inst(Lda(IndirectYIndexed), 254);
        b.inst(Iny, ());
        b.inst(Tax, ());
        b.inst(Lda(AbsoluteXIndexed), from);
        b.inst(Tax, ()); // backup in X

        for i in 0..=7 {
            if i != 0 {
                // no point copying when i == 0 as A will already contain value
                b.inst(Txa, ());
            }
            b.inst(And(Immediate), 1 << i);
            b.inst(
                Beq,
                LabelRelativeOffsetOwned(format!("{}-skip-bottom-{}", prefix, i)),
            );
            if i != 0 {
                b.inst(Inc(ZeroPage), i - 1);
            }
            b.inst(Inc(ZeroPage), i);
            if i != 7 {
                b.inst(Inc(ZeroPage), i + 1);
            }
            b.label(format!("{}-skip-bottom-{}", prefix, i));
        }

        // left, top-left, bottom-left
        for i in 0..=2 {
            b.inst(Lda(IndirectYIndexed), 254);
            b.inst(Iny, ());
            b.inst(Tax, ());
            b.inst(Lda(AbsoluteXIndexed), from);

            b.inst(And(Immediate), 1 << 7);
            b.inst(
                Beq,
                LabelRelativeOffsetOwned(format!("{}-skip-left-{}", prefix, i)),
            );
            b.inst(Inc(ZeroPage), 0);
            b.label(format!("{}-skip-left-{}", prefix, i));
        }

        // right, top-right, bottom-right
        for i in 0..=2 {
            b.inst(Lda(IndirectYIndexed), 254);
            b.inst(Iny, ());
            b.inst(Tax, ());
            b.inst(Lda(AbsoluteXIndexed), from);

            b.inst(And(Immediate), 1 << 0);
            b.inst(
                Beq,
                LabelRelativeOffsetOwned(format!("{}-skip-right-{}", prefix, i)),
            );
            b.inst(Inc(ZeroPage), 7);
            b.label(format!("{}-skip-right-{}", prefix, i));
        }

        // current
        b.inst(Ldx(ZeroPage), 253);
        b.inst(Lda(AbsoluteXIndexed), from);
        b.inst(Tax, ()); // backup in X

        for i in 0..=7 {
            if i != 0 {
                // no point copying when i == 0 as A will already contain value
                b.inst(Txa, ());
            }
            b.inst(And(Immediate), 1 << i);
            b.inst(
                Beq,
                LabelRelativeOffsetOwned(format!("{}-skip-current-{}", prefix, i)),
            );
            if i != 0 {
                b.inst(Inc(ZeroPage), i - 1);
            }
            if i != 7 {
                b.inst(Inc(ZeroPage), i + 1);
            }
            b.label(format!("{}-skip-current-{}", prefix, i));
        }

        // will build up result in zp8
        b.inst(Lda(Immediate), 0);
        b.inst(Sta(ZeroPage), 8);

        for i in 0..=7 {
            b.inst(Txa, ()); // X still contains current byte
            b.inst(And(Immediate), 1 << i);
            b.inst(
                Beq,
                LabelRelativeOffsetOwned(format!("{}-current-dead-{}", prefix, i)),
            );
            // currently alive
            b.inst(Lda(ZeroPage), i);
            b.inst(Cmp(Immediate), 2);
            b.inst(
                Beq,
                LabelRelativeOffsetOwned(format!("{}-next-alive-{}", prefix, i)),
            );
            b.inst(Cmp(Immediate), 3);
            b.inst(
                Beq,
                LabelRelativeOffsetOwned(format!("{}-next-alive-{}", prefix, i)),
            );
            b.inst(Jmp(Absolute), format!("{}-next-dead-{}", prefix, i));

            b.label(format!("{}-current-dead-{}", prefix, i));

            b.inst(Lda(ZeroPage), i);
            b.inst(Cmp(Immediate), 3);
            b.inst(
                Bne,
                LabelRelativeOffsetOwned(format!("{}-next-dead-{}", prefix, i)),
            );

            b.label(format!("{}-next-alive-{}", prefix, i));

            b.inst(Lda(ZeroPage), 8);
            b.inst(Ora(Immediate), 1 << i);
            b.inst(Sta(ZeroPage), 8);

            b.label(format!("{}-next-dead-{}", prefix, i));
        }

        // store the result in the output
        b.inst(Lda(ZeroPage), 8);
        b.inst(Ldx(ZeroPage), 253);
        b.inst(Sta(AbsoluteXIndexed), to);

        b.inst(Inc(ZeroPage), 253);
        b.inst(Clc, ());
        b.inst(Lda(ZeroPage), 254);
        b.inst(Adc(Immediate), 8);
        b.inst(Sta(ZeroPage), 254);
        b.inst(Lda(ZeroPage), 255);
        b.inst(Adc(Immediate), 0);
        b.inst(Sta(ZeroPage), 255); // increment pointers

        b.inst(Jmp(Absolute), format!("{}-life-update-start", prefix));
        b.label(format!("{}-life-update-end", prefix));
    }

    fn enqueue_delta(b: &mut Block, from: u16, to: u16, prefix: &str) {
        b.inst(Lda(Immediate), 0xFB);
        b.inst(Sta(ZeroPage), 0); // clear update buffer

        // write new draw queue by diffing previous and current images
        b.inst(Ldx(Immediate), 0);
        b.inst(Ldy(Immediate), 0);

        b.inst(Stx(ZeroPage), 255); // not currently in a run
        b.inst(Stx(ZeroPage), 254); // MSB of count is always 0, but needed to form address
        b.inst(Stx(ZeroPage), 253); // LSB of count

        b.label(format!("{}-diff-start", prefix));
        b.inst(Txa, ());
        b.inst(Cmp(Immediate), 120);
        b.inst(
            Beq,
            LabelRelativeOffsetOwned(format!("{}-diff-end", prefix)),
        );

        b.inst(Lda(AbsoluteXIndexed), Addr(from));
        b.inst(Eor(AbsoluteXIndexed), Addr(to));

        b.inst(
            Bne,
            LabelRelativeOffsetOwned(format!("{}-add-diff-to-draw-queue", prefix)),
        );

        b.inst(Sta(ZeroPage), 255); // we know A is zero - no longer in a run
        b.inst(Inx, ());
        b.inst(Jmp(Absolute), format!("{}-diff-start", prefix));

        b.label(format!("{}-add-diff-to-draw-queue", prefix));

        b.inst(Lda(ZeroPage), 255);
        b.inst(
            Bne,
            LabelRelativeOffsetOwned(format!("{}-increment-counter-append-byte", prefix)),
        );

        b.inst(Tya, ());
        b.inst(
            Beq,
            LabelRelativeOffsetOwned(format!("{}-append-offset", prefix)),
        );

        b.inst(Lda(ZeroPage), 253);
        b.inst(Sta(AbsoluteYIndexed), Addr(0)); // store the previous counter value at Y

        b.inst(Tya, ());
        b.inst(Clc, ());
        b.inst(Adc(ZeroPage), 253);
        b.inst(Tay, ());
        b.inst(Iny, ()); // Y now points where the offset will go

        b.label(format!("{}-append-offset", prefix));

        b.inst(Stx(ZeroPageYIndexed), 0); // X contains the current offset
        b.inst(Iny, ()); // Y now points where the length will go

        b.inst(Lda(Immediate), 0);
        b.inst(Sta(ZeroPage), 253); // clear current count LBS (MSB is always clear)

        b.label(format!("{}-increment-counter-append-byte", prefix));

        b.inst(Inc(ZeroPage), 253); // increment counter
        b.inst(Lda(AbsoluteXIndexed), Addr(to)); // load byte from current image
        b.inst(Sta(IndirectYIndexed), 253); // store at *(253) + Y

        b.inst(Inx, ());
        b.inst(Stx(ZeroPage), 255); // X can't be 0 at this point. Set flag to non-zero value.

        b.inst(Jmp(Absolute), format!("{}-diff-start", prefix));
        b.label(format!("{}-diff-end", prefix));

        b.inst(Tya, ());
        b.inst(Tax, ());
        b.inst(Lda(ZeroPage), 253);
        b.inst(
            Bne,
            LabelRelativeOffsetOwned(format!("{}-non-empty", prefix)),
        );
        b.inst(Lda(Immediate), 0xFA);
        b.inst(Sta(ZeroPage), 0); // empty queue - place terminator at start of zero page
        b.inst(Jmp(Absolute), format!("{}-end", prefix));
        b.label(format!("{}-non-empty", prefix));
        b.inst(Sta(ZeroPageXIndexed), 0); // store the previous counter value at Y (X is copied from Y)

        b.inst(Txa, ());
        b.inst(Clc, ());
        b.inst(Adc(ZeroPage), 253);
        b.inst(Tax, ());
        b.inst(Inx, ()); // X now points where the terminator will go
        b.inst(Lda(Immediate), 0xFC);
        b.inst(Sta(ZeroPageXIndexed), 0);

        b.label(format!("{}-end", prefix));
    }

    b.inst(Lda(ZeroPage), 252);
    b.inst(Eor(Immediate), 1);
    b.inst(Sta(ZeroPage), 252);
    b.inst(Beq, LabelRelativeOffset("enqueue-delta-b-to-a"));

    b.inst(Jsr(Absolute), "fn-a-to-b");
    b.inst(Jmp(Absolute), "post-enqueue-delta");

    b.label("enqueue-delta-b-to-a");
    b.inst(Jsr(Absolute), "fn-b-to-a");

    b.label("post-enqueue-delta");

    b.inst(Jmp(Absolute), "mainloop");

    b.infinite_loop();

    b.label("fn-a-to-b");
    conway_update(b, 0x0200, 0x0280, "A");
    enqueue_delta(b, 0x0200, 0x0280, "A");
    b.inst(Rts, ());

    b.label("fn-b-to-a");
    conway_update(b, 0x0280, 0x0200, "B");
    enqueue_delta(b, 0x0280, 0x0200, "B");
    b.inst(Rts, ());

    b.label("rng-init");
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(ZeroPage), 251);
    b.inst(Sta(ZeroPage), 250);
    b.inst(Sta(ZeroPage), 249);
    b.inst(Sta(ZeroPage), 248);
    b.inst(Rts, ());

    b.label("rng-increment");
    b.inst(Clc, ());
    b.inst(Lda(ZeroPage), 248);
    b.inst(Adc(Immediate), 1);
    b.inst(Sta(ZeroPage), 248);
    b.inst(Lda(ZeroPage), 249);
    b.inst(Adc(Immediate), 0);
    b.inst(Sta(ZeroPage), 249);
    b.inst(Lda(ZeroPage), 250);
    b.inst(Adc(Immediate), 0);
    b.inst(Sta(ZeroPage), 250);
    b.inst(Lda(ZeroPage), 251);
    b.inst(Adc(Immediate), 0);
    b.inst(Sta(ZeroPage), 251);
    b.inst(Rts, ());

    b.label("rng-generate");
    b.inst(Lda(ZeroPage), 248);
    b.inst(Ora(Immediate), 1); // make sure the seed is not 0
    b.inst(Sta(ZeroPage), 248);

    fn rng_generate_copy(b: &mut Block) {
        for i in 248..=251 {
            b.inst(Lda(ZeroPage), i);
            b.inst(Sta(ZeroPage), i - 4);
        }
    }

    fn rng_generate_xor(b: &mut Block) {
        for i in 248..=251 {
            b.inst(Lda(ZeroPage), i);
            b.inst(Eor(ZeroPage), i - 4);
            b.inst(Sta(ZeroPage), i);
        }
    }

    fn rng_generate_left_shift(b: &mut Block, n: u8) {
        for _ in 0..n {
            b.inst(Asl(ZeroPage), 244);
            b.inst(Rol(ZeroPage), 245);
            b.inst(Rol(ZeroPage), 246);
            b.inst(Rol(ZeroPage), 247);
        }
    }

    fn rng_generate_right_shift(b: &mut Block, n: u8) {
        for _ in 0..n {
            b.inst(Lsr(ZeroPage), 247);
            b.inst(Ror(ZeroPage), 246);
            b.inst(Ror(ZeroPage), 245);
            b.inst(Ror(ZeroPage), 244);
        }
    }

    rng_generate_copy(b);
    rng_generate_left_shift(b, 13);
    rng_generate_xor(b);

    rng_generate_copy(b);
    rng_generate_right_shift(b, 17);
    rng_generate_xor(b);

    rng_generate_copy(b);
    rng_generate_left_shift(b, 5);
    rng_generate_xor(b);

    b.inst(Rts, ());

    b.label("rng-new-state");
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(ZeroPage), 0);
    b.inst(Lda(Immediate), 120);
    b.inst(Sta(ZeroPage), 1); // initialize draw queue
    for y in 0..30u8 {
        b.inst(Jsr(Absolute), "rng-generate");
        for x in 0..4u8 {
            let index = y * 4 + x;
            b.inst(Lda(ZeroPage), 248 + x); // load random byte
            b.inst(Sta(ZeroPage), index + 2); // write pattern to draw queue
            b.inst(Sta(Absolute), Addr(0x0200 + index as u16));
            b.inst(Sta(Absolute), Addr(0x0280 + index as u16)); // write pattern to state
        }
    }

    // zero out 120th byte of each state buffer
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x0200 + 120));
    b.inst(Sta(Absolute), Addr(0x0280 + 120));

    // terminate draw queue
    b.inst(Lda(Immediate), 0xFA);
    b.inst(Sta(ZeroPage), 122);

    b.inst(Rts, ());

    b.label("controller-to-255");
    const CONTROLLER_REG: Addr = Addr(0x4016);

    // toggle the controller strobe bit to copy its current value into shift register
    b.inst(Lda(Immediate), 1);
    b.inst(Sta(Absolute), CONTROLLER_REG); // set controller strobe
    b.inst(Sta(ZeroPage), 255); // store a 1 at 255 - used to check when all bits are read
    b.inst(Lsr(Accumulator), ()); // clear accumulator
    b.inst(Sta(Absolute), CONTROLLER_REG); // clear controller strobe

    // shift each of the 8 bits of controller state from the shift register into address 255
    b.label("controller-to-255-loop");
    b.inst(Lda(Absolute), CONTROLLER_REG); // load single bit into LBS of acculumator
    b.inst(Lsr(Accumulator), ()); // shift bit into carry flag
    b.inst(Rol(ZeroPage), 255); // shift carry flag into 255, and MSB of 255 into carry flag

    // if that set the carry flag, this was the 8th iteration
    b.inst(Bcc, LabelRelativeOffset("controller-to-255-loop"));

    b.inst(Rts, ());

    b.label("nmi");
    b.inst(Rti, ());

    b.set_offset(OFFSET_TABLE_START_OFFSET);
    const CELL_BLOCK_NUM_ROWS: u8 = 30;
    const CELL_BLOCK_NUM_COLS: u8 = 4;
    const CELL_BLOCK_INDEX_OUT_OF_BOUNDS: u8 = CELL_BLOCK_NUM_ROWS * CELL_BLOCK_NUM_COLS;
    for cell_block_row in 0..CELL_BLOCK_NUM_ROWS {
        for cell_block_col in 0..CELL_BLOCK_NUM_COLS {
            let top = if cell_block_row == 0 {
                None
            } else {
                Some((cell_block_row - 1) * CELL_BLOCK_NUM_COLS + cell_block_col)
            };
            let bottom = if cell_block_row == CELL_BLOCK_NUM_ROWS - 1 {
                None
            } else {
                Some((cell_block_row + 1) * CELL_BLOCK_NUM_COLS + cell_block_col)
            };
            let left = if cell_block_col == 0 {
                None
            } else {
                Some(cell_block_row * CELL_BLOCK_NUM_COLS + (cell_block_col - 1))
            };
            let right = if cell_block_col == CELL_BLOCK_NUM_COLS - 1 {
                None
            } else {
                Some(cell_block_row * CELL_BLOCK_NUM_COLS + (cell_block_col + 1))
            };
            let top_left = left.and(top).map(|top| top - 1);
            let top_right = right.and(top).map(|top| top + 1);
            let bottom_left = left.and(bottom).map(|bottom| bottom - 1);
            let bottom_right = right.and(bottom).map(|bottom| bottom + 1);
            b.literal_byte(top.unwrap_or(CELL_BLOCK_INDEX_OUT_OF_BOUNDS));
            b.literal_byte(bottom.unwrap_or(CELL_BLOCK_INDEX_OUT_OF_BOUNDS));
            b.literal_byte(left.unwrap_or(CELL_BLOCK_INDEX_OUT_OF_BOUNDS));
            b.literal_byte(top_left.unwrap_or(CELL_BLOCK_INDEX_OUT_OF_BOUNDS));
            b.literal_byte(bottom_left.unwrap_or(CELL_BLOCK_INDEX_OUT_OF_BOUNDS));
            b.literal_byte(right.unwrap_or(CELL_BLOCK_INDEX_OUT_OF_BOUNDS));
            b.literal_byte(top_right.unwrap_or(CELL_BLOCK_INDEX_OUT_OF_BOUNDS));
            b.literal_byte(bottom_right.unwrap_or(CELL_BLOCK_INDEX_OUT_OF_BOUNDS));
        }
    }

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
    env_logger::init();
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
