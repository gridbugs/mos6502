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
        frontend.with_pixels(|mut pixels| {
            for mut p in pixels.iter_mut() {
                p.set_colour([0., 0., 0.]);
            }
            for i in 0..HEIGHT_PX {
                let x = (i + offset) % WIDTH_PX;
                pixels.set_pixel_colour(x, i, [1., 0., 1.]);
            }
            offset += 1;
        });
        frontend.render();
    }
}
