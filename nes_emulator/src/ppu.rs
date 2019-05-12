use glutin_frontend::Pixels;
use mos6502::address;
use mos6502::machine::{Address, Memory};
use std::fmt;

const OAM_SPRITE_BYTES: usize = 4;
const OAM_NUM_SPRITES: usize = 64;
const OAM_BYTES: usize = OAM_SPRITE_BYTES * OAM_NUM_SPRITES;

#[derive(Serialize, Deserialize)]
pub struct Oam {
    ram: Vec<u8>,
}

impl Oam {
    pub fn new() -> Self {
        Self {
            ram: [0; OAM_BYTES].to_vec(),
        }
    }
    pub fn dma<M: Memory>(&mut self, memory: &mut M, start_address_hi: u8) {
        let start_address = address::from_u8_lo_hi(0, start_address_hi);
        for (address, oam_byte) in (start_address..start_address.wrapping_add(OAM_BYTES as Address))
            .zip(self.ram.iter_mut())
        {
            *oam_byte = memory.read_u8(address)
        }
    }
}

impl fmt::Debug for Oam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for sprite_index in 0..OAM_NUM_SPRITES {
            let sprite_address = sprite_index * OAM_SPRITE_BYTES;
            let sprite = &self.ram[sprite_address..sprite_address + OAM_SPRITE_BYTES];
            writeln!(f, "{:02}: {:02X?}", sprite_index, sprite)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ppu {
    next_address_write_is_hi_byte: bool,
    address: u16,
    address_increment: u8,
    vblank_nmi: bool,
    sprite_pattern_table: PatternTableChoice,
    background_pattern_table: PatternTableChoice,
    read_buffer: u8,
}

pub type PpuAddress = u16;
pub const PATTERN_TABLE_BYTES: usize = 0x1000;
pub const NAME_TABLE_BYTES: usize = 0x400;
pub const PALETTE_START: PpuAddress = 0x3F00;

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

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NameTableChoice {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl NameTableChoice {
    pub fn address_offset_horizontal_mirror(self) -> PpuAddress {
        (self as PpuAddress / 2) * (NAME_TABLE_BYTES as PpuAddress)
    }
}

pub trait PpuMemory {
    fn write_u8(&mut self, address: PpuAddress, data: u8);
    fn read_u8(&self, address: PpuAddress) -> u8;
    fn pattern_table(&self, choice: PatternTableChoice) -> &[u8];
    fn name_table(&self, choice: NameTableChoice) -> &[u8];
    fn palette_ram(&self) -> &[u8];
}

pub trait RenderOutput {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8);
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            next_address_write_is_hi_byte: true,
            address: 0,
            address_increment: 1,
            vblank_nmi: false,
            sprite_pattern_table: PatternTableChoice::PatternTable0,
            background_pattern_table: PatternTableChoice::PatternTable0,
            read_buffer: 0,
        }
    }
    pub fn write_control(&mut self, data: u8) {
        self.address_increment = if data & control::flag::ADDRESS_INCREMENT != 0 {
            32
        } else {
            1
        };
        self.sprite_pattern_table = if (data & control::flag::SPRITE_PATTERN_TABLE) == 0 {
            PatternTableChoice::PatternTable0
        } else {
            PatternTableChoice::PatternTable1
        };
        self.background_pattern_table = if (data & control::flag::BACKGROUND_PATTERN_TABLE) == 0 {
            PatternTableChoice::PatternTable0
        } else {
            PatternTableChoice::PatternTable1
        };
        self.vblank_nmi = data & control::flag::VBLANK_NMI != 0;
    }
    pub fn write_mask(&mut self, _data: u8) {}
    pub fn read_status(&mut self) -> u8 {
        self.address = 0;
        self.next_address_write_is_hi_byte = true;
        status::flag::VBLANK
    }
    pub fn write_oam_address(&mut self, data: u8) {
        //println!("write oam address {}", data);
    }
    pub fn write_oam_data(&mut self, data: u8) {
        println!("write oam data {}", data);
    }
    pub fn read_oam_data(&mut self) -> u8 {
        println!("read oam data");
        0
    }
    pub fn write_scroll(&mut self, data: u8) {
        //println!("WRITE SCROLL {:X}", data);
    }
    pub fn write_address(&mut self, data: u8) {
        let shift = self.next_address_write_is_hi_byte as u32 * 8;
        let mask = 0xFF00u16.wrapping_shr(shift);
        self.address = (self.address & mask) | (data as u16).wrapping_shl(shift);
        self.next_address_write_is_hi_byte = !self.next_address_write_is_hi_byte;
    }
    pub fn write_data<M: PpuMemory>(&mut self, memory: &mut M, data: u8) {
        memory.write_u8(self.address, data);
        self.address = self.address.wrapping_add(self.address_increment as u16);
    }
    pub fn read_data<M: PpuMemory>(&mut self, memory: &M) -> u8 {
        let value_from_vram = memory.read_u8(self.address);
        let value_for_cpu = if self.address < PALETTE_START {
            self.read_buffer
        } else {
            value_from_vram
        };
        self.address = self.address.wrapping_add(self.address_increment as u16);
        self.read_buffer = value_from_vram;
        value_from_vram
    }
    pub fn render<M: PpuMemory, O: RenderOutput>(
        &mut self,
        memory: &M,
        oam: &Oam,
        mut pixels: &mut O,
    ) {
        let name_table_and_attribute_table = memory.name_table(NameTableChoice::TopLeft);
        let name_table = &name_table_and_attribute_table[0x0..=0x3BF];
        let attribute_table = &name_table_and_attribute_table[0x3C0..=0x3FF];
        let palette_ram = memory.palette_ram();
        let universal_background_colour = palette_ram[0];
        let background_pattern_table = memory.pattern_table(self.background_pattern_table);
        for (index, &name_table_entry) in name_table.iter().enumerate() {
            let x = index % 32;
            let y = index / 32;
            let attribute_base = (y / 4) * 8 + (x / 4);
            let attribute = attribute_table[attribute_base as usize];
            let top = y & 2 == 0;
            let left = x & 2 == 0;
            let shift = match (top, left) {
                (true, true) => 0,
                (true, false) => 2,
                (false, true) => 4,
                (false, false) => 6,
            };
            let palette_base = (attribute.wrapping_shr(shift) & 0x3) * 4;
            let palette = &palette_ram[palette_base as usize..palette_base as usize + 4];
            let pattern_address = name_table_entry as u16 * 16;
            let pattern_lo = &background_pattern_table
                [pattern_address as usize + 0x0..=pattern_address as usize + 0x7];
            let pattern_hi = &background_pattern_table
                [pattern_address as usize + 0x8..=pattern_address as usize + 0xF];
            for (row_index, (&pixel_row_lo, &pixel_row_hi)) in
                pattern_lo.iter().zip(pattern_hi.iter()).enumerate()
            {
                for i in 0..8 {
                    let palette_index_lo = pixel_row_lo & 128u8.wrapping_shr(i) != 0;
                    let palette_index_hi = pixel_row_hi & 128u8.wrapping_shr(i) != 0;
                    let palette_index =
                        palette_index_lo as u8 | (palette_index_hi as u8).wrapping_shl(1);
                    match palette_index {
                        0 => {
                            pixels.set_pixel_colour_universal_background(
                                x as u16 * 8 + i as u16,
                                y as u16 * 8 + row_index as u16,
                                universal_background_colour,
                            );
                        }
                        non_zero => {
                            let colour_code = palette[non_zero as usize];
                            pixels.set_pixel_colour_background(
                                x as u16 * 8 + i as u16,
                                y as u16 * 8 + row_index as u16,
                                colour_code,
                            );
                        }
                    }
                }
            }
        }
        let sprite_pattern_table = memory.pattern_table(self.sprite_pattern_table);
        for i in 0..OAM_NUM_SPRITES {
            let oam_entry_index = i * OAM_SPRITE_BYTES;
            let oam_entry = &oam.ram[oam_entry_index..oam_entry_index + OAM_SPRITE_BYTES];
            let position_y = oam_entry[0].wrapping_add(1);
            if position_y == 0 || position_y & 0xF0 == 0xF0 {
                continue;
            }
            let tile_index = oam_entry[1];
            let attributes = oam_entry[2];
            let position_x = oam_entry[3];
            let pattern_address = tile_index as u16 * 16;
            let pattern_lo = &sprite_pattern_table
                [pattern_address as usize + 0x0..=pattern_address as usize + 0x7];
            let pattern_hi = &sprite_pattern_table
                [pattern_address as usize + 0x8..=pattern_address as usize + 0xF];
            let palette_base = ((attributes & 0x3) + 4) * 4;
            let palette = &palette_ram[palette_base as usize..palette_base as usize + 4];
            for (row_index, (&pixel_row_lo, &pixel_row_hi)) in
                pattern_lo.iter().zip(pattern_hi.iter()).enumerate()
            {
                for i in 0..8 {
                    let palette_index_lo = pixel_row_lo & 128u8.wrapping_shr(i) != 0;
                    let palette_index_hi = pixel_row_hi & 128u8.wrapping_shr(i) != 0;
                    let palette_index =
                        palette_index_lo as u8 | (palette_index_hi as u8).wrapping_shl(1);
                    let colour_code = match palette_index {
                        0 => continue,
                        _ => palette[palette_index as usize],
                    };
                    let offset_x =
                        if attributes & oam_attribute::flag::FLIP_SPRITE_HORIZONTALLY == 0 {
                            i
                        } else {
                            7 - i
                        };
                    let offset_y = if attributes & oam_attribute::flag::FLIP_SPRITE_VERTICALLY == 0
                    {
                        row_index
                    } else {
                        7 - row_index
                    };
                    let x = position_x as u16 + offset_x as u16;
                    let y = position_y as u16 + offset_y as u16;
                    if x < 256 && y < 240 {
                        if attributes & oam_attribute::flag::PRIORITY == 0 {
                            pixels.set_pixel_colour_sprite_front(x, y, colour_code);
                        } else {
                            pixels.set_pixel_colour_sprite_back(x, y, colour_code);
                        }
                    }
                }
            }
        }
    }
}

pub mod control {
    pub mod bit {
        pub const VBLANK_NMI: u8 = 7;
        pub const ADDRESS_INCREMENT: u8 = 2;
        pub const SPRITE_PATTERN_TABLE: u8 = 3;
        pub const BACKGROUND_PATTERN_TABLE: u8 = 4;
    }
    pub mod flag {
        use super::bit;
        pub const VBLANK_NMI: u8 = 1 << bit::VBLANK_NMI;
        pub const ADDRESS_INCREMENT: u8 = 1 << bit::ADDRESS_INCREMENT;
        pub const SPRITE_PATTERN_TABLE: u8 = 1 << bit::SPRITE_PATTERN_TABLE;
        pub const BACKGROUND_PATTERN_TABLE: u8 = 1 << bit::BACKGROUND_PATTERN_TABLE;
    }
}

pub mod status {
    pub mod bit {
        pub const VBLANK: u8 = 7;
    }
    pub mod flag {
        use super::bit;
        pub const VBLANK: u8 = 1 << bit::VBLANK;
    }
}

pub mod oam_attribute {
    pub mod bit {
        pub const FLIP_SPRITE_VERTICALLY: u8 = 7;
        pub const FLIP_SPRITE_HORIZONTALLY: u8 = 6;
        pub const PRIORITY: u8 = 5;
    }
    pub mod flag {
        use super::bit;
        pub const FLIP_SPRITE_VERTICALLY: u8 = 1 << bit::FLIP_SPRITE_VERTICALLY;
        pub const FLIP_SPRITE_HORIZONTALLY: u8 = 1 << bit::FLIP_SPRITE_HORIZONTALLY;
        pub const PRIORITY: u8 = 1 << bit::PRIORITY;
    }
}
