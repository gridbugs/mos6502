pub trait RenderOutput {
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8);
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8);
}

pub struct RenderOutputPair<'a, A, B> {
    a: &'a mut A,
    b: &'a mut B,
}

impl<'a, A, B> RenderOutputPair<'a, A, B> {
    pub fn new(a: &'a mut A, b: &'a mut B) -> Self {
        Self { a, b }
    }
}

impl<'a, A, B> RenderOutput for RenderOutputPair<'a, A, B>
where
    A: RenderOutput,
    B: RenderOutput,
{
    fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.a.set_pixel_colour_sprite_back(x, y, colour_index);
        self.b.set_pixel_colour_sprite_back(x, y, colour_index);
    }
    fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.a.set_pixel_colour_sprite_front(x, y, colour_index);
        self.b.set_pixel_colour_sprite_front(x, y, colour_index);
    }
    fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.a.set_pixel_colour_background(x, y, colour_index);
        self.b.set_pixel_colour_background(x, y, colour_index);
    }
    fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.a
            .set_pixel_colour_universal_background(x, y, colour_index);
        self.b
            .set_pixel_colour_universal_background(x, y, colour_index);
    }
}

pub struct NoRenderOutput;

impl RenderOutput for NoRenderOutput {
    fn set_pixel_colour_sprite_back(&mut self, _x: u16, _y: u16, _colour_index: u8) {}
    fn set_pixel_colour_sprite_front(&mut self, _x: u16, _y: u16, _colour_index: u8) {}
    fn set_pixel_colour_background(&mut self, _x: u16, _y: u16, _colour_index: u8) {}
    fn set_pixel_colour_universal_background(&mut self, _x: u16, _y: u16, _colour_index: u8) {}
}
