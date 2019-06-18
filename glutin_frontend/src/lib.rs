#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
pub extern crate glutin;
extern crate nes_specs;

mod dimensions {
    use nes_specs;
    pub const PIXEL_BUFFER_SIZE: usize = nes_specs::SCREEN_TOTAL_PX as usize;
}

mod formats {
    pub type ColourFormat = gfx::format::Srgba8;
    pub type DepthFormat = gfx::format::DepthStencil;
}

mod quad {
    pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
    pub const QUAD_COORDS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
}

mod renderer {
    use super::dimensions::*;
    use super::formats::*;
    use super::quad::*;
    use gfx;

    gfx_vertex_struct!(QuadCorner {
        corner_zero_to_one: [f32; 2] = "a_CornerZeroToOne",
    });

    gfx_pipeline!(ppu_pixel_pipe {
        quad_corners: gfx::VertexBuffer<QuadCorner> = (),
        pixel_colours: gfx::ShaderResource<[f32; 4]> = "t_PixelColours",
        out_colour: gfx::BlendTarget<gfx::format::Rgba8> =
            ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    });

    gfx_pipeline!(post_processor_pipe {
        quad_corners: gfx::VertexBuffer<QuadCorner> = (),
        in_colour: gfx::TextureSampler<<ColourFormat as gfx::format::Formatted>::View> = "t_InColour",
        out_colour: gfx::BlendTarget<ColourFormat> =
            ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
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
        ppu_pixel_bundle: gfx::Bundle<R, ppu_pixel_pipe::Data<R>>,
        post_processor_bundle: gfx::Bundle<R, post_processor_pipe::Data<R>>,
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
        ) -> Self {
            let pixel_colour_buffer = factory
                .create_buffer::<[f32; 4]>(
                    PIXEL_BUFFER_SIZE,
                    gfx::buffer::Role::Vertex,
                    gfx::memory::Usage::Data,
                    gfx::memory::Bind::TRANSFER_DST,
                )
                .expect("Failed to create buffer");
            let pixel_colour_upload_buffer = factory
                .create_upload_buffer::<[f32; 4]>(PIXEL_BUFFER_SIZE)
                .expect("Failed to create buffer");
            let quad_corners_data = QUAD_COORDS
                .iter()
                .map(|v| QuadCorner {
                    corner_zero_to_one: *v,
                })
                .collect::<Vec<_>>();
            let (_, post_processor_in, ppu_pixel_out) = factory
                .create_render_target(nes_specs::SCREEN_WIDTH_PX, nes_specs::SCREEN_HEIGHT_PX)
                .expect("Failed to create render target");
            let ppu_pixel_bundle = {
                let pso = factory
                    .create_pipeline_simple(
                        include_bytes!("shaders/ppu_pixel.150.vert"),
                        include_bytes!("shaders/ppu_pixel.150.frag"),
                        ppu_pixel_pipe::new(),
                    )
                    .expect("Failed to create pipeline");
                let (quad_corners_buf, slice) =
                    factory.create_vertex_buffer_with_slice(&quad_corners_data, &QUAD_INDICES[..]);
                let pixel_colour_srv = factory
                    .view_buffer_as_shader_resource(&pixel_colour_buffer)
                    .expect("Failed to view buffer as srv");
                let data = ppu_pixel_pipe::Data {
                    quad_corners: quad_corners_buf,
                    pixel_colours: pixel_colour_srv,
                    out_colour: ppu_pixel_out,
                };
                gfx::pso::bundle::Bundle::new(slice, pso, data)
            };
            let post_processor_bundle = {
                let pso = factory
                    .create_pipeline_simple(
                        include_bytes!("shaders/post_processor.150.vert"),
                        include_bytes!("shaders/post_processor.150.frag"),
                        post_processor_pipe::new(),
                    )
                    .expect("Failed to create pipeline");
                let (quad_corners_buf, slice) =
                    factory.create_vertex_buffer_with_slice(&quad_corners_data, &QUAD_INDICES[..]);
                let sampler = factory.create_sampler(gfx::texture::SamplerInfo::new(
                    gfx::texture::FilterMethod::Scale,
                    gfx::texture::WrapMode::Border,
                ));
                let data = post_processor_pipe::Data {
                    quad_corners: quad_corners_buf,
                    in_colour: (post_processor_in, sampler),
                    out_colour: rtv,
                };
                gfx::pso::bundle::Bundle::new(slice, pso, data)
            };
            Self {
                encoder,
                factory,
                device,
                pixel_colour_upload_buffer,
                pixel_colour_buffer,
                ppu_pixel_bundle,
                post_processor_bundle,
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
            self.ppu_pixel_bundle.encode(&mut self.encoder);
            self.post_processor_bundle.encode(&mut self.encoder);
            self.encoder.flush(&mut self.device);
            self.device.cleanup();
        }
    }

}

use formats::*;
use renderer::Renderer;
use std::slice;
mod colour;

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
    depths: [u8; nes_specs::SCREEN_TOTAL_PX as usize],
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
    depths: &'a mut [u8; nes_specs::SCREEN_TOTAL_PX as usize],
}

impl<'a> Pixels<'a> {
    fn set_pixel_colour(&mut self, x: u16, y: u16, colour_index: u8, depth: u8) {
        let offset = (y * nes_specs::SCREEN_WIDTH_PX + x) as usize;
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
    pub fn new(scale: f64) -> Self {
        let events_loop = glutin::EventsLoop::new();
        let window_size = glutin::dpi::LogicalSize::new(
            nes_specs::SCREEN_WIDTH_PX as f64 * scale,
            nes_specs::SCREEN_HEIGHT_PX as f64 * scale,
        );
        let window_builder = glutin::WindowBuilder::new()
            .with_dimensions(window_size)
            .with_max_dimensions(window_size)
            .with_min_dimensions(window_size);
        let context_builder = glutin::ContextBuilder::new()
            .with_srgb(true)
            .with_gl(glutin::GlRequest::Latest);
        let windowed_context = context_builder
            .build_windowed(window_builder, &events_loop)
            .expect("Failed to create window");
        let hidpi = windowed_context.window().get_hidpi_factor();
        let window_size = glutin::dpi::PhysicalSize::new(
            nes_specs::SCREEN_WIDTH_PX as f64 * scale,
            nes_specs::SCREEN_HEIGHT_PX as f64 * scale,
        )
        .to_logical(hidpi);
        windowed_context.window().set_inner_size(window_size);
        let (windowed_context, device, mut factory, rtv, _dsv) =
            gfx_window_glutin::init_existing::<ColourFormat, DepthFormat>(windowed_context);
        let encoder = factory.create_command_buffer().into();
        let renderer = Renderer::new(encoder, factory, device, rtv);
        let colour_table = colour::ColourTable::new();
        Self {
            events_loop,
            windowed_context,
            renderer,
            colour_table,
            depths: [depth::EMPTY; nes_specs::SCREEN_TOTAL_PX as usize],
        }
    }
    pub fn render(&mut self) {
        self.renderer.render();
        self.windowed_context.swap_buffers().unwrap();
        self.depths = [0; nes_specs::SCREEN_TOTAL_PX as usize];
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
