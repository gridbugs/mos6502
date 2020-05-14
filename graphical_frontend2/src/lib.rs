pub mod input {
    pub use winit::event::{ElementState, Event as WinitEvent, VirtualKeyCode, WindowEvent};
    pub type Event<'a> = WinitEvent<'a, ()>;
}

mod depth {
    pub const EMPTY: u8 = 0;
    pub const UNIVERSAL_BACKGROUND: u8 = 1;
    pub const SPRITE_BACK: u8 = 2;
    pub const BACKGROUND: u8 = 3;
    pub const SPRITE_FRONT: u8 = 4;
}

pub enum ControlFlow {
    Quit,
}

pub trait AppTrait {
    fn handle_input(&mut self, event: input::Event) -> Option<ControlFlow>;
    fn tick(&mut self, pixels: Pixels) -> Option<ControlFlow>;
}

pub struct Frontend {
    window: winit::window::Window,
    pixels: pixels::Pixels,
    event_loop: winit::event_loop::EventLoop<()>,
    colour_table: ColourTable,
    depths: [u8; nes_specs::SCREEN_TOTAL_PX as usize],
}

impl Frontend {
    pub fn new(scale: f64) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let width = nes_specs::SCREEN_WIDTH_PX as f64 * scale;
        let height = nes_specs::SCREEN_HEIGHT_PX as f64 * scale;
        let logical_size = winit::dpi::LogicalSize::new(width, height);
        let window = winit::window::WindowBuilder::new()
            .with_title("Nes Emulator")
            .with_inner_size(logical_size)
            .with_min_inner_size(logical_size)
            .with_max_inner_size(logical_size)
            .build(&event_loop)
            .unwrap();
        let hidpi_factor = window.scale_factor();
        let surface = pixels::wgpu::Surface::create(&window);
        let physical_size = logical_size.to_physical::<f64>(hidpi_factor);
        let physical_width = physical_size.width.round() as u32;
        let physical_height = physical_size.height.round() as u32;
        let surface_texture = pixels::SurfaceTexture::new(physical_width, physical_height, surface);
        let pixels = pixels::Pixels::new(
            nes_specs::SCREEN_WIDTH_PX as u32,
            nes_specs::SCREEN_HEIGHT_PX as u32,
            surface_texture,
        )
        .unwrap();
        let colour_table = ColourTable::new();
        let depths = [depth::EMPTY; nes_specs::SCREEN_TOTAL_PX as usize];
        Self {
            window,
            pixels,
            event_loop,
            colour_table,
            depths,
        }
    }

    pub fn run<A>(self, mut app: A)
    where
        A: AppTrait + 'static,
    {
        let Self {
            window,
            mut pixels,
            event_loop,
            colour_table,
            mut depths,
        } = self;
        let mut input = winit_input_helper::WinitInputHelper::new();
        event_loop.run(move |event, _, control_flow| {
            let app_control_flow = match event {
                winit::event::Event::RedrawRequested(_) => {
                    if pixels.render().is_err() {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                        return;
                    }
                    depths = [0; nes_specs::SCREEN_TOTAL_PX as usize];
                    None
                }
                winit::event::Event::MainEventsCleared => {
                    let app_control_flow = app.tick(Pixels {
                        colour_table: &colour_table,
                        frame: pixels.get_frame(),
                        depths: &mut depths,
                    });
                    if app_control_flow.is_none() {
                        window.request_redraw();
                    }
                    app_control_flow
                }
                other => app.handle_input(other),
            };
            if let Some(app_control_flow) = app_control_flow {
                match app_control_flow {
                    ControlFlow::Quit => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                        return;
                    }
                }
            }
        });
    }
}

struct ColourTable {
    colours: Vec<[u8; 3]>,
}

impl ColourTable {
    fn new() -> Self {
        let colours = nes_palette::Palette::basic()
            .to_bytes()
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        Self { colours }
    }
    fn lookup(&self, colour_index: u8) -> [u8; 3] {
        self.colours[(colour_index & nes_palette::COLOUR_MASK) as usize]
    }
}

pub struct Pixels<'a> {
    colour_table: &'a ColourTable,
    frame: &'a mut [u8],
    depths: &'a mut [u8; nes_specs::SCREEN_TOTAL_PX as usize],
}

impl<'a> Pixels<'a> {
    fn set_pixel_colour(&mut self, x: u16, y: u16, colour_index: u8, depth: u8) {
        let offset = (y * nes_specs::SCREEN_WIDTH_PX + x) as usize;
        let current_depth = &mut self.depths[offset];
        if depth > *current_depth {
            *current_depth = depth;
            let base_index = offset * 4;
            let [r, g, b] = self.colour_table.lookup(colour_index);
            self.frame[base_index + 0] = r;
            self.frame[base_index + 1] = g;
            self.frame[base_index + 2] = b;
            self.frame[base_index + 3] = 255;
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
}
