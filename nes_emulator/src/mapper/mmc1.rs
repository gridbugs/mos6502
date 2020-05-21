use crate::mapper::Error;
use crate::mapper::PpuAddress;
use crate::mapper::{CpuMapper, Mapper, PpuMapper};
use crate::mapper::{NameTableChoice, PaletteRam, PatternTableChoice};
use crate::mapper::{PersistentState, PersistentStateError};
use crate::nes::Nes;
use crate::ppu::{name_table_mirroring, NAME_TABLE_BYTES};
use crate::DynamicNes;
use mos6502_model::Address;
use serde::{Deserialize, Serialize};
use serde_big_array::big_array;

big_array! { BigArray; }

const PRG_RAM_BYTES: usize = 8 * 1024;
const PRG_ROM_BANK_BYTES: usize = 16 * 1024;
const CHR_ROM_BANK_BYTES: usize = 4 * 1024;
const NAME_TABLE_RAM_BYTES: usize = 2 * NAME_TABLE_BYTES;
const MAX_NUM_SHIFT_REGISTER_WRITES: u8 = 4;

mod registers {
    pub const CONTROL: u8 = 0;
    pub const CHR_BANK0: u8 = 1;
    pub const CHR_BANK1: u8 = 2;
    pub const PRG_BANK: u8 = 3;
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum Mirroring {
    SingleScreenLower,
    SingleScreenUpper,
    Horizontal,
    Vertical,
}

impl Mirroring {
    fn name_table_base_address(self, name_table: NameTableChoice) -> PpuAddress {
        use name_table_mirroring::physical_base_address;
        use Mirroring::*;
        match self {
            SingleScreenLower => physical_base_address::single_screen_lower(),
            SingleScreenUpper => physical_base_address::single_screen_upper(),
            Horizontal => physical_base_address::horizontal(name_table),
            Vertical => physical_base_address::vertical(name_table),
        }
    }
    fn name_table_physical_offset(self, virtual_offset: PpuAddress) -> PpuAddress {
        use name_table_mirroring::physical_offset;
        use Mirroring::*;
        match self {
            SingleScreenLower => physical_offset::single_screen_lower(virtual_offset),
            SingleScreenUpper => physical_offset::single_screen_upper(virtual_offset),
            Horizontal => physical_offset::horizontal(virtual_offset),
            Vertical => physical_offset::vertical(virtual_offset),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct PrgRomBank {
    #[serde(with = "BigArray")]
    rom: [u8; PRG_ROM_BANK_BYTES],
}

#[derive(Serialize, Deserialize, Clone)]
struct ChrRomBank {
    #[serde(with = "BigArray")]
    rom: [u8; CHR_ROM_BANK_BYTES],
}

#[derive(Serialize, Deserialize, Clone)]
enum PrgRomBankMode {
    SwitchBoth,
    SwitchLower,
    SwitchUpper,
}

#[derive(Serialize, Deserialize, Clone)]
enum ChrRomBankMode {
    SwitchTogether,
    SwitchSeperate,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mmc1 {
    prg_rom_banks: Vec<PrgRomBank>,
    chr_rom_banks: Vec<ChrRomBank>,
    #[serde(with = "BigArray")]
    prg_ram: [u8; PRG_RAM_BYTES],
    #[serde(with = "BigArray")]
    name_table_ram: [u8; NAME_TABLE_RAM_BYTES],
    palette_ram: PaletteRam,
    prg_rom_bank0: usize,
    prg_rom_bank1: usize,
    chr_rom_bank0: usize,
    chr_rom_bank1: usize,
    mirroring: Mirroring,
    prg_rom_bank_mode: PrgRomBankMode,
    chr_rom_bank_mode: ChrRomBankMode,
    shift_register: u8,
    num_shift_register_writes: u8,
}

impl Mmc1 {
    fn make_prg_rom_banks(prg_rom_raw: &[u8]) -> Result<Vec<PrgRomBank>, Error> {
        let num_prg_rom_banks = prg_rom_raw.len() / PRG_ROM_BANK_BYTES;
        if num_prg_rom_banks * PRG_ROM_BANK_BYTES != prg_rom_raw.len() {
            return Err(Error::UnexpectedPrgRomSize);
        }
        let mut prg_rom_banks = (0..num_prg_rom_banks)
            .map(|i| {
                let start = i * PRG_ROM_BANK_BYTES;
                let end = start + PRG_ROM_BANK_BYTES;
                let mut rom = [0; PRG_ROM_BANK_BYTES];
                rom.copy_from_slice(&prg_rom_raw[start..end]);
                PrgRomBank { rom }
            })
            .collect::<Vec<_>>();
        match num_prg_rom_banks {
            0 => {
                prg_rom_banks.push(PrgRomBank {
                    rom: [0; PRG_ROM_BANK_BYTES],
                });
                prg_rom_banks.push(PrgRomBank {
                    rom: [0; PRG_ROM_BANK_BYTES],
                });
            }
            1 => {
                prg_rom_banks.push(PrgRomBank {
                    rom: [0; PRG_ROM_BANK_BYTES],
                });
            }
            _ => (),
        }
        Ok(prg_rom_banks)
    }
    fn make_chr_rom_banks(chr_rom_raw: &[u8]) -> Result<Vec<ChrRomBank>, Error> {
        let num_chr_rom_banks = chr_rom_raw.len() / CHR_ROM_BANK_BYTES;
        if num_chr_rom_banks * CHR_ROM_BANK_BYTES != chr_rom_raw.len() {
            return Err(Error::UnexpectedChrRomSize);
        }
        let mut chr_rom_banks = (0..num_chr_rom_banks)
            .map(|i| {
                let start = i * CHR_ROM_BANK_BYTES;
                let end = start + CHR_ROM_BANK_BYTES;
                let mut rom = [0; CHR_ROM_BANK_BYTES];
                rom.copy_from_slice(&chr_rom_raw[start..end]);
                ChrRomBank { rom }
            })
            .collect::<Vec<_>>();
        match num_chr_rom_banks {
            0 => {
                chr_rom_banks.push(ChrRomBank {
                    rom: [0; CHR_ROM_BANK_BYTES],
                });
                chr_rom_banks.push(ChrRomBank {
                    rom: [0; CHR_ROM_BANK_BYTES],
                });
            }
            1 => {
                chr_rom_banks.push(ChrRomBank {
                    rom: [0; CHR_ROM_BANK_BYTES],
                });
            }
            _ => (),
        }
        Ok(chr_rom_banks)
    }
    pub fn new(prg_rom_raw: &[u8], chr_rom_raw: &[u8]) -> Result<Self, Error> {
        let prg_rom_banks = Self::make_prg_rom_banks(prg_rom_raw)?;
        let chr_rom_banks = Self::make_chr_rom_banks(chr_rom_raw)?;
        let palette_ram = PaletteRam::default();
        let prg_ram = [0; PRG_RAM_BYTES];
        let name_table_ram = [0; NAME_TABLE_RAM_BYTES];
        let mirroring = Mirroring::SingleScreenLower;
        let prg_rom_bank0 = 0;
        let chr_rom_bank0 = 0;
        let prg_rom_bank1 = prg_rom_banks.len() - 1;
        let chr_rom_bank1 = 1;
        let prg_rom_bank_mode = PrgRomBankMode::SwitchLower;
        let chr_rom_bank_mode = ChrRomBankMode::SwitchTogether;
        let shift_register = 0;
        let num_shift_register_writes = 0;
        Ok(Self {
            palette_ram,
            prg_rom_banks,
            chr_rom_banks,
            prg_ram,
            name_table_ram,
            mirroring,
            prg_rom_bank0,
            chr_rom_bank0,
            prg_rom_bank1,
            chr_rom_bank1,
            prg_rom_bank_mode,
            chr_rom_bank_mode,
            shift_register,
            num_shift_register_writes,
        })
    }
    fn write_control_register(&mut self, data: u8) {
        self.mirroring = match data & 3 {
            0 => Mirroring::SingleScreenLower,
            1 => Mirroring::SingleScreenUpper,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        };
        self.prg_rom_bank_mode = match data.wrapping_shr(2) & 3 {
            0 | 1 => PrgRomBankMode::SwitchBoth,
            2 => {
                self.prg_rom_bank0 = 0;
                PrgRomBankMode::SwitchUpper
            }
            3 => {
                self.prg_rom_bank1 = self.prg_rom_banks.len() - 1;
                PrgRomBankMode::SwitchLower
            }
            _ => unreachable!(),
        };
        self.chr_rom_bank_mode = match data.wrapping_shr(4) & 1 {
            0 => ChrRomBankMode::SwitchTogether,
            1 => ChrRomBankMode::SwitchSeperate,
            _ => unreachable!(),
        };
    }
    fn write_chr_bank0(&mut self, data: u8) {
        match self.chr_rom_bank_mode {
            ChrRomBankMode::SwitchSeperate => {
                self.chr_rom_bank0 = data as usize;
            }
            ChrRomBankMode::SwitchTogether => {
                self.chr_rom_bank0 = (data & (!1)) as usize;
                self.chr_rom_bank1 = (data | 1) as usize;
            }
        }
    }
    fn write_chr_bank1(&mut self, data: u8) {
        match self.chr_rom_bank_mode {
            ChrRomBankMode::SwitchSeperate => {
                self.chr_rom_bank0 = data as usize;
            }
            ChrRomBankMode::SwitchTogether => (),
        }
    }
    fn write_prg_bank(&mut self, data: u8) {
        let bank = data & 0xF;
        match self.prg_rom_bank_mode {
            PrgRomBankMode::SwitchBoth => {
                self.prg_rom_bank0 = (bank & (!1)) as usize;
                self.prg_rom_bank1 = (bank | 1) as usize;
            }
            PrgRomBankMode::SwitchLower => {
                self.prg_rom_bank0 = bank as usize;
            }
            PrgRomBankMode::SwitchUpper => {
                self.prg_rom_bank1 = bank as usize;
            }
        }
    }
    fn write_register(&mut self, register: u8, data: u8) {
        match register {
            registers::CONTROL => self.write_control_register(data),
            registers::CHR_BANK0 => self.write_chr_bank0(data),
            registers::CHR_BANK1 => self.write_chr_bank1(data),
            registers::PRG_BANK => self.write_prg_bank(data),
            _ => unreachable!(),
        }
    }
    fn write_u8(&mut self, address: Address, data: u8) {
        if data & 1 << 7 != 0 {
            self.num_shift_register_writes = 0;
            self.shift_register = 0;
        } else {
            if self.num_shift_register_writes == MAX_NUM_SHIFT_REGISTER_WRITES {
                let value = ((data & 1) << 4) | self.shift_register;
                let offset = (address.wrapping_shr(13) & 3) as u8;
                self.write_register(offset, value);
                self.num_shift_register_writes = 0;
                self.shift_register = 0;
            } else {
                self.shift_register |= (data & 1) << self.num_shift_register_writes;
                self.num_shift_register_writes += 1;
            }
        }
    }
}

impl PpuMapper for Mmc1 {
    fn ppu_write_u8(&mut self, address: PpuAddress, data: u8) {
        let address = address % 0x4000;
        match address {
            0x0000..=0x0FFF => self.chr_rom_banks[self.chr_rom_bank0].rom[address as usize] = data,
            0x1000..=0x1FFF => {
                self.chr_rom_banks[self.chr_rom_bank1].rom[(address & 0x0FFF) as usize] = data;
            }
            0x2000..=0x3EFF => {
                let physical_offset =
                    self.mirroring.name_table_physical_offset(address & 0x0FFF) as usize;
                self.name_table_ram[physical_offset] = data
            }
            0x3F00..=0x3FFF => self.palette_ram.write_u8(address as u8, data),
            _ => unreachable!(),
        }
    }
    fn ppu_read_u8(&self, address: PpuAddress) -> u8 {
        let address = address % 0x4000;
        match address {
            0x0000..=0x0FFF => self.chr_rom_banks[self.chr_rom_bank0].rom[address as usize],
            0x1000..=0x1FFF => {
                self.chr_rom_banks[self.chr_rom_bank1].rom[(address & 0x0FFF) as usize]
            }
            0x2000..=0x3EFF => {
                let physical_offset =
                    self.mirroring.name_table_physical_offset(address & 0x0FFF) as usize;
                self.name_table_ram[physical_offset]
            }
            0x3F00..=0x3FFF => self.palette_ram.read_u8(address as u8),
            _ => unreachable!(),
        }
    }
    fn ppu_pattern_table(&self, choice: PatternTableChoice) -> &[u8] {
        let bank = match choice {
            PatternTableChoice::PatternTable0 => self.chr_rom_bank0,
            PatternTableChoice::PatternTable1 => self.chr_rom_bank1,
        };
        &self.chr_rom_banks[bank].rom
    }
    fn ppu_name_table(&self, choice: NameTableChoice) -> &[u8] {
        let address_offset = self.mirroring.name_table_base_address(choice) as usize;
        &self.name_table_ram[address_offset..(address_offset + NAME_TABLE_BYTES)]
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
            0x6000..=0x7FFF => self.prg_ram[address as usize % 0x2000] = data,
            0x8000..=0xFFFF => self.write_u8(address, data),
            other => eprintln!(
                "unexpected cartridge write of {:X} to address {:X}",
                data, other
            ),
        }
    }
    fn cpu_read_u8_read_only(&self, address: Address) -> u8 {
        match address {
            0x6000..=0x7FFF => self.prg_ram[address as usize % 0x2000],
            0x8000..=0xBFFF => {
                self.prg_rom_banks[self.prg_rom_bank0].rom[(address as usize) % 0x4000]
            }
            0xC000..=0xFFFF => {
                self.prg_rom_banks[self.prg_rom_bank1].rom[(address as usize) % 0x4000]
            }
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
    fn save_persistent_state(&self) -> Option<PersistentState> {
        Some(PersistentState::BatteryBackedRam(self.prg_ram.to_vec()))
    }
    fn load_persistent_state(
        &mut self,
        persistent_state: &PersistentState,
    ) -> Result<(), PersistentStateError> {
        match persistent_state {
            PersistentState::BatteryBackedRam(data) => self.prg_ram.copy_from_slice(&data),
        }
        Ok(())
    }
}
