extern crate glutin_frontend;
extern crate nes_specs;
use glutin_frontend::*;

const BLACK: u8 = 0x0F;
const PINK: u8 = 0x15;

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
                p.set_colour(BLACK);
            }
            for i in 0..nes_specs::SCREEN_HEIGHT_PX {
                let x = (i + offset) % nes_specs::SCREEN_WIDTH_PX;
                pixels.set_pixel_colour_background(x, i, PINK);
            }
            offset += 1;
        });
        frontend.render();
    }
}
