mod apu;
pub mod mapper;
pub mod nes;
pub mod ppu;
mod timing;

use ines::Ines;
use mapper::{mmc1, nrom};
use nes::Nes;
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

use mapper::{PersistentState, PersistentStateError};

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
}
