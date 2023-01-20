use ines::Ines;
use mos6502_assembler::{Addr, Block, LabelRelativeOffset};
use mos6502_model::{interrupt_vector, Address};

const PRG_START: Address = 0x8000;
const INTERRUPT_VECTOR_START_PC_OFFSET: Address = interrupt_vector::START_LO - PRG_START;
const INTERRUPT_VECTOR_NMI_OFFSET: Address = interrupt_vector::NMI_LO - PRG_START;

fn program(b: &mut Block, audio_data: &[u8]) {
    use mos6502_model::addressing_mode::*;
    use mos6502_model::assembler_instruction::*;

    b.set_offset(INTERRUPT_VECTOR_START_PC_OFFSET);
    b.label_offset_le("reset");
    b.set_offset(INTERRUPT_VECTOR_NMI_OFFSET);
    b.label_offset_le("nmi");

    b.set_offset(0);

    b.label("nmi");
    b.inst(Rti, ());

    b.label("reset");

    // write 0 to the ppu control register
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x2000));

    // Enable the DMC
    b.inst(Lda(Immediate), 1 << 4);
    b.inst(Sta(Absolute), Addr(0x4015));

    let audio_start = 0x8070;
    let audio_max = 0xFFFA;

    let delay = 95;
    let audio_data_pointer_zero_page = 2;

    b.label("play-audio");

    // read ppu status to clear ppu address latch, then write 0 to the ppu address
    b.inst(Bit(Absolute), Addr(0x2002));
    // write 0 to the ppu address
    b.inst(Lda(Immediate), 0);
    b.inst(Sta(Absolute), Addr(0x2006));
    b.inst(Sta(Absolute), Addr(0x2006));

    // use this position to count the bytes read. Max will be 0x2000 bytes.
    b.inst(Sta(ZeroPage), audio_data_pointer_zero_page);
    b.inst(Sta(ZeroPage), audio_data_pointer_zero_page + 1);

    // read trash value from ppu
    b.inst(Lda(Absolute), Addr(0x2007));

    b.label("audio-loop-start-ppu");

    b.inst(Ldx(Immediate), delay);
    b.label("delay2");
    b.inst(Dex, ());
    b.inst(Bne, LabelRelativeOffset("delay2"));

    b.inst(Lda(Absolute), Addr(0x2007));
    b.inst(Sta(Absolute), Addr(0x4011));

    b.inst(Jsr(Absolute), "incr-counter");

    // test that we're still in bounds
    b.inst(Lda(Immediate), mos6502_model::address::hi(0x2000));
    b.inst(Cmp(ZeroPage), audio_data_pointer_zero_page + 1);
    b.inst(Bne, LabelRelativeOffset("audio-loop-start-ppu"));
    b.inst(Lda(Immediate), mos6502_model::address::lo(0x2000));
    b.inst(Cmp(ZeroPage), audio_data_pointer_zero_page);
    b.inst(Bne, LabelRelativeOffset("audio-loop-start-ppu"));

    // play the rest of the sample from prg rom
    b.inst(Lda(Immediate), mos6502_model::address::lo(audio_start));
    b.inst(Sta(ZeroPage), audio_data_pointer_zero_page);
    b.inst(Lda(Immediate), mos6502_model::address::hi(audio_start));
    b.inst(Sta(ZeroPage), audio_data_pointer_zero_page + 1);

    b.label("audio-loop-start");

    b.inst(Ldx(Immediate), delay);
    b.label("delay1");
    b.inst(Dex, ());
    b.inst(Bne, LabelRelativeOffset("delay1"));

    // load the data from the audio buffer and send it to the dmc
    b.inst(Lda(XIndexedIndirect), audio_data_pointer_zero_page);
    b.inst(Sta(Absolute), Addr(0x4011));

    b.inst(Jsr(Absolute), "incr-counter");

    // test that we're still in bounds
    b.inst(Lda(Immediate), mos6502_model::address::hi(audio_max));
    b.inst(Cmp(ZeroPage), audio_data_pointer_zero_page + 1);
    b.inst(Bne, LabelRelativeOffset("audio-loop-start"));
    b.inst(Lda(Immediate), mos6502_model::address::lo(audio_max));
    b.inst(Cmp(ZeroPage), audio_data_pointer_zero_page);
    b.inst(Bne, LabelRelativeOffset("audio-loop-start"));

    b.label("end");
    b.inst(Jmp(Absolute), "end");

    b.label("incr-counter");
    // increment the pointer to the next audio sample
    b.inst(Clc, ());
    b.inst(Lda(ZeroPage), audio_data_pointer_zero_page);
    b.inst(Adc(Immediate), 1);
    b.inst(Sta(ZeroPage), audio_data_pointer_zero_page);
    b.inst(Lda(ZeroPage), audio_data_pointer_zero_page + 1);
    b.inst(Adc(Immediate), 0);
    b.inst(Sta(ZeroPage), audio_data_pointer_zero_page + 1);
    b.inst(Rts, ());

    assert!(b.offset() < audio_start);
    b.set_offset(audio_start - PRG_START);

    let num_samples = audio_max as usize - audio_start as usize;
    for &sample in &audio_data[ines::CHR_ROM_BLOCK_BYTES..(ines::CHR_ROM_BLOCK_BYTES + num_samples)]
    {
        b.literal_byte(sample);
    }

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
