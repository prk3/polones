use crate::game_file::GameFile;

use super::Mapper;

pub struct Mapper001 {
    game: GameFile,
    // has_ram: bool,
}

impl Mapper for Mapper001 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        if game.mapper != 0 {
            return Err("Mapper 000: Unexpected mapper");
        }
        if game.prg_rom().len() != 0x4000 && game.prg_rom().len() != 0x8000 {
            dbg!(game.prg_rom().len());
            return Err("Mapper 000: Unexpected prg rom size");
        }
        if game.chr_rom().len() != 0x2000 {
            return Err("Mapper 000: Unexpected chr rom size");
        }
        // TODO add ram support
        Ok(Self {
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
            0x8000..=0xFFFF => {
                self.game.prg_rom()[(address - 0x8000) as usize & (self.game.prg_rom().len() - 1)]
            }
            // TODO add ram support
            // 0x6000..=0x7FFF if has_ram => {},
            _ => panic!("Mapper 000: CPU read from {:04X} out of bounds.", address),
        }
    }

    fn cpu_write(&mut self, address: u16, _byte: u8) {
        match address {
            0x8000..=0xFFFF => {
                eprintln!("Mapper 000: CPU write to {:04X} ignored.", address);
            }
            // TODO add ram support
            // 0x6000..=0x7FFF => {
            //     g.prg_rom()[(address - 0x6000) as usize] = byte;
            // }
            _ => panic!("Mapper 000: CPU write to {:04X} out of bounds.", address),
        }
    }

    fn ppu_address_mapped(&self, address: u16) -> bool {
        match address {
            0x0000..=0x1FFF => true,
            _ => false,
        }
    }

    fn ppu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.game.chr_rom()[address as usize],
            _ => panic!("Mapper 000: PPU read of {:04X} out of bounds.", address),
        }
    }

    fn ppu_write(&mut self, address: u16, _byte: u8) {
        match address {
            0x0000..=0x1FFF => {
                eprintln!("Mapper 000: PPU write to {:04X} ignored.", address);
            }
            _ => panic!("Mapper 000: PPU write to {:04x} out of bounds.", address),
        }
    }

    fn ppu_nametable_address_mapped(&self, address: u16) -> u16 {
        // address bits 11-8
        // address   vertical   horizontal
        //    00XX       00XX         00XX
        //    01XX       01XX         00XX
        //    10XX       00XX         01XX
        //    10XX       01XX         01XX
        if self.game.nametable_mirroring_vertical {
            address & 0b0000_0111_1111_1111
        } else {
            (address & 0b0000_0011_1111_1111) | ((address & 0b0000_1000_0000_0000) >> 1)
        }
    }
}
