use gif_renderer::{Frame as GifFrame, Renderer as GifRenderer};
use ines::Ines;
use nes_emulator_core::dynamic_nes::DynamicNes;
use nes_render_output::NoRenderOutput;
use std::fs::File;

struct Args {
    rom_path: String,
    gif_path: String,
}

impl Args {
    fn parser() -> impl meap::Parser<Item = Self> {
        meap::let_map! {
            let {
                rom_path = opt_req::<String, _>("PATH", 'r').name("rom-path").desc("path to tetris rom file");
                gif_path = opt_req::<String, _>("PATH", 'g').name("gif-path").desc("path to gif to create");
            } in {
                Self {
                    rom_path,
                    gif_path,
                }
            }
        }
    }
}

fn ines_from_file(path: &str) -> Ines {
    use std::io::Read;
    let mut input = Vec::new();
    let mut rom_file = File::open(path).expect("Failed to open rom file");
    rom_file.read_to_end(&mut input).unwrap();
    Ines::parse(&input).unwrap()
}

fn start_game(nes: &mut DynamicNes, rng_bump: u32) {
    for _ in 0..(300 + rng_bump) {
        nes.run_for_frame(&mut NoRenderOutput);
    }
    nes.controller1_mut().set_start();
    for _ in 0..10 {
        nes.run_for_frame(&mut NoRenderOutput);
    }
    nes.controller1_mut().clear_start();
    for _ in 0..10 {
        nes.run_for_frame(&mut NoRenderOutput);
    }
    nes.controller1_mut().set_start();
    for _ in 0..10 {
        nes.run_for_frame(&mut NoRenderOutput);
    }
    nes.controller1_mut().clear_start();
    for _ in 0..10 {
        nes.run_for_frame(&mut NoRenderOutput);
    }
    nes.controller1_mut().set_start();
    for _ in 0..10 {
        nes.run_for_frame(&mut NoRenderOutput);
    }
    nes.controller1_mut().clear_start();
    for _ in 0..10 {
        nes.run_for_frame(&mut NoRenderOutput);
    }
    nes.controller1_mut().set_start();
    for _ in 0..10 {
        nes.run_for_frame(&mut NoRenderOutput);
    }
    nes.controller1_mut().clear_start();
    for _ in 0..10 {
        nes.run_for_frame(&mut NoRenderOutput);
    }
}

fn main() {
    use meap::Parser;
    let args = Args::parser().with_help_default().parse_env_or_exit();
    let ines = ines_from_file(args.rom_path.as_str());
    let mut nes = DynamicNes::from_ines(&ines).unwrap();
    start_game(&mut nes, 0);
    let mut renderer = GifRenderer::new(File::create(args.gif_path.as_str()).unwrap());
    for _ in 0..1000 {
        let mut frame = GifFrame::new();
        nes.run_for_frame(&mut frame);
        renderer.add(&frame);
    }
}
