use crate::game_file::GameFile;

use super::Mapper;

pub struct Mapper000 {
    game: GameFile,
    // has_ram: bool,
}

impl Mapper for Mapper000 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        if game.mapper != 0 {
            return Err("Unexpected mapper");
        }
        if game.prg_rom().len() != 0x2000 && game.prg_rom().len() != 0x4000 {
            return Err("Unexpected prg rom size");
        }
        if game.chr_rom().len() != 0x2000 {
            return Err("Unexpected chr rom size");
        }
        // TODO add ram support
        Ok(Mapper000 {
            game,
            // has_ram: false,
        })
    }

    fn cpu_address_mapped(&self, address: u16) -> bool {
        match address {
            0x8000..=0xFFFF => true,
            // TODO add ram support
            // 0x6000..=0x7FFF if self.has_ram => true,
            _ => false,
        }
    }

    fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x8000..=0xFFFF => self.game.prg_rom()[(address - 0x8000) as usize & (self.game.prg_rom().len() - 1)],
            // TODO add ram support
            // 0x6000..=0x7FFF if has_ram => {},
            _ => panic!("read out of bounds"),
        }
    }

    fn cpu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x8000..=0xFFFF => {
                eprintln!("write to rom ignored");
            }
            // TODO add ram support
            // 0x6000..=0x7FFF => {
            //     g.prg_rom()[(address - 0x6000) as usize] = byte;
            // }
            _ => panic!("write out of bounds"),
        }
    }

    fn ppu_read(&mut self, address: u16) -> u8 {
        todo!("Mapper ppu read at {:04X} not implemented", address)
    }

    fn ppu_write(&mut self, address: u16, byte: u8) {
        todo!("Mapper ppu write at {:04X} not implemented", address)
    }
}
