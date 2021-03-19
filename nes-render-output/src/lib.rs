pub trait RenderOutput {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8);
}
