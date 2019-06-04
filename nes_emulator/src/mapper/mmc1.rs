use crate::mapper::Error;
use crate::mapper::PpuAddress;
use crate::mapper::PATTERN_TABLE_BYTES;
use crate::mapper::{CpuMapper, Mapper, PpuMapper};
use crate::mapper::{NameTableChoice, PaletteRam, PatternTableChoice};
use crate::nes::Nes;
use crate::DynamicNes;
use mos6502::Address;

enum Mirroring {
    OneScreenLower,
    OneScreenUpper,
    Vertical,
    Horizontal,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mmc1 {
    palette_ram: PaletteRam,
}

impl Mmc1 {
    pub fn new(prg_rom_raw: &[u8], chr_rom_raw: &[u8]) -> Result<Self, Error> {
        let palette_ram = PaletteRam::default();
        Ok(Self { palette_ram })
    }
}

impl PpuMapper for Mmc1 {
    fn ppu_write_u8(&mut self, address: PpuAddress, data: u8) {
        let address = address % 0x4000;
        match address {
            0x0000..=0x3EFF => unimplemented!(),
            0x3F00..=0x3FFF => self.palette_ram.write_u8(address as u8, data),
            _ => unreachable!(),
        }
    }
    fn ppu_read_u8(&self, address: PpuAddress) -> u8 {
        let address = address % 0x4000;
        match address {
            0x0000..=0x3EFF => unimplemented!(),
            0x3F00..=0x3FFF => self.palette_ram.read_u8(address as u8),
            _ => unreachable!(),
        }
    }
    fn ppu_pattern_table(&self, choice: PatternTableChoice) -> &[u8] {
        unimplemented!()
    }
    fn ppu_name_table(&self, choice: NameTableChoice) -> &[u8] {
        unimplemented!()
    }
    fn ppu_palette_ram(&self) -> &[u8] {
        &self.palette_ram.ram
    }
}

impl CpuMapper for Mmc1 {
    fn cpu_read_u8(&mut self, address: Address) -> u8 {
        self.cpu_read_u8_read_only(address)
    }
    fn cpu_write_u8(&mut self, address: Address, data: u8) {
        match address {
            other => eprintln!(
                "unexpected cartridge write of {:X} to address {:X}",
                data, other
            ),
        }
    }
    fn cpu_read_u8_read_only(&self, address: Address) -> u8 {
        match address {
            other => {
                eprintln!("unexpected cartridge read from address {:X}", other);
                0
            }
        }
    }
}

impl Mapper for Mmc1 {
    fn clone_dynamic_nes(nes: &Nes<Self>) -> DynamicNes {
        DynamicNes::Mmc1(nes.clone())
    }
}
