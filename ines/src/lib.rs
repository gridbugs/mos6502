const HEADER_BYTES: usize = 16;
const K: usize = 1024;
pub const PRG_ROM_BLOCK_BYTES: usize = 16 * K;
pub const CHR_ROM_BLOCK_BYTES: usize = 8 * K;
const HEADER_CHECKSUM: [u8; 4] = [78, 69, 83, 26];

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreenVram,
}

#[derive(Debug, Clone, Copy)]
pub enum Mapper {
    Nrom,
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    UnimplementedMapper { code: u8 },
}

impl Mapper {
    fn encode(self) -> u8 {
        match self {
            Mapper::Nrom => 0,
        }
    }
    fn decode(code: u8) -> Result<Self, Error> {
        match code {
            0 => Ok(Mapper::Nrom),
            other => Err(Error::UnimplementedMapper { code: other }),
        }
    }
}

#[derive(Debug)]
pub struct Header {
    pub num_prg_rom_blocks: u8,
    pub num_chr_rom_blocks: u8,
    pub mapper: Mapper,
    pub mirroring: Mirroring,
}

impl Header {
    fn parse(buffer: &[u8]) -> Result<Self, Error> {
        let checksum = &buffer[0..HEADER_CHECKSUM.len()];
        if checksum != &HEADER_CHECKSUM {
            panic!("Invalid checksum");
        }
        let mirroring = if buffer[6] & (1 << 3) != 0 {
            Mirroring::FourScreenVram
        } else if buffer[6] & (1 << 0) != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
        let num_prg_rom_blocks = buffer[4];
        let num_chr_rom_blocks = buffer[5];
        let mapper_number = (buffer[7] & 0xF0) | (buffer[6] >> 4);
        let mapper = Mapper::decode(mapper_number)?;
        Ok(Self {
            num_prg_rom_blocks,
            num_chr_rom_blocks,
            mapper,
            mirroring,
        })
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
        let mapper_number = self.mapper.encode();
        buffer[6] = mapper_number << 4;
        buffer[7] = mapper_number & 0xF0;
        match self.mirroring {
            Mirroring::Horizontal => {
                buffer[6] |= (1 << 0);
                buffer[6] &= !(1 << 3);
            }
            Mirroring::Vertical => {
                buffer[6] &= !(1 << 0);
                buffer[6] &= !(1 << 3);
            }
            Mirroring::FourScreenVram => buffer[6] |= (1 << 3),
        }
    }
}

pub struct Ines {
    pub header: Header,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl Ines {
    pub fn parse(buffer: &[u8]) -> Result<Self, Error> {
        let header_raw = &buffer[0..HEADER_BYTES];
        let data = &buffer[HEADER_BYTES..];
        let header = Header::parse(header_raw)?;
        let prg_rom_bytes = header.prg_rom_bytes();
        let chr_rom_bytes = header.chr_rom_bytes();
        let prg_rom = data[0..prg_rom_bytes].to_vec();
        let chr_rom = data[prg_rom_bytes..(prg_rom_bytes + chr_rom_bytes)].to_vec();
        Ok(Self {
            header,
            prg_rom,
            chr_rom,
        })
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
