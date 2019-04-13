extern crate glutin_frontend;
use glutin_frontend::*;

fn main() {
    let mut frontend = Frontend::new();
    let mut running = true;
    let mut offset = 0u16;
    loop {
        frontend.poll_glutin_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => {
                    running = false;
                }
                _ => (),
            },
            _ => (),
        });
        if !running {
            break;
        }
        frontend.with_pixels(|pixels| {
            for p in pixels.iter_mut() {
                *p = [0., 0., 0., 1.];
            }
            for i in 0..HEIGHT_PX {
                let x = (i + offset) % WIDTH_PX;
                pixels[(i * WIDTH_PX + x) as usize] = [1., 0., 1., 1.];
            }
            offset += 1;
        });
        frontend.render();
    }
}
