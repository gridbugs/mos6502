pub struct Frontend {
    window: winit::window::Window,
    pixels: pixels::Pixels,
    event_loop: winit::event_loop::EventLoop<()>,
}

impl Frontend {
    pub fn new(scale: f64) -> Self {
        let logical_width = nes_specs::SCREEN_WIDTH_PX as f64 * scale;
        let logical_height = nes_specs::SCREEN_HEIGHT_PX as f64 * scale;
        let logical_size = winit::dpi::LogicalSize::new(logical_width, logical_height);
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("Nes Emulator")
            .with_inner_size(logical_size)
            .with_min_inner_size(logical_size)
            .with_max_inner_size(logical_size)
            .build(&event_loop)
            .unwrap();
        let hidpi_factor = window.scale_factor();
        let surface = pixels::wgpu::Surface::create(&window);
        let physical_size = logical_size.to_physical(hidpi_factor);
        let surface_texture =
            pixels::SurfaceTexture::new(physical_size.width, physical_size.height, surface);
        let mut pixels = pixels::Pixels::new(
            nes_specs::SCREEN_WIDTH_PX as u32,
            nes_specs::SCREEN_HEIGHT_PX as u32,
            surface_texture,
        )
        .unwrap();
        Self {
            window,
            pixels,
            event_loop,
        }
    }

    pub fn run(self) {
        let mut input = winit_input_helper::WinitInputHelper::new();
        let Self {
            window,
            mut pixels,
            event_loop,
        } = self;
        event_loop.run(move |event, _, control_flow| {
            if let winit::event::Event::RedrawRequested(_) = event {
                let frame = pixels.get_frame();
                for i in 0..(nes_specs::SCREEN_HEIGHT_PX as usize) {
                    let base = (i * nes_specs::SCREEN_WIDTH_PX as usize + i) * 4;
                    frame[base + 0] = 255;
                    frame[base + 1] = 0;
                    frame[base + 2] = 0;
                    frame[base + 3] = 255;
                }
                pixels.render().unwrap();
            }
            if input.update(event) {
                if input.key_pressed(winit::event::VirtualKeyCode::Escape) || input.quit() {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                    return;
                }
                window.request_redraw();
            }
        });
        panic!("should not get here");
    }
}
