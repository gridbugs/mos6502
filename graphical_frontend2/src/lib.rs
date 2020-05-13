pub struct Frontend {
    window: winit::window::Window,
}

impl Frontend {
    pub fn new(scale: f64) -> Self {
        let window_size = winit::dpi::LogicalSize::new(
            nes_specs::SCREEN_WIDTH_PX as f64 * scale,
            nes_specs::SCREEN_HEIGHT_PX as f64 * scale,
        );
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("Nes Emulator")
            .with_inner_size(window_size)
            .with_min_inner_size(window_size)
            .with_max_inner_size(window_size)
            .build(&event_loop)
            .unwrap();
        let surface = pixels::wgpu::Surface::create(&window);
        let surface_texture = pixels::SurfaceTexture::new(
            (nes_specs::SCREEN_WIDTH_PX as f64 * scale) as u32,
            (nes_specs::SCREEN_HEIGHT_PX as f64 * scale) as u32,
            surface,
        );
        let pixels = pixels::Pixels::new(
            (nes_specs::SCREEN_WIDTH_PX as f64 * scale) as u32,
            (nes_specs::SCREEN_HEIGHT_PX as f64 * scale) as u32,
            surface_texture,
        )
        .unwrap();
        Self { window }
    }
}
