use ines::Ines;
use mos6502_assembler::{Addr, Block, LabelRelativeOffset};
use mos6502_model::{interrupt_vector, Address};

const PRG_START: Address = 0x8000;
const INTERRUPT_VECTOR_START_PC_OFFSET: Address = interrupt_vector::START_LO - PRG_START;
const INTERRUPT_VECTOR_NMI_OFFSET: Address = interrupt_vector::NMI_LO - PRG_START;

// Write the PRG ROM section of a ROM with code to play audio samples, and the audio samples
// themselves, skipping over the first ines::CHR_ROM_BLOCK_BYTES bytes with the expectation that
// they are stored in CHR ROM instead.
//
// b is a mas6502 assembler which we can use to emit code and add data to the ROM
// audio_data is an array of samples quantized into 7-bit integers
fn program(b: &mut Block, audio_data: &[u8]) {
    use mos6502_model::addressing_mode::*;
    use mos6502_model::assembler_instruction::*;

    // Add the interrupt vector at the end of the ROM
    b.set_offset(INTERRUPT_VECTOR_START_PC_OFFSET);
    b.label_offset_le("reset");
    b.set_offset(INTERRUPT_VECTOR_NMI_OFFSET);
    b.label_offset_le("nmi");

    // Start adding instructions to PRG ROM at the beginning of memory
    b.set_offset(0);

    // Simple interrupt handler for non-maskable interrupts
    b.label("nmi");
    b.inst(Rti, ());

    // Execution will start at the "reset" label. This is our program's entry point.
    b.label("reset");

    // write 0 to the ppu control register
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x2000));

    // Enable the DMC
    b.inst(Lda(Immediate), 1 << 4);
    b.inst(Sta(Absolute), Addr(0x4015));

    b.label("play-audio");

    // read ppu status to clear ppu address latch, then write 0 to the ppu address
    //b.inst(Bit(Absolute), Addr(0x2002)); // clear PPU address latch

    // We'll start by playing the audio samples stored in tile memory. Tile memory is read through
    // the registers of the PPU. PPU addresses are 16 bytes wide, and the current address is set by
    // writing the high byte, then the low byte, to the PPUADDR register mapped into memory at
    // address 0x2006. We'll start reading samples from PPU address 0, so write a 0 to both the
    // high and low bytes of the PPU address.
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Sta(Absolute), Addr(0x2006));

    // We'll need a 16 bit counter to keep track of our current position within tile memory so that
    // we know when to stop reading. We'll use a little-endian 16-bit integer at address 2. Start
    // by setting the counter to 0. Note that the accumulator still has a 0 in it from setting the
    // PPU address to 0 above.
    let counter_addr = 2;
    b.inst(Sta(ZeroPage), counter_addr);
    b.inst(Sta(ZeroPage), counter_addr + 1);

    // We're going to be reading audio samples out of PPU memory. After setting the PPU address,
    // the first read will return garbage, and subsequent reads will start from the address written
    // to the PPU address. Read the single garbage value from the PPU and discard it.
    b.inst(Lda(Absolute), Addr(0x2007));

    // label indicating the start of the loop which plays samples from PPU memory
    b.label("ppu-loop-start");

    // Call a function "delay" which spins for a fixed number of cycles. This is needed so that the
    // rate at which we send samples to the DMC load register is 3000 times per second. Without
    // this, the song would play in fast-forward. Note that we do this before sending each sample
    // to the DMC.
    b.inst(Jsr(Absolute), "delay");

    // Read a value from PPU memory into the accumulator. Reading from PPU memory has the effect of
    // incrementing the current PPU address, so the nexit read will be from the next location in
    // PPU memory.
    b.inst(Lda(Absolute), Addr(0x2007));

    // Write the value we just read from PPU memory into the DMC's load register.
    b.inst(Sta(Absolute), Addr(0x4011));

    // Call a function which increments the 16 bit counter at address 2.
    b.inst(Jsr(Absolute), "incr-counter");

    // Check that the counter is less than 0x2000 (8k). As soon as it reaches 0x2000, we've
    // consumed all the audio samples stored in PPU memory, so we exit this loop.
    let num_ppu_samples = 0x2000;
    b.inst(Lda(Immediate), mos6502_model::address::hi(num_ppu_samples));
    b.inst(Cmp(ZeroPage), counter_addr + 1);
    b.inst(Bne, LabelRelativeOffset("ppu-loop-start"));
    b.inst(Lda(Immediate), mos6502_model::address::lo(num_ppu_samples));
    b.inst(Cmp(ZeroPage), counter_addr);
    b.inst(Bne, LabelRelativeOffset("ppu-loop-start"));

    // Now we need to play the rest of the sample from regular ROM (PRG ROM to be precise). The ROM
    // is mapped into memory from 0x8000 up to 0xFFFF. Set asside 0x70 (112) bytes of memory to
    // store this program before the audio data begins. The final 6 bytes of ROM are used for the
    // interrupt table, so we can't store audio data there. This leaves us with memory from 0x8070
    // to (but not including) 0xFFFA to store audio data.
    let audio_start = 0x8070;
    let audio_max = 0xFFFA;

    // We'll reuse the 16 bit integer at address 2 to keep track of our current position within the
    // audio data. Start by setting its value to 0x8070.
    b.inst(Lda(Immediate), mos6502_model::address::lo(audio_start));
    b.inst(Sta(ZeroPage), counter_addr);
    b.inst(Lda(Immediate), mos6502_model::address::hi(audio_start));
    b.inst(Sta(ZeroPage), counter_addr + 1);

    // Label indicating the start of the loop over data stored in PRG ROM.
    b.label("loop-start");

    // Pause so that the playback rate matches the sample rate of 3kHz.
    b.inst(Jsr(Absolute), "delay");

    // Read the next sample from ROM. The 6502 processor has an addressing mode which reads an
    // address from a 16 bit little-endian address in the first 256 bytes of RAM (called the "zero
    // page"). This lets us use the 16 bit counter as the address of the current sample. The index
    // register X is added to final address before it's accessed. This line assumes that X is 0.
    // The "delay" function we just called ensures that X is 0 when it returns (internally it uses
    // X as a counter and counts down to 0).
    b.inst(Lda(XIndexedIndirect), counter_addr);

    // Write the sample to the DMC load register.
    b.inst(Sta(Absolute), Addr(0x4011));

    // Increment the counter at address 2.
    b.inst(Jsr(Absolute), "incr-counter");

    // Exit the loop when the counter reaches the end of the sample buffer.
    b.inst(Lda(Immediate), mos6502_model::address::hi(audio_max));
    b.inst(Cmp(ZeroPage), counter_addr + 1);
    b.inst(Bne, LabelRelativeOffset("loop-start"));
    b.inst(Lda(Immediate), mos6502_model::address::lo(audio_max));
    b.inst(Cmp(ZeroPage), counter_addr);
    b.inst(Bne, LabelRelativeOffset("loop-start"));

    // Loop forever.
    b.label("end");
    b.inst(Jmp(Absolute), "end");

    // A function which adds 1 to the 16 bit little-endian counter at address 2
    b.label("incr-counter");
    b.inst(Clc, ());
    b.inst(Lda(ZeroPage), counter_addr);
    b.inst(Adc(Immediate), 1);
    b.inst(Sta(ZeroPage), counter_addr);
    b.inst(Lda(ZeroPage), counter_addr + 1);
    b.inst(Adc(Immediate), 0);
    b.inst(Sta(ZeroPage), counter_addr + 1);
    b.inst(Rts, ());

    // A function which returns after some time has passed, and ensures that the X index register
    // is 0.
    b.label("delay");
    // This function works by loading this value into the X index register and then repeatedly
    // decrementing it until it reaches 0. The optimal starting value could be computed by counting
    // the number of cycles in the loops which send successive audio samples to the DMC, but it was
    // easier to just tune this by hand. The number of cycles each loop spends updating the value
    // in the DMC load register is similar enough that the same delay function can be used for
    // both.
    let delay = 107;
    b.inst(Ldx(Immediate), delay);
    b.label("delay-loop");
    b.inst(Dex, ());
    b.inst(Bne, LabelRelativeOffset("delay-loop"));
    b.inst(Rts, ());

    // Make sure that the code ends before the declared start of the audio buffer (otherwise we
    // could overwrite code with audio data).
    assert!(b.offset() < audio_start);

    // The assembler keeps track of a current address which is incremented as intsructions are
    // issued. Explicitly set the current address to the start of the audio buffer. The assembler
    // doesn't know that the ROM will be mapped into memory starting at address 0x8000, so we
    // subtract it so the audio buffer starts relative to address 0 instead.
    b.set_offset(audio_start - 0x8000);

    // Loop over all the samples, adding them to the ROM. The offset of ines::CHR_ROM_BLOCK_BYTES
    // is because the CHR_ROM (ie. tile memory) contains the beginning of the audio data. Samples
    // have already been quantized into 7 bit integers.
    let num_samples = audio_max as usize - audio_start as usize;
    for &sample in &audio_data[ines::CHR_ROM_BLOCK_BYTES..(ines::CHR_ROM_BLOCK_BYTES + num_samples)]
    {
        b.literal_byte(sample);
    }

    // Make sure we left enough room for the interrupt table.
    assert!(b.offset() <= audio_max - PRG_START);
}

fn chr_rom(audio_data: &[u8]) -> Vec<u8> {
    let mut chr_rom = vec![0; ines::CHR_ROM_BLOCK_BYTES];
    chr_rom.copy_from_slice(&audio_data[0..ines::CHR_ROM_BLOCK_BYTES]);
    chr_rom
}

fn prg_rom(audio_data: &[u8]) -> Vec<u8> {
    let mut block = Block::new();
    program(&mut block, audio_data);
    let mut prg_rom = Vec::new();
    block
        .assemble(PRG_START, ines::PRG_ROM_BLOCK_BYTES * 2, &mut prg_rom)
        .expect("Failed to assemble");
    prg_rom
}

struct Args {
    wav_path: String,
}

impl Args {
    fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                wav_path = pos_req("PATH");
            } in {
                Self { wav_path }
            }
        }
    }
}

fn decode_wav(path: String) -> (Vec<u8>, u32) {
    let mut reader = hound::WavReader::open(path).unwrap();
    let spec = reader.spec();
    let original_data = reader
        .samples::<i32>()
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();
    let mut min = i32::MAX;
    let mut max = i32::MIN;
    let mut channel_means = Vec::new();
    for sample in original_data.chunks(spec.channels as usize) {
        let mean = sample.iter().sum::<i32>() / (spec.channels as i32);
        channel_means.push(mean);
        min = mean.min(min);
        max = mean.max(max);
    }
    let mut output = Vec::new();
    for sample in channel_means {
        let normalized = ((127 * (sample - min)) / (max - min)).clamp(0, 127);
        output.push(normalized as u8);
    }
    (output, spec.sample_rate)
}

fn main() {
    use meap::Parser;
    let Args { wav_path } = Args::parser().with_help_default().parse_env_or_exit();
    let (audio_data, sample_rate) = decode_wav(wav_path);
    use std::io::Write;
    let audio_offset = ((3. * 60. + 35.5) * (sample_rate as f64)) as usize;
    let audio_data_ref = &audio_data[audio_offset..];
    let ines = Ines {
        header: ines::Header {
            num_prg_rom_blocks: 2,
            num_chr_rom_blocks: 1,
            mapper: ines::Mapper::Nrom,
            mirroring: ines::Mirroring::Vertical,
            four_screen_vram: false,
        },
        prg_rom: prg_rom(audio_data_ref),
        chr_rom: chr_rom(audio_data_ref),
    };
    let mut encoded = Vec::new();
    ines.encode(&mut encoded);
    std::io::stdout()
        .lock()
        .write_all(&encoded)
        .expect("Failed to write encoded rom to stdout");
}
