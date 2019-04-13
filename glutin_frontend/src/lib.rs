#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

mod formats {
    pub type ColourFormat = gfx::format::Rgba8;
    pub type DepthFormat = gfx::format::DepthStencil;
}

mod renderer {
    use super::formats::*;
    use gfx;

    const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
    const QUAD_COORDS: [[f32; 2]; 4] = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];

    gfx_vertex_struct!(QuadCorner {
        corner_zero_to_one: [f32; 2] = "a_CornerZeroToOne",
    });

    gfx_pipeline!(pipe {
        quad_corners: gfx::VertexBuffer<QuadCorner> = (),
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
            let data = pipe::Data {
                quad_corners: quad_corners_buf,
                out_colour: rtv,
                out_depth: dsv,
            };
            let bundle = gfx::pso::bundle::Bundle::new(slice, pso, data);
            Self {
                encoder,
                factory,
                device,
                bundle,
            }
        }
        pub fn render(&mut self) {
            self.encoder
                .clear(&self.bundle.data.out_colour, [0.0, 0.0, 0.0, 1.0]);
            self.encoder.clear_depth(&self.bundle.data.out_depth, 1.0);
            self.bundle.encode(&mut self.encoder);
            self.encoder.flush(&mut self.device);
            self.device.cleanup();
        }
    }

}

use formats::*;
use renderer::Renderer;

type GlutinRenderer = Renderer<
    gfx_device_gl::Resources,
    gfx_device_gl::CommandBuffer,
    gfx_device_gl::Factory,
    gfx_device_gl::Device,
>;

pub struct Frontend {
    renderer: GlutinRenderer,
    window: glutin::WindowedContext,
    events_loop: glutin::EventsLoop,
}

const NES_SCREEN_WIDTH_PX: u32 = 256;
const NES_SCREEN_HEIGHT_PX: u32 = 240;
const SCALE: u32 = 1;

impl Frontend {
    pub fn new() -> Self {
        let events_loop = glutin::EventsLoop::new();
        let context_builder = glutin::ContextBuilder::new();
        let window_size = glutin::dpi::LogicalSize {
            width: (NES_SCREEN_WIDTH_PX * SCALE) as f64,
            height: (NES_SCREEN_HEIGHT_PX * SCALE) as f64,
        };
        let window_builder = glutin::WindowBuilder::new()
            .with_dimensions(window_size)
            .with_max_dimensions(window_size)
            .with_min_dimensions(window_size);
        let (window, device, mut factory, rtv, dsv) =
            gfx_window_glutin::init::<ColourFormat, DepthFormat>(
                window_builder,
                context_builder,
                &events_loop,
            )
            .expect("Failed to create window");
        let encoder = factory.create_command_buffer().into();
        let renderer = Renderer::new(encoder, factory, device, rtv, dsv);
        Self {
            events_loop,
            window,
            renderer,
        }
    }
    pub fn render(&mut self) {
        self.renderer.render();
        self.window.swap_buffers().unwrap();
    }
    pub fn poll_glutin_events<F: FnMut(glutin::Event)>(&mut self, f: F) {
        self.events_loop.poll_events(f)
    }
}
