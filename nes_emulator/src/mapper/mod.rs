use ines;
use mos6502::Address;

pub type PpuAddress = u16;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NameTableChoice {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PatternTableChoice {
    PatternTable0,
    PatternTable1,
}

impl PatternTableChoice {
    pub fn base_address(self) -> PpuAddress {
        self as PpuAddress * (PATTERN_TABLE_BYTES as PpuAddress)
    }
}

const PATTERN_TABLE_BYTES: usize = 0x1000;
const NAME_TABLE_BYTES: usize = 0x400;

pub trait PpuMapper {
    fn ppu_write_u8(&mut self, address: PpuAddress, data: u8);
    fn ppu_read_u8(&self, address: PpuAddress) -> u8;
    fn ppu_pattern_table(&self, choice: PatternTableChoice) -> &[u8];
    fn ppu_name_table(&self, choice: NameTableChoice) -> &[u8];
    fn ppu_palette_ram(&self) -> &[u8];
}

pub trait CpuMapper {
    fn cpu_read_u8(&mut self, address: Address) -> u8;
    fn cpu_write_u8(&mut self, address: Address, data: u8);
    fn cpu_read_u8_read_only(&self, address: Address) -> u8;
}

pub trait Mapper: CpuMapper + PpuMapper {
    fn clone_dynamic(&self) -> Dynamic;
}

#[derive(Debug)]
pub enum Error {
    UnexpectedPrgRomSize,
    UnexpectedChrRomSize,
}

const PALETTE_RAM_BYTES: usize = 0x20;
#[derive(Default, Clone, Serialize, Deserialize)]
struct PaletteRam {
    ram: [u8; PALETTE_RAM_BYTES],
}

impl PaletteRam {
    fn read_u8(&self, offset: u8) -> u8 {
        self.ram[offset as usize % PALETTE_RAM_BYTES]
    }
    fn write_u8(&mut self, offset: u8, data: u8) {
        self.ram[offset as usize % PALETTE_RAM_BYTES] = data;
    }
}

pub mod nrom;

#[derive(Serialize, Deserialize)]
pub enum Dynamic {
    NromHorizontal(nrom::Nrom<mirroring::Horizontal>),
    NromVertical(nrom::Nrom<mirroring::Vertical>),
    NromFourScreenVram(nrom::Nrom<mirroring::FourScreenVram>),
}

impl Dynamic {
    pub fn new(
        ines::Ines {
            header,
            prg_rom,
            chr_rom,
        }: ines::Ines,
    ) -> Result<Self, Error> {
        match (header.mapper, header.mirroring) {
            (ines::Mapper::Nrom, ines::Mirroring::Horizontal) => Ok(Dynamic::NromHorizontal(
                nrom::Nrom::new(mirroring::Horizontal, &prg_rom, &chr_rom)?,
            )),
            (ines::Mapper::Nrom, ines::Mirroring::Vertical) => Ok(Dynamic::NromVertical(
                nrom::Nrom::new(mirroring::Vertical, &prg_rom, &chr_rom)?,
            )),
            (ines::Mapper::Nrom, ines::Mirroring::FourScreenVram) => {
                Ok(Dynamic::NromFourScreenVram(nrom::Nrom::new(
                    mirroring::FourScreenVram,
                    &prg_rom,
                    &chr_rom,
                )?))
            }
            _ => panic!(),
        }
    }
}

pub mod mirroring {
    use super::nrom;
    use super::{Dynamic, NameTableChoice, PpuAddress, NAME_TABLE_BYTES};

    pub trait Mirroring: Clone + Copy {
        fn name_table_base_address(name_table: NameTableChoice) -> PpuAddress;
    }

    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub struct Horizontal;

    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub struct Vertical;

    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub struct FourScreenVram;

    impl Mirroring for Horizontal {
        fn name_table_base_address(name_table: NameTableChoice) -> PpuAddress {
            (name_table as PpuAddress / 2) * (NAME_TABLE_BYTES as PpuAddress)
        }
    }
    impl Mirroring for Vertical {
        fn name_table_base_address(name_table: NameTableChoice) -> PpuAddress {
            (name_table as PpuAddress % 2) * (NAME_TABLE_BYTES as PpuAddress)
        }
    }
    impl Mirroring for FourScreenVram {
        fn name_table_base_address(name_table: NameTableChoice) -> PpuAddress {
            name_table as PpuAddress * (NAME_TABLE_BYTES as PpuAddress)
        }
    }

    pub trait SpecificMapperDynamic: Mirroring {
        fn nrom(nrom: nrom::Nrom<Self>) -> Dynamic;
    }

    impl SpecificMapperDynamic for Horizontal {
        fn nrom(nrom: nrom::Nrom<Self>) -> Dynamic {
            Dynamic::NromHorizontal(nrom)
        }
    }
    impl SpecificMapperDynamic for Vertical {
        fn nrom(nrom: nrom::Nrom<Self>) -> Dynamic {
            Dynamic::NromVertical(nrom)
        }
    }
    impl SpecificMapperDynamic for FourScreenVram {
        fn nrom(nrom: nrom::Nrom<Self>) -> Dynamic {
            Dynamic::NromFourScreenVram(nrom)
        }
    }
}
