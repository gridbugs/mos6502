const HEADER_BYTES: usize = 16;
const K: usize = 1024;
pub const PRG_ROM_BLOCK_BYTES: usize = 16 * K;
pub const CHR_ROM_BLOCK_BYTES: usize = 8 * K;
const HEADER_CHECKSUM: [u8; 4] = [78, 69, 83, 26];

#[derive(Debug)]
pub struct Header {
    pub num_prg_rom_blocks: u8,
    pub num_chr_rom_blocks: u8,
    pub mapper_number: u8,
}

impl Header {
    fn parse(buffer: &[u8]) -> Self {
        let checksum = &buffer[0..HEADER_CHECKSUM.len()];
        if checksum != &HEADER_CHECKSUM {
            panic!("Invalid checksum");
        }
        let num_prg_rom_blocks = buffer[4];
        let num_chr_rom_blocks = buffer[5];
        let mapper_number = (buffer[7] & 0xF0) | (buffer[6] >> 4);
        Self {
            num_prg_rom_blocks,
            num_chr_rom_blocks,
            mapper_number,
        }
    }
    fn prg_rom_bytes(&self) -> usize {
        self.num_prg_rom_blocks as usize * PRG_ROM_BLOCK_BYTES
    }
    fn chr_rom_bytes(&self) -> usize {
        self.num_chr_rom_blocks as usize * CHR_ROM_BLOCK_BYTES
    }
    fn encode(&self, buffer: &mut Vec<u8>) {
        (&mut buffer[0..HEADER_CHECKSUM.len()]).copy_from_slice(&HEADER_CHECKSUM);
        buffer[4] = self.num_prg_rom_blocks;
        buffer[5] = self.num_chr_rom_blocks;
        buffer[6] = self.mapper_number << 4;
        buffer[7] = self.mapper_number & 0xF0;
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
        let prg_rom_bytes = header.prg_rom_bytes();
        let chr_rom_bytes = header.chr_rom_bytes();
        let prg_rom = data[0..prg_rom_bytes].to_vec();
        let chr_rom = data[prg_rom_bytes..(prg_rom_bytes + chr_rom_bytes)].to_vec();
        Self {
            header,
            prg_rom,
            chr_rom,
        }
    }
    pub fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.resize(
            HEADER_BYTES + self.header.prg_rom_bytes() + self.header.chr_rom_bytes(),
            0,
        );
        self.header.encode(buffer);
        let prg_start = HEADER_BYTES;
        let chr_start = prg_start + self.header.prg_rom_bytes();
        (&mut buffer[prg_start..(prg_start + self.prg_rom.len())]).copy_from_slice(&self.prg_rom);
        (&mut buffer[chr_start..(chr_start + self.chr_rom.len())]).copy_from_slice(&self.chr_rom);
    }
}
