use gif_renderer::{Frame as GifFrame, Renderer as GifRenderer};
use ines::Ines;
use mos6502_model::debug::{InstructionType, InstructionWithOperand};
use mos6502_model::machine::{Address, Cpu, Memory, MemoryReadOnly};
use nes_emulator_core::{dynamic_nes::DynamicNes, nes::RunForCycles};
use nes_render_output::NoRenderOutput;
use std::collections::BTreeMap;
use std::fmt;
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

fn start_game(nes: &mut DynamicNes, rng_bump: u32, trace_run: &mut TraceRun) {
    for _ in 0..(300 + rng_bump) {
        nes.run_for_frame_general(trace_run, &mut NoRenderOutput);
    }
    nes.controller1_mut().set_start();
    for _ in 0..10 {
        nes.run_for_frame_general(trace_run, &mut NoRenderOutput);
    }
    nes.controller1_mut().clear_start();
    for _ in 0..10 {
        nes.run_for_frame_general(trace_run, &mut NoRenderOutput);
    }
    nes.controller1_mut().set_start();
    for _ in 0..10 {
        nes.run_for_frame_general(trace_run, &mut NoRenderOutput);
    }
    nes.controller1_mut().clear_start();
    for _ in 0..10 {
        nes.run_for_frame_general(trace_run, &mut NoRenderOutput);
    }
    nes.controller1_mut().set_start();
    for _ in 0..10 {
        nes.run_for_frame_general(trace_run, &mut NoRenderOutput);
    }
    nes.controller1_mut().clear_start();
    for _ in 0..10 {
        nes.run_for_frame_general(trace_run, &mut NoRenderOutput);
    }
    nes.controller1_mut().set_start();
    for _ in 0..10 {
        nes.run_for_frame_general(trace_run, &mut NoRenderOutput);
    }
    nes.controller1_mut().clear_start();
}

#[derive(Debug)]
struct Histogram<T: Ord> {
    counts: BTreeMap<T, u64>,
}

impl<T: Ord> Histogram<T> {
    fn new() -> Self {
        Self {
            counts: BTreeMap::new(),
        }
    }
    fn insert(&mut self, t: T) {
        *self.counts.entry(t).or_insert(0) += 1;
    }
}

#[derive(Debug)]
struct TraceRun {
    nmi_address_histogram: Histogram<Address>,
    function_call_histogram: Histogram<Address>,
}

impl TraceRun {
    fn new() -> Self {
        Self {
            nmi_address_histogram: Histogram::new(),
            function_call_histogram: Histogram::new(),
        }
    }
}

impl Default for TraceRun {
    fn default() -> Self {
        Self::new()
    }
}

impl RunForCycles for TraceRun {
    fn run_for_cycles<M: Memory + MemoryReadOnly>(
        &mut self,
        cpu: &mut Cpu,
        memory: &mut M,
        num_cycles: u32,
    ) {
        if let Some(address) = cpu.retrieve_nmi_return_address_during_nmi(memory) {
            self.nmi_address_histogram.insert(address);
        }
        let mut count = 0;
        while count < num_cycles {
            if let Ok(instruction_with_operand) = InstructionWithOperand::next(cpu, memory) {
                match instruction_with_operand.instruction().instruction_type() {
                    InstructionType::Jsr => {
                        let function_address = instruction_with_operand.operand_u16_le().unwrap();
                        match function_address {
                            _ => (),
                        }
                        self.function_call_histogram.insert(function_address);
                    }
                    _ => (),
                }
            }
            count += cpu.step(memory).unwrap() as u32;
        }
    }
}

impl fmt::Display for TraceRun {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(fmt, "Addresses interrupted by NMI:")?;
        for (address, count) in self.nmi_address_histogram.counts.iter() {
            writeln!(fmt, "0x{:X}: {}", address, count)?;
        }
        writeln!(fmt, "\nFunction calls by frequency:")?;
        let mut calls = self
            .function_call_histogram
            .counts
            .iter()
            .collect::<Vec<_>>();
        calls.sort_by_key(|(_, count)| *count);
        calls.reverse();
        for (address, count) in calls {
            writeln!(fmt, "0x{:X}: {}", address, count)?;
        }
        Ok(())
    }
}

struct State {
    gif_renderer: GifRenderer<File>,
    frame: GifFrame,
    nes: DynamicNes,
    trace_run: TraceRun,
}

impl State {
    fn wait(&mut self, n: u64) {
        for _ in 0..n {
            self.frame.clear();
            self.nes
                .run_for_frame_general(&mut self.trace_run, &mut self.frame);
            self.gif_renderer.add(&self.frame);
        }
    }
}

fn main() {
    use meap::Parser;
    let args = Args::parser().with_help_default().parse_env_or_exit();
    let ines = ines_from_file(args.rom_path.as_str());
    let mut nes = DynamicNes::from_ines(&ines).unwrap();
    let mut trace_run = TraceRun::new();
    start_game(&mut nes, 2, &mut trace_run);
    let gif_renderer = GifRenderer::new(File::create(args.gif_path.as_str()).unwrap());
    let frame = GifFrame::new();
    let analysis = nes.analyse();
    {
        use std::io::Write;
        let mut file = File::create("/tmp/functions.txt").unwrap();
        for (address, trace) in analysis.function_traces() {
            write!(&mut file, "0x{:X}:\n{}\n", address, trace).unwrap();
        }
    }
    let mut state = State {
        gif_renderer,
        frame,
        nes,
        trace_run,
    };
    state.wait(120);
    state.nes.controller1_mut().set_a();
    state.wait(10);
    state.nes.controller1_mut().clear_a();
    state.wait(10);
    state.nes.controller1_mut().set_a();
    state.wait(10);
    state.nes.controller1_mut().clear_a();
    state.wait(10);
    state.nes.controller1_mut().set_left();
    state.wait(40);
    state.nes.controller1_mut().clear_left();
    state.wait(10);
    state.nes.controller1_mut().set_down();
    state.wait(100);
    state.nes.controller1_mut().clear_down();
    state.wait(10);
    state.nes.controller1_mut().set_left();
    state.wait(10);
    state.nes.controller1_mut().clear_left();
    state.wait(10);
    state.nes.controller1_mut().set_a();
    state.wait(10);
    state.nes.controller1_mut().clear_a();
    state.wait(10);
    state.nes.controller1_mut().set_a();
    state.wait(10);
    state.nes.controller1_mut().clear_a();
    state.wait(10);
    state.nes.controller1_mut().set_down();
    state.wait(100);
    state.nes.controller1_mut().clear_down();
    state.wait(10);
    state.nes.controller1_mut().set_b();
    state.wait(10);
    state.nes.controller1_mut().clear_b();
    state.wait(10);
    state.nes.controller1_mut().set_b();
    state.wait(10);
    state.nes.controller1_mut().clear_b();
    state.wait(10);
    state.nes.controller1_mut().set_right();
    state.wait(30);
    state.nes.controller1_mut().clear_right();
    state.wait(10);
    state.nes.controller1_mut().set_down();
    state.wait(100);
    state.nes.controller1_mut().clear_down();
    state.nes.controller1_mut().set_b();
    state.wait(10);
    state.nes.controller1_mut().clear_b();
    state.wait(10);
    state.nes.controller1_mut().set_right();
    state.wait(10);
    state.nes.controller1_mut().clear_right();
    state.wait(10);
    state.nes.controller1_mut().set_down();
    state.wait(100);
    state.nes.controller1_mut().clear_down();
}
