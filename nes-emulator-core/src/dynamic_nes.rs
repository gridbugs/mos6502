use crate::mapper::{self, mmc1, nrom, PersistentState, PersistentStateError};
use crate::nes::{Controller, Nes, RunForCycles, RunForCyclesRegular};
use analyser::{Analysis, MemoryMap};
use ines::Ines;
use mos6502_model::{machine::MemoryReadOnly, Address};
use nes_render_output::RenderOutput;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum DynamicNes {
    NromHorizontal(Nes<nrom::Nrom<nrom::Horizontal>>),
    NromVertical(Nes<nrom::Nrom<nrom::Vertical>>),
    Mmc1(Nes<mmc1::Mmc1>),
}

#[derive(Debug)]
pub enum Error {
    UnexpectedFormat(mapper::Error),
    InesParseError(ines::Error),
    DeserializeError(bincode::Error),
}

impl From<mapper::Error> for Error {
    fn from(e: mapper::Error) -> Self {
        Error::UnexpectedFormat(e)
    }
}

impl DynamicNes {
    pub fn from_ines(ines: &Ines) -> Result<Self, Error> {
        let &Ines {
            ref header,
            ref prg_rom,
            ref chr_rom,
        } = ines;
        use ines::Mapper::*;
        use ines::Mirroring::*;
        use mmc1::Mmc1;
        use nrom::Nrom;
        use DynamicNes as D;
        let mapper = header.mapper;
        let mirroring = header.mirroring;
        let dynamic_nes = match mapper {
            Nrom => match mirroring {
                Horizontal => {
                    D::NromHorizontal(Nes::new(Nrom::new(nrom::Horizontal, &prg_rom, &chr_rom)?))
                }
                Vertical => {
                    D::NromVertical(Nes::new(Nrom::new(nrom::Vertical, &prg_rom, &chr_rom)?))
                }
            },
            Mmc1 => D::Mmc1(Nes::new(Mmc1::new(&prg_rom, &chr_rom)?)),
        };
        Ok(dynamic_nes)
    }

    pub fn load_persistent_state(
        &mut self,
        ps: &PersistentState,
    ) -> Result<(), PersistentStateError> {
        match self {
            DynamicNes::NromHorizontal(n) => n.load_persistent_state(ps),
            DynamicNes::NromVertical(n) => n.load_persistent_state(ps),
            DynamicNes::Mmc1(n) => n.load_persistent_state(ps),
        }
    }

    pub fn run_for_frame_general<R: RunForCycles, O: RenderOutput>(
        &mut self,
        run: &mut R,
        render_output: &mut O,
    ) {
        match self {
            DynamicNes::NromHorizontal(n) => n.run_for_frame_general(run, render_output, None),
            DynamicNes::NromVertical(n) => n.run_for_frame_general(run, render_output, None),
            DynamicNes::Mmc1(n) => n.run_for_frame_general(run, render_output, None),
        }
    }

    pub fn run_for_frame<RO: RenderOutput>(&mut self, render_output: &mut RO) {
        self.run_for_frame_general(&mut RunForCyclesRegular, render_output);
    }

    pub fn controller1_mut(&mut self) -> &mut Controller {
        match self {
            DynamicNes::NromHorizontal(n) => n.controller1_mut(),
            DynamicNes::NromVertical(n) => n.controller1_mut(),
            DynamicNes::Mmc1(n) => n.controller1_mut(),
        }
    }

    pub fn analyse(&self) -> Analysis {
        Analysis::analyse(self, self, None)
    }
}

impl MemoryMap for DynamicNes {
    fn normalise_function_call<MRO: MemoryReadOnly>(&self, a: Address, m: &MRO) -> Option<Address> {
        match self {
            DynamicNes::NromHorizontal(n) => n.mapper().normalise_function_call(a, m),
            DynamicNes::NromVertical(n) => n.mapper().normalise_function_call(a, m),
            DynamicNes::Mmc1(n) => n.mapper().normalise_function_call(a, m),
        }
    }
}

impl MemoryReadOnly for DynamicNes {
    fn read_u8_read_only(&self, a: Address) -> u8 {
        match self {
            DynamicNes::NromHorizontal(n) => n.devices_with_oam().read_u8_read_only(a),
            DynamicNes::NromVertical(n) => n.devices_with_oam().read_u8_read_only(a),
            DynamicNes::Mmc1(n) => n.devices_with_oam().read_u8_read_only(a),
        }
    }
}
