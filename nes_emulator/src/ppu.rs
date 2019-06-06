use crate::mapper::{NameTableChoice, PatternTableChoice, PpuMapper};
use mos6502::address;
use mos6502::machine::{Address, Memory};
use nes_specs;
use std::fmt;

pub const NAME_TABLE_BYTES: usize = 0x400;
const OAM_SPRITE_BYTES: usize = 4;
const OAM_NUM_SPRITES: usize = 64;
const OAM_BYTES: usize = OAM_SPRITE_BYTES * OAM_NUM_SPRITES;
const TILE_SIZE_PX: u16 = 8;
const SCREEN_WIDTH_TILES: u16 = nes_specs::SCREEN_WIDTH_PX / TILE_SIZE_PX;
const SCREEN_HEIGHT_TILES: u16 = nes_specs::SCREEN_HEIGHT_PX / TILE_SIZE_PX;
const ATTRIBUTE_SIZE_TILES: u16 = 4;
const SCREEN_WIDTH_ATTRIBUTES: u16 = SCREEN_WIDTH_TILES / ATTRIBUTE_SIZE_TILES;
const PALETTE_NUM_COLOURS: u8 = 4;
const PATTERN_BYTES: u8 = 16;
const ATTRIBUTE_TABLE_START_INDEX: usize = 960;

pub mod name_table_mirroring {
    pub mod physical_base_address {
        use crate::mapper::{NameTableChoice, PpuAddress};
        use crate::ppu::NAME_TABLE_BYTES;
        pub const fn horizontal(name_table: NameTableChoice) -> PpuAddress {
            (name_table as PpuAddress / 2) * (NAME_TABLE_BYTES as PpuAddress)
        }
        pub const fn vertical(name_table: NameTableChoice) -> PpuAddress {
            (name_table as PpuAddress % 2) * (NAME_TABLE_BYTES as PpuAddress)
        }
        pub const fn single_screen_lower() -> PpuAddress {
            0
        }
        pub const fn single_screen_upper() -> PpuAddress {
            NAME_TABLE_BYTES as PpuAddress
        }
    }
    pub mod physical_offset {
        use crate::mapper::PpuAddress;
        pub const fn single_screen_lower(virtual_offset: PpuAddress) -> PpuAddress {
            virtual_offset & 0x03FF
        }
        pub const fn single_screen_upper(virtual_offset: PpuAddress) -> PpuAddress {
            single_screen_lower(virtual_offset) | 0x0400
        }
        pub const fn horizontal(virtual_offset: PpuAddress) -> PpuAddress {
            single_screen_lower(virtual_offset) | (virtual_offset & 0x0800).wrapping_shr(1)
        }
        pub const fn vertical(virtual_offset: PpuAddress) -> PpuAddress {
            virtual_offset & !0x0800
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
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

struct NameTableEntry {
    top_left_pixel_x: u16,
    top_left_pixel_y: u16,
    pattern_data_lo: [u8; PATTERN_BYTES as usize / 2],
    pattern_data_hi: [u8; PATTERN_BYTES as usize / 2],
    palette: [u8; PALETTE_NUM_COLOURS as usize],
}

impl NameTableEntry {
    fn lookup<M: PpuMapper>(
        tile_x: u16,
        tile_y: u16,
        background_pattern_table: &[u8],
        memory: &M,
    ) -> Self {
        let name_table_choice = if tile_x < SCREEN_WIDTH_TILES {
            if tile_y < SCREEN_HEIGHT_TILES {
                NameTableChoice::TopLeft
            } else {
                NameTableChoice::BottomLeft
            }
        } else {
            if tile_y < SCREEN_HEIGHT_TILES {
                NameTableChoice::TopRight
            } else {
                NameTableChoice::BottomRight
            }
        };
        let name_table = memory.ppu_name_table(name_table_choice);
        let name_table_relative_tile_x = tile_x % SCREEN_WIDTH_TILES;
        let name_table_relative_tile_y = tile_y % SCREEN_HEIGHT_TILES;
        let name_table_pattern_address_index =
            name_table_relative_tile_y * SCREEN_WIDTH_TILES + name_table_relative_tile_x;
        let pattern_index = name_table[name_table_pattern_address_index as usize];
        let pattern_offset = pattern_index as u16 * PATTERN_BYTES as u16;
        let mut pattern_data_lo = [0; PATTERN_BYTES as usize / 2];
        let mut pattern_data_hi = [0; PATTERN_BYTES as usize / 2];
        pattern_data_lo.copy_from_slice(
            &background_pattern_table
                [pattern_offset as usize..pattern_offset as usize + (PATTERN_BYTES as usize / 2)],
        );
        pattern_data_hi.copy_from_slice(
            &background_pattern_table[(pattern_offset as usize + (PATTERN_BYTES as usize / 2))
                ..(pattern_offset as usize + PATTERN_BYTES as usize)],
        );
        let name_table_relative_attribute_x = name_table_relative_tile_x / ATTRIBUTE_SIZE_TILES;
        let name_table_relative_attribute_y = name_table_relative_tile_y / ATTRIBUTE_SIZE_TILES;
        let name_table_attribute_index = name_table_relative_attribute_y * SCREEN_WIDTH_ATTRIBUTES
            + name_table_relative_attribute_x;
        let attribute_block =
            name_table[ATTRIBUTE_TABLE_START_INDEX + name_table_attribute_index as usize];
        let tile_2x2_block_coord_x = name_table_relative_tile_x / 2;
        let tile_2x2_block_coord_y = name_table_relative_tile_y / 2;
        let attribute_shift_to_select_2x2_tile_block = match (
            tile_2x2_block_coord_y % 2 == 0,
            tile_2x2_block_coord_x % 2 == 0,
        ) {
            (true, true) => 0,
            (true, false) => 2,
            (false, true) => 4,
            (false, false) => 6,
        };
        let palette_base = (attribute_block.wrapping_shr(attribute_shift_to_select_2x2_tile_block)
            & 0x3)
            * PALETTE_NUM_COLOURS;
        let mut palette = [0; PALETTE_NUM_COLOURS as usize];
        palette.copy_from_slice(
            &memory.ppu_palette_ram()[palette_base as usize..palette_base as usize + 4],
        );
        Self {
            top_left_pixel_x: tile_x * TILE_SIZE_PX,
            top_left_pixel_y: tile_y * TILE_SIZE_PX,
            pattern_data_lo,
            pattern_data_hi,
            palette,
        }
    }
    fn render<O: RenderOutput>(
        &self,
        pixel_min_x: u16,
        pixel_min_y: u16,
        pixel_max_x: u16,
        pixel_max_y: u16,
        background_colour_code: u8,
        pixels: &mut O,
    ) {
        for (pixel_row, (&pixel_row_lo, &pixel_row_hi)) in self
            .pattern_data_lo
            .iter()
            .zip(self.pattern_data_hi.iter())
            .enumerate()
        {
            let pixel_y = self.top_left_pixel_y + pixel_row as u16;
            if pixel_y < pixel_min_y || pixel_y > pixel_max_y {
                continue;
            }
            let screen_pixel_y = pixel_y - pixel_min_y;
            for pixel_col in 0..8u16 {
                let pixel_x = self.top_left_pixel_x + TILE_SIZE_PX - pixel_col - 1;
                if pixel_x < pixel_min_x || pixel_x > pixel_max_x {
                    continue;
                }
                let screen_pixel_x = pixel_x - pixel_min_x;
                let palette_index = (pixel_row_lo.wrapping_shr(pixel_col as u32) & 0x1)
                    | (pixel_row_hi.wrapping_shr(pixel_col as u32).wrapping_shl(1) & 0x2);

                match palette_index {
                    0 => pixels.set_pixel_colour_universal_background(
                        screen_pixel_x,
                        screen_pixel_y,
                        background_colour_code,
                    ),
                    non_zero => pixels.set_pixel_colour_background(
                        screen_pixel_x,
                        screen_pixel_y,
                        self.palette[non_zero as usize],
                    ),
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ppu {
    next_address_write_is_hi_byte: bool,
    address: u16,
    address_increment: u8,
    vblank_nmi: bool,
    sprite_pattern_table: PatternTableChoice,
    background_pattern_table: PatternTableChoice,
    read_buffer: u8,
    oam_address: u8,
    next_scroll_write_is_x: bool,
    scroll_x: u8,
    scroll_y: u8,
}

pub type PpuAddress = u16;
pub const PALETTE_START: PpuAddress = 0x3F00;

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
            oam_address: 0,
            next_scroll_write_is_x: true,
            scroll_x: 0,
            scroll_y: 0,
        }
    }
    pub fn vblank_nmi(&self) -> bool {
        self.vblank_nmi
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
        self.next_scroll_write_is_x = true;
        status::flag::VBLANK
    }
    pub fn write_oam_address(&mut self, data: u8) {
        self.oam_address = data;
    }
    pub fn write_oam_data(&mut self, data: u8, oam: &mut Oam) {
        oam.ram[self.oam_address as usize] = data;
        self.oam_address = self.oam_address.wrapping_add(1);
    }
    pub fn read_oam_data(&mut self, oam: &Oam) -> u8 {
        let data = oam.ram[self.oam_address as usize];
        self.oam_address = self.oam_address.wrapping_add(1);
        data
    }
    pub fn write_scroll(&mut self, data: u8) {
        if self.next_scroll_write_is_x {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.next_scroll_write_is_x = !self.next_scroll_write_is_x;
    }
    pub fn write_address(&mut self, data: u8) {
        let shift = self.next_address_write_is_hi_byte as u32 * 8;
        let mask = 0xFF00u16.wrapping_shr(shift);
        self.address = (self.address & mask) | (data as u16).wrapping_shl(shift);
        self.next_address_write_is_hi_byte = !self.next_address_write_is_hi_byte;
    }
    pub fn write_data<M: PpuMapper>(&mut self, memory: &mut M, data: u8) {
        memory.ppu_write_u8(self.address, data);
        self.address = self.address.wrapping_add(self.address_increment as u16);
    }
    pub fn read_data<M: PpuMapper>(&mut self, memory: &M) -> u8 {
        let value_from_vram = memory.ppu_read_u8(self.address);
        let value_for_cpu = if self.address < PALETTE_START {
            self.read_buffer
        } else {
            value_from_vram
        };
        self.address = self.address.wrapping_add(self.address_increment as u16);
        self.read_buffer = value_from_vram;
        value_for_cpu
    }
    fn render_background<M: PpuMapper, O: RenderOutput>(&self, memory: &M, pixels: &mut O) {
        let pixel_max_x = self.scroll_x as u16 + nes_specs::SCREEN_WIDTH_PX - 1;
        let pixel_max_y = self.scroll_y as u16 + nes_specs::SCREEN_HEIGHT_PX - 1;
        let tile_min_x = self.scroll_x as u16 / TILE_SIZE_PX;
        let tile_min_y = self.scroll_y as u16 / TILE_SIZE_PX;
        let tile_max_x = pixel_max_x / TILE_SIZE_PX;
        let tile_max_y = pixel_max_y / TILE_SIZE_PX;
        let background_pattern_table = memory.ppu_pattern_table(self.background_pattern_table);
        let universal_background_colour = memory.ppu_palette_ram()[0];
        for tile_y in tile_min_y..=tile_max_y {
            for tile_x in tile_min_x..=tile_max_x {
                NameTableEntry::lookup(tile_x, tile_y, background_pattern_table, memory).render(
                    self.scroll_x as u16,
                    self.scroll_y as u16,
                    pixel_max_x,
                    pixel_max_y,
                    universal_background_colour,
                    pixels,
                );
            }
        }
    }
    fn render_sprites<M: PpuMapper, O: RenderOutput>(&self, memory: &M, oam: &Oam, pixels: &mut O) {
        let palette_ram = memory.ppu_palette_ram();
        let sprite_pattern_table = memory.ppu_pattern_table(self.sprite_pattern_table);
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
                    let x = (position_x as u16)
                        .wrapping_add(offset_x as u16)
                        .wrapping_sub(self.scroll_x as u16);
                    let y = (position_y as u16)
                        .wrapping_add(offset_y as u16)
                        .wrapping_sub(self.scroll_y as u16);
                    if x < nes_specs::SCREEN_WIDTH_PX && y < nes_specs::SCREEN_HEIGHT_PX {
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
    pub fn render<M: PpuMapper, O: RenderOutput>(&self, memory: &M, oam: &Oam, pixels: &mut O) {
        self.render_background(memory, pixels);
        self.render_sprites(memory, oam, pixels);
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
