#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
pub extern crate glutin;

mod dimensions {
    pub const NES_SCREEN_WIDTH_PX: u16 = 256;
    pub const NES_SCREEN_HEIGHT_PX: u16 = 240;
    pub const SCALE: u16 = 2;
    pub const PIXEL_BUFFER_SIZE: usize = (NES_SCREEN_WIDTH_PX * NES_SCREEN_HEIGHT_PX) as usize;
}

mod formats {
    pub type ColourFormat = gfx::format::Rgba8;
    pub type DepthFormat = gfx::format::DepthStencil;
}

mod renderer {
    use super::dimensions::*;
    use super::formats::*;
    use gfx;

    const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
    const QUAD_COORDS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

    gfx_vertex_struct!(QuadCorner {
        corner_zero_to_one: [f32; 2] = "a_CornerZeroToOne",
    });

    gfx_pipeline!(pipe {
        quad_corners: gfx::VertexBuffer<QuadCorner> = (),
        pixel_colours: gfx::ShaderResource<[f32; 4]> = "t_PixelColours",
        out_colour: gfx::BlendTarget<ColourFormat> =
            ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    });

    pub struct Renderer<R, C, F, D>
    where
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
        D: gfx::Device<Resources = R, CommandBuffer = C>,
    {
        encoder: gfx::Encoder<R, C>,
        factory: F,
        device: D,
        pixel_colour_upload_buffer: gfx::handle::Buffer<R, [f32; 4]>,
        pixel_colour_buffer: gfx::handle::Buffer<R, [f32; 4]>,
        bundle: gfx::Bundle<R, pipe::Data<R>>,
    }

    impl<R, C, F, D> Renderer<R, C, F, D>
    where
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R> + gfx::traits::FactoryExt<R>,
        D: gfx::Device<Resources = R, CommandBuffer = C>,
    {
        pub fn new(
            encoder: gfx::Encoder<R, C>,
            mut factory: F,
            device: D,
            rtv: gfx::handle::RenderTargetView<R, ColourFormat>,
            dsv: gfx::handle::DepthStencilView<R, DepthFormat>,
        ) -> Self {
            let pso = factory
                .create_pipeline_simple(
                    include_bytes!("shaders/shader.150.vert"),
                    include_bytes!("shaders/shader.150.frag"),
                    pipe::new(),
                )
                .expect("Failed to create pipeline");
            let quad_corners_data = QUAD_COORDS
                .iter()
                .map(|v| QuadCorner {
                    corner_zero_to_one: *v,
                })
                .collect::<Vec<_>>();
            let (quad_corners_buf, slice) =
                factory.create_vertex_buffer_with_slice(&quad_corners_data, &QUAD_INDICES[..]);
            let pixel_colour_buffer = factory
                .create_buffer::<[f32; 4]>(
                    PIXEL_BUFFER_SIZE,
                    gfx::buffer::Role::Vertex,
                    gfx::memory::Usage::Data,
                    gfx::memory::Bind::TRANSFER_DST,
                )
                .expect("Failed to create buffer");
            let pixel_colour_srv = factory
                .view_buffer_as_shader_resource(&pixel_colour_buffer)
                .expect("Failed to view buffer as srv");
            let pixel_colour_upload_buffer = factory
                .create_upload_buffer::<[f32; 4]>(PIXEL_BUFFER_SIZE)
                .expect("Failed to create buffer");
            let data = pipe::Data {
                quad_corners: quad_corners_buf,
                pixel_colours: pixel_colour_srv,
                out_colour: rtv,
                out_depth: dsv,
            };
            let bundle = gfx::pso::bundle::Bundle::new(slice, pso, data);
            Self {
                encoder,
                factory,
                device,
                bundle,
                pixel_colour_upload_buffer,
                pixel_colour_buffer,
            }
        }
        pub fn with_pixels<G: FnMut(&mut [[f32; 4]])>(&mut self, mut g: G) {
            let mut writer = self
                .factory
                .write_mapping(&self.pixel_colour_upload_buffer)
                .expect("Failed to map pixel colour buffer");
            g(&mut writer);
        }
        pub fn render(&mut self) {
            self.encoder
                .copy_buffer(
                    &self.pixel_colour_upload_buffer,
                    &self.pixel_colour_buffer,
                    0,
                    0,
                    PIXEL_BUFFER_SIZE,
                )
                .expect("Failed to copy pixel colour buffer");
            self.encoder
                .clear(&self.bundle.data.out_colour, [0.0, 0.0, 0.0, 1.0]);
            self.encoder.clear_depth(&self.bundle.data.out_depth, 1.0);
            self.bundle.encode(&mut self.encoder);
            self.encoder.flush(&mut self.device);
            self.device.cleanup();
        }
    }

}

use dimensions::*;
use formats::*;
use renderer::Renderer;
use std::slice;
mod colour;

pub use dimensions::NES_SCREEN_HEIGHT_PX as HEIGHT_PX;
pub use dimensions::NES_SCREEN_WIDTH_PX as WIDTH_PX;
const NUM_PIXELS: usize = (WIDTH_PX * HEIGHT_PX) as usize;

type GlutinRenderer = Renderer<
    gfx_device_gl::Resources,
    gfx_device_gl::CommandBuffer,
    gfx_device_gl::Factory,
    gfx_device_gl::Device,
>;

pub struct Frontend {
    renderer: GlutinRenderer,
    windowed_context: glutin::WindowedContext<glutin::PossiblyCurrent>,
    events_loop: glutin::EventsLoop,
    colour_table: colour::ColourTable,
    depths: [u8; NUM_PIXELS],
}

mod depth {
    pub const EMPTY: u8 = 0;
    pub const UNIVERSAL_BACKGROUND: u8 = 1;
    pub const SPRITE_BACK: u8 = 2;
    pub const BACKGROUND: u8 = 3;
    pub const SPRITE_FRONT: u8 = 4;
}

pub struct Pixels<'a> {
    colour_table: &'a colour::ColourTable,
    raw: &'a mut [[f32; 4]],
    depths: &'a mut [u8; NUM_PIXELS],
}

impl<'a> Pixels<'a> {
    fn set_pixel_colour(&mut self, x: u16, y: u16, colour_index: u8, depth: u8) {
        let offset = (y * NES_SCREEN_WIDTH_PX + x) as usize;
        let current_depth = &mut self.depths[offset];
        if depth > *current_depth {
            *current_depth = depth;
            self.raw[offset] = self.colour_table.lookup(colour_index);
        }
    }
    pub fn set_pixel_colour_sprite_back(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::SPRITE_BACK);
    }
    pub fn set_pixel_colour_sprite_front(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::SPRITE_FRONT);
    }
    pub fn set_pixel_colour_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::BACKGROUND);
    }
    pub fn set_pixel_colour_universal_background(&mut self, x: u16, y: u16, colour_index: u8) {
        self.set_pixel_colour(x, y, colour_index, depth::UNIVERSAL_BACKGROUND);
    }
    pub fn iter_mut(&mut self) -> PixelsIterMut {
        PixelsIterMut {
            colour_table: self.colour_table,
            iter: self.raw.iter_mut(),
        }
    }
}

pub struct Pixel<'a> {
    colour_table: &'a colour::ColourTable,
    raw: &'a mut [f32; 4],
}

impl<'a> Pixel<'a> {
    pub fn set_colour(&mut self, colour_index: u8) {
        *self.raw = self.colour_table.lookup(colour_index);
    }
}

pub struct PixelsIterMut<'a> {
    colour_table: &'a colour::ColourTable,
    iter: slice::IterMut<'a, [f32; 4]>,
}

impl<'a> Iterator for PixelsIterMut<'a> {
    type Item = Pixel<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let colour_table = &self.colour_table;
        self.iter.next().map(|raw| Pixel { colour_table, raw })
    }
}

impl Frontend {
    pub fn new() -> Self {
        let events_loop = glutin::EventsLoop::new();
        let window_size = glutin::dpi::LogicalSize::new(
            (NES_SCREEN_WIDTH_PX * SCALE) as f64,
            (NES_SCREEN_HEIGHT_PX * SCALE) as f64,
        );
        let window_builder = glutin::WindowBuilder::new()
            .with_dimensions(window_size)
            .with_max_dimensions(window_size)
            .with_min_dimensions(window_size);
        let context_builder = glutin::ContextBuilder::new();
        let windowed_context = context_builder
            .build_windowed(window_builder, &events_loop)
            .expect("Failed to create window");
        let hidpi = windowed_context.window().get_hidpi_factor();
        let window_size = glutin::dpi::PhysicalSize::new(
            (NES_SCREEN_WIDTH_PX * SCALE) as f64,
            (NES_SCREEN_HEIGHT_PX * SCALE) as f64,
        )
        .to_logical(hidpi);
        windowed_context.window().set_inner_size(window_size);
        let (windowed_context, device, mut factory, rtv, dsv) =
            gfx_window_glutin::init_existing::<ColourFormat, DepthFormat>(windowed_context);
        let encoder = factory.create_command_buffer().into();
        let renderer = Renderer::new(encoder, factory, device, rtv, dsv);
        let colour_table = colour::ColourTable::new();
        Self {
            events_loop,
            windowed_context,
            renderer,
            colour_table,
            depths: [depth::EMPTY; NUM_PIXELS],
        }
    }
    pub fn render(&mut self) {
        self.renderer.render();
        self.windowed_context.swap_buffers().unwrap();
        self.depths = [0; NUM_PIXELS];
    }
    pub fn poll_glutin_events<F: FnMut(glutin::Event)>(&mut self, f: F) {
        self.events_loop.poll_events(f)
    }
    pub fn with_pixels<F: FnMut(Pixels)>(&mut self, mut f: F) {
        let colour_table = &self.colour_table;
        let depths = &mut self.depths;
        self.renderer.with_pixels(|raw| {
            f(Pixels {
                colour_table,
                raw,
                depths,
            })
        })
    }
}
