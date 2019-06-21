use crate::mapper::{NameTableChoice, PatternTableChoice, PpuMapper};
use mos6502::address;
use mos6502::machine::{Address, Memory};
use nes_name_table_debug::NameTableFrame;
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
const BLACK_COLOUR_CODE: u8 = 0xF;

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

#[derive(Clone, Copy, Debug)]
struct OamEntry {
    tile_index: u8,
    attributes: u8,
    position_x: u8,
    position_y: u8,
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
    fn get(&self, sprite_index: usize) -> Option<OamEntry> {
        let oam_entry_address = sprite_index * OAM_SPRITE_BYTES;
        let oam_entry = &self.ram[oam_entry_address..oam_entry_address + OAM_SPRITE_BYTES];
        let position_y = oam_entry[0].wrapping_add(1);
        if position_y == 0 || position_y & 0xF0 == 0xF0 {
            return None;
        }
        Some(OamEntry {
            position_y,
            tile_index: oam_entry[1],
            attributes: oam_entry[2],
            position_x: oam_entry[3],
        })
    }
}

impl OamEntry {
    fn flip_sprite_horizontally(&self) -> bool {
        self.attributes & oam_attribute::flag::FLIP_SPRITE_HORIZONTALLY != 0
    }
    fn flip_sprite_vertically(&self) -> bool {
        self.attributes & oam_attribute::flag::FLIP_SPRITE_VERTICALLY != 0
    }
    fn palette_base(&self) -> usize {
        (((self.attributes & 0x3) + 4) * 4) as usize
    }
    fn is_in_front_of_background(&self) -> bool {
        self.attributes & oam_attribute::flag::PRIORITY == 0
    }
    fn offset_within_pattern_table_8x8(&self) -> u16 {
        self.tile_index as u16 * 16
    }
    fn offset_within_pattern_table_8x16(&self) -> (u16, PatternTableChoice) {
        let tile_index = self.tile_index & !1;
        let tile_offset = tile_index as u16 * 16;
        let pattern_table_choice = if self.tile_index & 1 == 0 {
            PatternTableChoice::PatternTable0
        } else {
            PatternTableChoice::PatternTable1
        };
        (tile_offset, pattern_table_choice)
    }
    fn render_pixel_row<O: RenderOutput>(
        &self,
        pixel_row_lo: u8,
        pixel_row_hi: u8,
        pixel_row_index: usize,
        palette: &[u8],
        extra_offset_y: u8,
        show_left_8_pixels: bool,
        pixels: &mut O,
    ) {
        for pixel_col in 0..8 {
            let colour_code = match pattern_to_palette_index(pixel_row_lo, pixel_row_hi, pixel_col)
            {
                0 => continue,
                non_zero => palette[non_zero as usize],
            };
            let offset_x = if self.flip_sprite_horizontally() {
                pixel_col
            } else {
                7 - pixel_col
            };
            let offset_y = if self.flip_sprite_vertically() {
                7 - pixel_row_index
            } else {
                pixel_row_index
            };
            let x = (self.position_x as u16).wrapping_add(offset_x as u16);
            let y = (self.position_y as u16)
                .wrapping_add(offset_y as u16)
                .wrapping_add(extra_offset_y as u16);
            if x < nes_specs::SCREEN_WIDTH_PX && y < nes_specs::SCREEN_HEIGHT_PX {
                let colour_code = if !show_left_8_pixels && x < 8 {
                    BLACK_COLOUR_CODE
                } else {
                    colour_code
                };
                if self.is_in_front_of_background() {
                    pixels.set_pixel_colour_sprite_front(x, y, colour_code);
                } else {
                    pixels.set_pixel_colour_sprite_back(x, y, colour_code);
                }
            }
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

fn pattern_to_palette_index(pixel_row_lo: u8, pixel_row_hi: u8, pixel_index: u32) -> u8 {
    (pixel_row_lo.wrapping_shr(pixel_index) & 0x1)
        | (pixel_row_hi.wrapping_shr(pixel_index).wrapping_shl(1) & 0x2)
}

struct NameTableEntry {
    top_left_pixel_x: u16,
    pattern_data_lo: [u8; PATTERN_BYTES as usize / 2],
    pattern_data_hi: [u8; PATTERN_BYTES as usize / 2],
    palette: [u8; PALETTE_NUM_COLOURS as usize],
}

pub struct SpriteZeroHit {
    screen_pixel_x: u8,
}

impl SpriteZeroHit {
    pub fn screen_pixel_x(&self) -> u8 {
        self.screen_pixel_x
    }
}

impl NameTableEntry {
    fn lookup<M: PpuMapper>(
        tile_x: u16,
        tile_y: u16,
        background_pattern_table: &[u8],
        memory: &M,
    ) -> Self {
        let name_table_choice = if tile_x % (SCREEN_WIDTH_TILES * 2) < SCREEN_WIDTH_TILES {
            if tile_y % (SCREEN_HEIGHT_TILES * 2) < SCREEN_HEIGHT_TILES {
                NameTableChoice::TopLeft
            } else {
                NameTableChoice::BottomLeft
            }
        } else {
            if tile_y % (SCREEN_HEIGHT_TILES * 2) < SCREEN_HEIGHT_TILES {
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
            pattern_data_lo,
            pattern_data_hi,
            palette,
        }
    }
    fn render_pixel_row<O: RenderOutput>(
        &self,
        scanline: u8,
        pixel_row: u16,
        pixel_min_x: u16,
        pixel_max_x: u16,
        background_colour_code: u8,
        show_left_8_pixels: bool,
        sprite_zero_row: SpriteZeroRow,
        pixels: &mut O,
    ) -> Option<SpriteZeroHit> {
        let pixel_row_lo = self.pattern_data_lo[pixel_row as usize];
        let pixel_row_hi = self.pattern_data_hi[pixel_row as usize];
        let mut sprite_zero_hit = None;
        for pixel_col in 0..8u16 {
            let pixel_x = self.top_left_pixel_x + TILE_SIZE_PX - pixel_col - 1;
            if pixel_x < pixel_min_x || pixel_x > pixel_max_x {
                continue;
            }
            let screen_pixel_x = pixel_x - pixel_min_x;
            if !show_left_8_pixels && screen_pixel_x < 8 {
                pixels.set_pixel_colour_background(
                    screen_pixel_x,
                    scanline as u16,
                    BLACK_COLOUR_CODE,
                );
            } else {
                match pattern_to_palette_index(pixel_row_lo, pixel_row_hi, pixel_col as u32) {
                    0 => pixels.set_pixel_colour_universal_background(
                        screen_pixel_x,
                        scanline as u16,
                        background_colour_code,
                    ),
                    non_zero => {
                        if sprite_zero_hit.is_none() && sprite_zero_row.is_hit(screen_pixel_x as u8)
                        {
                            sprite_zero_hit = Some(SpriteZeroHit {
                                screen_pixel_x: screen_pixel_x as u8,
                            });
                        }
                        pixels.set_pixel_colour_background(
                            screen_pixel_x,
                            scanline as u16,
                            self.palette[non_zero as usize],
                        );
                    }
                }
            }
        }
        sprite_zero_hit
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum SpriteSize {
    Small,
    Large,
}

// The layout of this value is:
// yyy NN YYYYY XXXXX
// ||| || ||||| +++++-- coarse X scroll
// ||| || +++++-------- coarse Y scroll
// ||| ++-------------- nametable select
// +++----------------- fine Y scroll
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct ScrollStateAddress(u16);

impl ScrollStateAddress {
    fn set_name_table_from_low_2_bits(&mut self, data: u8) {
        self.0 = (self.0 & !(0x3 << 10)) | ((data & 0x3) as u16) << 10;
    }
    fn set_coarse_x_scroll_from_scroll(&mut self, data: u8) {
        self.0 = (self.0 & !0x1F) | ((data as u16) >> 3);
    }
    fn set_y_scroll(&mut self, data: u8) {
        self.0 = (self.0 & !(0x1F << 5)) | (((data as u16) >> 3) << 5);
        self.0 = (self.0 & !(0x7 << 12)) | ((data as u16 & 0x7) << 12);
    }
    fn set_address_hi(&mut self, data: u8) {
        self.0 = (self.0 & !(0x3F << 8)) | ((data as u16 & 0x3F) << 8);
        self.0 = self.0 & !(0x1 << 14);
    }
    fn set_address_lo(&mut self, data: u8) {
        self.0 = (self.0 & !0xFF) | data as u16;
    }
    fn copy_from_other_with_mask(&mut self, other: Self, mask: u16) {
        self.0 = (self.0 & !mask) | (other.0 & mask);
    }
    fn ppu_address(&self) -> PpuAddress {
        self.0
    }
    fn scroll_x_coarse(&self) -> u16 {
        let tile_coord = (self.0 & 0x1F) | ((self.0 & (0x1 << 10)) >> 5);
        tile_coord << 3
    }
    fn scroll_y(&self) -> u16 {
        let nametable_select_offset = ((self.0 & (0x1 << 11)) >> 11) * nes_specs::SCREEN_HEIGHT_PX;
        let coarse_y_scroll = ((self.0 & (0x1F << 5)) >> 5) << 3;
        let fine_y_scroll = (self.0 >> 12) & 0x7;
        nametable_select_offset + coarse_y_scroll + fine_y_scroll
    }
    fn increment_ppu_address(&mut self, by: u8) {
        self.0 = self.0.wrapping_add(by as u16);
    }
    fn increment_vertical_scroll(&mut self) {
        if self.0 & (0x7 << 12) != (0x7 << 12) {
            // increment fine y scroll
            self.0 += 0x1 << 12;
        } else {
            // set fine y scroll to 0
            self.0 &= !(0x7 << 12);
            let mut tile_scroll_y = (self.0 & (0x1F << 5)) >> 5;
            match tile_scroll_y {
                29 => {
                    // we're currently on the bottom pixel row of the bottom tile row, so wrap to 0
                    tile_scroll_y = 0;
                    // and flip the Y bit of the name table select
                    self.0 ^= 0x1 << 11;
                }
                31 => {
                    // We're currently inside the attribute table (this is allowed).
                    // In this case the hardware will just wrap the tile row to 0
                    tile_scroll_y = 0;
                }
                _ => {
                    // no wrapping necessary
                    tile_scroll_y += 1;
                }
            }
            self.0 = (self.0 & !(0x1F << 5)) | (tile_scroll_y << 5);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ScrollState {
    current_vram_address: ScrollStateAddress,
    temporary_vram_address: ScrollStateAddress,
    fine_x_scroll: u8,
    first_write_toggle: bool,
}

impl ScrollState {
    fn new() -> Self {
        Self {
            current_vram_address: ScrollStateAddress(0),
            temporary_vram_address: ScrollStateAddress(0),
            fine_x_scroll: 0,
            first_write_toggle: true,
        }
    }
    fn write_control(&mut self, data: u8) {
        self.temporary_vram_address
            .set_name_table_from_low_2_bits(data);
    }
    fn read_status(&mut self) {
        self.first_write_toggle = true;
    }
    fn write_scroll(&mut self, data: u8) {
        if self.first_write_toggle {
            self.fine_x_scroll = data & 7;
            self.temporary_vram_address
                .set_coarse_x_scroll_from_scroll(data);
        } else {
            self.temporary_vram_address.set_y_scroll(data);
        }
        self.first_write_toggle = !self.first_write_toggle;
    }
    fn write_address(&mut self, data: u8) {
        if self.first_write_toggle {
            self.temporary_vram_address.set_address_hi(data);
        } else {
            self.temporary_vram_address.set_address_lo(data);
            self.current_vram_address = self.temporary_vram_address;
        }
        self.first_write_toggle = !self.first_write_toggle;
    }
    fn copy_horizontal_scroll(&mut self) {
        let mask = 0x1F | (0x1 << 10);
        self.current_vram_address
            .copy_from_other_with_mask(self.temporary_vram_address, mask);
    }
    fn copy_vertical_scroll(&mut self) {
        let mask = (0x1F << 5) | (0xF << 11);
        self.current_vram_address
            .copy_from_other_with_mask(self.temporary_vram_address, mask);
    }
    fn scroll_x(&self) -> u16 {
        let coarse = self.current_vram_address.scroll_x_coarse();
        coarse | (self.fine_x_scroll as u16 & 0x7)
    }
    fn scroll_y(&self) -> u16 {
        self.current_vram_address.scroll_y()
    }
    fn ppu_address(&self) -> PpuAddress {
        self.current_vram_address.ppu_address()
    }
    fn increment_ppu_address(&mut self, by: u8) {
        self.current_vram_address.increment_ppu_address(by);
    }
    fn increment_vertical_scroll(&mut self) {
        self.current_vram_address.increment_vertical_scroll();
    }
}

#[derive(Clone, Copy)]
pub struct Scanline(u8);

impl Scanline {
    pub fn index(&self) -> u8 {
        self.0
    }
}

pub struct ScanlineIter {
    scanline: u8,
}

impl ScanlineIter {
    pub fn new() -> Self {
        Self { scanline: 0 }
    }
}

impl Iterator for ScanlineIter {
    type Item = Scanline;
    fn next(&mut self) -> Option<Self::Item> {
        let scanline = self.scanline;
        if scanline as u16 == nes_specs::SCREEN_HEIGHT_PX {
            None
        } else {
            self.scanline += 1;
            Some(Scanline(scanline))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ppu {
    address_increment: u8,
    vblank_nmi: bool,
    sprite_pattern_table: PatternTableChoice,
    background_pattern_table: PatternTableChoice,
    read_buffer: u8,
    oam_address: u8,
    vblank_flag: bool,
    sprite_size: SpriteSize,
    show_background: bool,
    show_sprites: bool,
    show_background_left_8_pixels: bool,
    show_sprites_left_8_pixels: bool,
    sprite_zero_hit: bool,
    scroll_state: ScrollState,
}

pub type PpuAddress = u16;
pub const PALETTE_START: PpuAddress = 0x3F00;

pub trait RenderOutput {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8);
}

#[derive(Debug)]
pub struct SpriteZero {
    opaque_pixel_map: u128,
    top_left_x: u8,
    top_left_y: u8,
}

#[derive(Clone, Copy)]
struct SpriteZeroRow {
    top_left_x: u8,
    opaque_pixel_map: u8,
}

impl SpriteZero {
    fn blank() -> Self {
        Self {
            opaque_pixel_map: 0,
            top_left_x: 0,
            top_left_y: 0,
        }
    }
    fn opaque_pixel_map_row(&self, scanline: u8) -> SpriteZeroRow {
        let opaque_pixel_map = if let Some(offset) = scanline.checked_sub(self.top_left_y) {
            if offset < 16 {
                (self.opaque_pixel_map >> (offset * 8)) as u8
            } else {
                0
            }
        } else {
            0
        };
        SpriteZeroRow {
            opaque_pixel_map,
            top_left_x: self.top_left_x,
        }
    }
}

impl SpriteZeroRow {
    fn is_hit(&self, pixel_x: u8) -> bool {
        pixel_x
            .checked_sub(self.top_left_x)
            .map(|offset| {
                if offset < 8 {
                    (0x80 >> offset) & self.opaque_pixel_map != 0
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            address_increment: 1,
            vblank_nmi: false,
            sprite_pattern_table: PatternTableChoice::PatternTable0,
            background_pattern_table: PatternTableChoice::PatternTable0,
            read_buffer: 0,
            oam_address: 0,
            vblank_flag: false,
            sprite_size: SpriteSize::Small,
            show_background: false,
            show_sprites: false,
            show_background_left_8_pixels: false,
            show_sprites_left_8_pixels: false,
            sprite_zero_hit: false,
            scroll_state: ScrollState::new(),
        }
    }
    pub fn is_vblank_nmi_enabled(&self) -> bool {
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
        self.sprite_size = if data & 1 << 5 == 0 {
            SpriteSize::Small
        } else {
            SpriteSize::Large
        };
        self.scroll_state.write_control(data);
    }
    pub fn write_mask(&mut self, data: u8) {
        self.show_background_left_8_pixels = data & (1 << 1) != 0;
        self.show_sprites_left_8_pixels = data & (1 << 2) != 0;
        self.show_background = data & (1 << 3) != 0;
        self.show_sprites = data & (1 << 4) != 0;
    }
    pub fn read_status(&mut self) -> u8 {
        let value = if self.vblank_flag {
            status::flag::VBLANK
        } else {
            0
        } | if self.sprite_zero_hit {
            status::flag::SPRITE_ZERO_HIT
        } else {
            0
        };
        self.vblank_flag = false;
        self.scroll_state.read_status();
        value
    }
    pub fn before_vblank(&mut self) {
        self.vblank_flag = true;
    }
    pub fn after_vblank(&mut self) {
        self.vblank_flag = false;
        self.sprite_zero_hit = false;
        if self.show_background {
            self.scroll_state.copy_vertical_scroll();
        }
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
        self.scroll_state.write_scroll(data);
    }
    pub fn write_address(&mut self, data: u8) {
        self.scroll_state.write_address(data);
    }
    pub fn write_data<M: PpuMapper>(&mut self, memory: &mut M, data: u8) {
        memory.ppu_write_u8(self.scroll_state.ppu_address(), data);
        self.scroll_state
            .increment_ppu_address(self.address_increment);
    }
    pub fn read_data<M: PpuMapper>(&mut self, memory: &M) -> u8 {
        let address = self.scroll_state.ppu_address();
        let value_from_vram = memory.ppu_read_u8(address);
        let value_for_cpu = if address < PALETTE_START {
            self.read_buffer
        } else {
            value_from_vram
        };
        self.scroll_state
            .increment_ppu_address(self.address_increment);
        self.read_buffer = value_from_vram;
        value_for_cpu
    }
    fn render_sprite_8x8<O: RenderOutput>(
        oam_entry: OamEntry,
        pattern_lo: &[u8],
        pattern_hi: &[u8],
        palette: &[u8],
        extra_offset_y: u8,
        show_sprites_left_8_pixels: bool,
        pixels: &mut O,
    ) {
        for (row_index, (&pixel_row_lo, &pixel_row_hi)) in
            pattern_lo.iter().zip(pattern_hi.iter()).enumerate()
        {
            oam_entry.render_pixel_row(
                pixel_row_lo,
                pixel_row_hi,
                row_index,
                palette,
                extra_offset_y,
                show_sprites_left_8_pixels,
                pixels,
            );
        }
    }

    fn render_sprites_8x8<M: PpuMapper, O: RenderOutput>(
        &self,
        memory: &M,
        oam: &Oam,
        pixels: &mut O,
    ) {
        let palette_ram = memory.ppu_palette_ram();
        let sprite_pattern_table = memory.ppu_pattern_table(self.sprite_pattern_table);
        for sprite_index in 0..OAM_NUM_SPRITES {
            let oam_entry = if let Some(oam_entry) = oam.get(sprite_index) {
                oam_entry
            } else {
                continue;
            };
            let pattern_offset = oam_entry.offset_within_pattern_table_8x8();
            let pattern_lo = &sprite_pattern_table
                [pattern_offset as usize + 0x0..=pattern_offset as usize + 0x7];
            let pattern_hi = &sprite_pattern_table
                [pattern_offset as usize + 0x8..=pattern_offset as usize + 0xF];
            let palette_base = oam_entry.palette_base();
            let palette = &palette_ram[palette_base..palette_base + 4];
            Self::render_sprite_8x8(
                oam_entry,
                pattern_lo,
                pattern_hi,
                palette,
                0,
                self.show_sprites_left_8_pixels,
                pixels,
            );
        }
    }
    fn render_sprites_8x16<M: PpuMapper, O: RenderOutput>(
        &self,
        memory: &M,
        oam: &Oam,
        pixels: &mut O,
    ) {
        let palette_ram = memory.ppu_palette_ram();
        for sprite_index in 0..OAM_NUM_SPRITES {
            let oam_entry = if let Some(oam_entry) = oam.get(sprite_index) {
                oam_entry
            } else {
                continue;
            };
            let (pattern_offset, pattern_table_choice) =
                oam_entry.offset_within_pattern_table_8x16();
            let sprite_pattern_table = memory.ppu_pattern_table(pattern_table_choice);
            let pattern_top_lo = &sprite_pattern_table
                [pattern_offset as usize + 0x00..=pattern_offset as usize + 0x07];
            let pattern_top_hi = &sprite_pattern_table
                [pattern_offset as usize + 0x08..=pattern_offset as usize + 0x0F];
            let pattern_bottom_lo = &sprite_pattern_table
                [pattern_offset as usize + 0x10..=pattern_offset as usize + 0x17];
            let pattern_bottom_hi = &sprite_pattern_table
                [pattern_offset as usize + 0x18..=pattern_offset as usize + 0x1F];
            let palette_base = oam_entry.palette_base();
            let palette = &palette_ram[palette_base..palette_base + 4];
            let (offset_top, offset_bottom) = if oam_entry.flip_sprite_vertically() {
                (8, 0)
            } else {
                (0, 8)
            };
            Self::render_sprite_8x8(
                oam_entry,
                pattern_top_lo,
                pattern_top_hi,
                palette,
                offset_top,
                self.show_sprites_left_8_pixels,
                pixels,
            );
            Self::render_sprite_8x8(
                oam_entry,
                pattern_bottom_lo,
                pattern_bottom_hi,
                palette,
                offset_bottom,
                self.show_sprites_left_8_pixels,
                pixels,
            );
        }
    }
    fn clear_background_scanline<O: RenderOutput>(&self, scanline: u8, pixels: &mut O) {
        for pixel_x in 0..nes_specs::SCREEN_WIDTH_PX {
            pixels.set_pixel_colour_background(pixel_x, scanline as u16, BLACK_COLOUR_CODE);
        }
    }
    pub fn render_background_scanline<M: PpuMapper, O: RenderOutput>(
        &mut self,
        scanline: Scanline,
        sprite_zero: &SpriteZero,
        memory: &M,
        pixels: &mut O,
    ) -> Option<SpriteZeroHit> {
        let mut ret = None;
        if self.show_background {
            let scroll_x = self.scroll_state.scroll_x();
            let scroll_y = self.scroll_state.scroll_y();
            let tile_y = scroll_y / TILE_SIZE_PX;
            let pixel_offset_within_tile_y = scroll_y % TILE_SIZE_PX;
            let pixel_max_x = scroll_x + nes_specs::SCREEN_WIDTH_PX - 1;
            let tile_min_x = scroll_x / TILE_SIZE_PX;
            let tile_max_x = pixel_max_x / TILE_SIZE_PX;
            let background_pattern_table = memory.ppu_pattern_table(self.background_pattern_table);
            let universal_background_colour = memory.ppu_palette_ram()[0];
            let sprite_zero_row = sprite_zero.opaque_pixel_map_row(scanline.0);
            for tile_x in tile_min_x..=tile_max_x {
                let name_table_entry =
                    NameTableEntry::lookup(tile_x, tile_y, background_pattern_table, memory);
                if let Some(sprite_zero_hit) = name_table_entry.render_pixel_row(
                    scanline.0,
                    pixel_offset_within_tile_y,
                    scroll_x,
                    pixel_max_x,
                    universal_background_colour,
                    self.show_background_left_8_pixels,
                    sprite_zero_row,
                    pixels,
                ) {
                    if !self.sprite_zero_hit {
                        ret = Some(sprite_zero_hit);
                    }
                    self.sprite_zero_hit = true;
                }
            }
            self.scroll_state.copy_horizontal_scroll();
            self.scroll_state.increment_vertical_scroll();
        } else {
            self.clear_background_scanline(scanline.0, pixels);
        }
        ret
    }
    pub fn debug_render_name_table_frame<M: PpuMapper>(
        &self,
        memory: &M,
        name_table_frame: &mut NameTableFrame,
    ) {
        let background_pattern_table = memory.ppu_pattern_table(self.background_pattern_table);
        for &(name_table_choice, offset_x, offset_y) in &[
            (NameTableChoice::TopLeft, 0, 0),
            (NameTableChoice::TopRight, nes_specs::SCREEN_WIDTH_PX, 0),
            (NameTableChoice::BottomLeft, 0, nes_specs::SCREEN_HEIGHT_PX),
            (
                NameTableChoice::BottomRight,
                nes_specs::SCREEN_WIDTH_PX,
                nes_specs::SCREEN_HEIGHT_PX,
            ),
        ] {
            let universal_background_colour = memory.ppu_palette_ram()[0];
            let name_table = memory.ppu_name_table(name_table_choice);
            for tile_y in 0..SCREEN_HEIGHT_TILES {
                for tile_x in 0..SCREEN_WIDTH_TILES {
                    let tile_index = tile_y * SCREEN_WIDTH_TILES + tile_x;
                    let pattern_index = name_table[tile_index as usize];
                    let pattern_offset = pattern_index as u16 * PATTERN_BYTES as u16;
                    let mut pattern_data_lo = [0; PATTERN_BYTES as usize / 2];
                    let mut pattern_data_hi = [0; PATTERN_BYTES as usize / 2];
                    pattern_data_lo.copy_from_slice(
                        &background_pattern_table[pattern_offset as usize
                            ..pattern_offset as usize + (PATTERN_BYTES as usize / 2)],
                    );
                    pattern_data_hi.copy_from_slice(
                        &background_pattern_table[(pattern_offset as usize
                            + (PATTERN_BYTES as usize / 2))
                            ..(pattern_offset as usize + PATTERN_BYTES as usize)],
                    );
                    let attribute_x = tile_x / ATTRIBUTE_SIZE_TILES;
                    let attribute_y = tile_y / ATTRIBUTE_SIZE_TILES;
                    let attribute_index = attribute_y * SCREEN_WIDTH_ATTRIBUTES + attribute_x;
                    let attribute_block =
                        name_table[ATTRIBUTE_TABLE_START_INDEX + attribute_index as usize];
                    let tile_2x2_block_coord_x = tile_x / 2;
                    let tile_2x2_block_coord_y = tile_y / 2;
                    let attribute_shift_to_select_2x2_tile_block = match (
                        tile_2x2_block_coord_y % 2 == 0,
                        tile_2x2_block_coord_x % 2 == 0,
                    ) {
                        (true, true) => 0,
                        (true, false) => 2,
                        (false, true) => 4,
                        (false, false) => 6,
                    };
                    let palette_base = (attribute_block
                        .wrapping_shr(attribute_shift_to_select_2x2_tile_block)
                        & 0x3)
                        * PALETTE_NUM_COLOURS;
                    let mut palette = [0; PALETTE_NUM_COLOURS as usize];
                    palette.copy_from_slice(
                        &memory.ppu_palette_ram()[palette_base as usize..palette_base as usize + 4],
                    );
                    for j in 0..TILE_SIZE_PX {
                        let pixel_row_lo = pattern_data_lo[j as usize];
                        let pixel_row_hi = pattern_data_hi[j as usize];
                        for i in 0..TILE_SIZE_PX {
                            let pixel_colour_index = match pattern_to_palette_index(
                                pixel_row_lo,
                                pixel_row_hi,
                                (TILE_SIZE_PX - 1 - i) as u32,
                            ) {
                                0 => universal_background_colour,
                                non_zero => palette[non_zero as usize],
                            };
                            let pixel_x = offset_x + (tile_x * TILE_SIZE_PX) + i;
                            let pixel_y = offset_y + (tile_y * TILE_SIZE_PX) + j;
                            name_table_frame.set_pixel(pixel_x, pixel_y, pixel_colour_index);
                        }
                    }
                }
            }
        }
    }
    pub fn scroll_x(&self) -> u16 {
        self.scroll_state.scroll_x()
    }
    pub fn scroll_y(&self) -> u16 {
        self.scroll_state.scroll_y()
    }
    fn sprite_opaque_pixel_map_8x8(pattern_lo: &[u8], pattern_hi: &[u8]) -> u64 {
        let mut result = 0;
        for (row_index, (&pixel_row_lo, &pixel_row_hi)) in
            pattern_lo.iter().zip(pattern_hi.iter()).enumerate()
        {
            let pattern_opaque_pixel_map = (pixel_row_lo | pixel_row_hi) as u64;
            let offset = row_index as u32 * 8;
            result |= pattern_opaque_pixel_map << offset;
        }
        result
    }
    fn sprite_zero_8x8<M: PpuMapper>(&self, oam_entry: OamEntry, memory: &mut M) -> SpriteZero {
        let sprite_pattern_table = memory.ppu_pattern_table(self.sprite_pattern_table);
        let pattern_offset = oam_entry.offset_within_pattern_table_8x8();
        let pattern_lo =
            &sprite_pattern_table[pattern_offset as usize + 0x0..=pattern_offset as usize + 0x7];
        let pattern_hi =
            &sprite_pattern_table[pattern_offset as usize + 0x8..=pattern_offset as usize + 0xF];
        let opaque_pixel_map = Self::sprite_opaque_pixel_map_8x8(pattern_lo, pattern_hi) as u128;
        let top_left_x = oam_entry.position_x;
        let top_left_y = oam_entry.position_y;
        SpriteZero {
            opaque_pixel_map,
            top_left_x,
            top_left_y,
        }
    }
    fn sprite_zero_8x16<M: PpuMapper>(&self, oam_entry: OamEntry, memory: &mut M) -> SpriteZero {
        let (pattern_offset, pattern_table_choice) = oam_entry.offset_within_pattern_table_8x16();
        let sprite_pattern_table = memory.ppu_pattern_table(pattern_table_choice);
        let pattern_top_lo =
            &sprite_pattern_table[pattern_offset as usize + 0x00..=pattern_offset as usize + 0x07];
        let pattern_top_hi =
            &sprite_pattern_table[pattern_offset as usize + 0x08..=pattern_offset as usize + 0x0F];
        let pattern_bottom_lo =
            &sprite_pattern_table[pattern_offset as usize + 0x10..=pattern_offset as usize + 0x17];
        let pattern_bottom_hi =
            &sprite_pattern_table[pattern_offset as usize + 0x18..=pattern_offset as usize + 0x1F];
        let opaque_pixel_map_top =
            Self::sprite_opaque_pixel_map_8x8(pattern_top_lo, pattern_top_hi);
        let opaque_pixel_map_bottom =
            Self::sprite_opaque_pixel_map_8x8(pattern_bottom_lo, pattern_bottom_hi);
        let opaque_pixel_map =
            (opaque_pixel_map_top as u128) | ((opaque_pixel_map_bottom as u128) << 64);
        let top_left_x = oam_entry.position_x;
        let top_left_y = oam_entry.position_y;
        SpriteZero {
            opaque_pixel_map,
            top_left_x,
            top_left_y,
        }
    }
    pub fn sprite_zero<M: PpuMapper>(&self, oam: &Oam, memory: &mut M) -> SpriteZero {
        if !(self.show_background && self.show_sprites) {
            SpriteZero::blank()
        } else if let Some(oam_entry) = oam.get(0) {
            if oam_entry.position_x >= 8
                || (self.show_background_left_8_pixels && self.show_sprites_left_8_pixels)
            {
                match self.sprite_size {
                    SpriteSize::Small => self.sprite_zero_8x8(oam_entry, memory),
                    SpriteSize::Large => self.sprite_zero_8x16(oam_entry, memory),
                }
            } else {
                SpriteZero::blank()
            }
        } else {
            SpriteZero::blank()
        }
    }
    pub fn render_sprites<M: PpuMapper, O: RenderOutput>(
        &self,
        memory: &M,
        oam: &Oam,
        pixels: &mut O,
    ) {
        if !self.show_sprites {
            return;
        }
        match self.sprite_size {
            SpriteSize::Small => self.render_sprites_8x8(memory, oam, pixels),
            SpriteSize::Large => self.render_sprites_8x16(memory, oam, pixels),
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
        pub const SPRITE_ZERO_HIT: u8 = 6;
    }
    pub mod flag {
        use super::bit;
        pub const VBLANK: u8 = 1 << bit::VBLANK;
        pub const SPRITE_ZERO_HIT: u8 = 1 << bit::SPRITE_ZERO_HIT;
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
