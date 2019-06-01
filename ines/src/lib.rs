const HEADER_BYTES: usize = 16;
const K: usize = 1024;
pub const PRG_ROM_BANK_BYTES: usize = 16 * K;
pub const CHR_ROM_BANK_BYTES: usize = 8 * K;
const HEADER_CHECKSUM: [u8; 4] = [78, 69, 83, 26];

#[derive(Debug)]
pub struct Header {
    pub num_prg_rom_banks: u8,
    pub num_chr_rom_banks: u8,
    pub mapper_number: u8,
}

impl Header {
    fn parse(buffer: &[u8]) -> Self {
        let checksum = &buffer[0..HEADER_CHECKSUM.len()];
        if checksum != &HEADER_CHECKSUM {
            panic!("Invalid checksum");
        }
        let num_prg_rom_banks = buffer[4];
        let num_chr_rom_banks = buffer[5];
        let mapper_number = (buffer[6] & 0xf0) | (buffer[6] >> 4);
        Self {
            num_prg_rom_banks,
            num_chr_rom_banks,
            mapper_number,
        }
    }
    fn prg_rom_bytes(&self) -> usize {
        self.num_prg_rom_banks as usize * PRG_ROM_BANK_BYTES
    }
    fn chr_rom_bytes(&self) -> usize {
        self.num_chr_rom_banks as usize * CHR_ROM_BANK_BYTES
    }
    fn encode(&self, buffer: &mut Vec<u8>) {
        (&mut buffer[0..HEADER_CHECKSUM.len()]).copy_from_slice(&HEADER_CHECKSUM);
        buffer[4] = self.num_prg_rom_banks;
        buffer[5] = self.num_chr_rom_banks;
    }
}

pub struct PrgRomBank {
    pub rom: [u8; PRG_ROM_BANK_BYTES],
}

pub struct ChrRomBank {
    pub rom: [u8; CHR_ROM_BANK_BYTES],
}

pub struct Ines {
    pub header: Header,
    pub prg_rom: Vec<PrgRomBank>,
    pub chr_rom: Vec<ChrRomBank>,
}

impl Ines {
    pub fn parse(buffer: &[u8]) -> Self {
        let header_raw = &buffer[0..HEADER_BYTES];
        let header = Header::parse(header_raw);
        let prg_rom_size = header.prg_rom_bytes();
        let chr_rom_size = header.chr_rom_bytes();
        let prg_rom_start = HEADER_BYTES;
        let chr_rom_start = prg_rom_start + prg_rom_size;
        let prg_rom_raw = &buffer[prg_rom_start..(prg_rom_start + prg_rom_size)];
        let chr_rom_raw = &buffer[chr_rom_start..(chr_rom_start + chr_rom_size)];
        let prg_rom = (0..(header.num_prg_rom_banks as usize))
            .map(|i| {
                let start = i * PRG_ROM_BANK_BYTES;
                let end = start + PRG_ROM_BANK_BYTES;
                let mut rom = [0; PRG_ROM_BANK_BYTES];
                rom.copy_from_slice(&prg_rom_raw[start..end]);
                PrgRomBank { rom }
            })
            .collect::<Vec<_>>();
        let chr_rom = (0..(header.num_chr_rom_banks as usize))
            .map(|i| {
                let start = i * CHR_ROM_BANK_BYTES;
                let end = start + CHR_ROM_BANK_BYTES;
                let mut rom = [0; CHR_ROM_BANK_BYTES];
                rom.copy_from_slice(&chr_rom_raw[start..end]);
                ChrRomBank { rom }
            })
            .collect::<Vec<_>>();
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
        let prg_rom_size = self.header.prg_rom_bytes();
        let chr_rom_size = self.header.chr_rom_bytes();
        let prg_rom_start = HEADER_BYTES;
        let chr_rom_start = prg_rom_start + prg_rom_size;
        {
            let prg_rom_buffer = &mut buffer[prg_rom_start..(prg_rom_start + prg_rom_size)];
            self.prg_rom.iter().enumerate().for_each(|(i, bank)| {
                let start = i * PRG_ROM_BANK_BYTES;
                let end = start + PRG_ROM_BANK_BYTES;
                (&mut prg_rom_buffer[start..end]).copy_from_slice(&bank.rom);
            });
        }
        {
            let chr_rom_buffer = &mut buffer[chr_rom_start..(chr_rom_start + chr_rom_size)];
            self.chr_rom.iter().enumerate().for_each(|(i, bank)| {
                let start = i * CHR_ROM_BANK_BYTES;
                let end = start + CHR_ROM_BANK_BYTES;
                (&mut chr_rom_buffer[start..end]).copy_from_slice(&bank.rom);
            });
        }
    }
}
