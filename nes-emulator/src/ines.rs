const HEADER_BYTES: usize = 16;
const PRG_ROM_BLOCK_BYTES: usize = 16384;
const CHR_ROM_BLOCK_BYTES: usize = 8192;
const HEADER_CHECKSUM: [u8; 4] = [78, 69, 83, 26];

pub struct Header {
    num_prg_rom_blocks: usize,
    num_chr_rom_blocks: usize,
}

impl Header {
    fn parse(buffer: &[u8]) -> Self {
        let checksum = &buffer[0..HEADER_CHECKSUM.len()];
        if checksum != &HEADER_CHECKSUM {
            panic!("Invalid checksum");
        }
        let num_prg_rom_blocks = buffer[4] as usize;
        let num_chr_rom_blocks = buffer[5] as usize;
        Self {
            num_prg_rom_blocks,
            num_chr_rom_blocks,
        }
    }
}

pub struct Ines {
    pub header: Header,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl Ines {
    pub fn parse(buffer: &[u8]) -> Self {
        let header_raw = &buffer[0..HEADER_BYTES];
        let data = &buffer[HEADER_BYTES..];
        let header = Header::parse(header_raw);
        let prg_rom_bytes = header.num_prg_rom_blocks * PRG_ROM_BLOCK_BYTES;
        let chr_rom_bytes = header.num_chr_rom_blocks * CHR_ROM_BLOCK_BYTES;
        let prg_rom = data[0..prg_rom_bytes].to_vec();
        let chr_rom = data[prg_rom_bytes..(prg_rom_bytes + chr_rom_bytes)].to_vec();
        Self {
            header,
            prg_rom,
            chr_rom,
        }
    }
}
