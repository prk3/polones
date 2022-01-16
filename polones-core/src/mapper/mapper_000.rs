use crate::game_file::GameFile;
use crate::ram::Ram;

use super::Mapper;

pub struct Mapper000 {
    game: GameFile,
    ram: Option<(usize, Ram<{ 4 * 1024 }>)>, // size + ram
}

impl Mapper for Mapper000 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        if game.prg_rom().len() != 16 * 1024 && game.prg_rom().len() != 32 * 1024 {
            return Err("Mapper 000: Unexpected prg rom size");
        }
        if game.chr_rom().is_none() || game.chr_rom().unwrap().len() != 8 * 1024 {
            return Err("Mapper 000: Unexpected chr rom size");
        }
        if game.prg_ram_size != None
            && game.prg_ram_size != Some(2 * 1024)
            && game.prg_ram_size != Some(4 * 1024)
        {
            return Err("Mapper 000: Unexpected prg ram size");
        }

        Ok(Self {
            ram: game.prg_ram_size.map(|size| (size, Ram::new())),
            game,
        })
    }

    fn cpu_address_mapped(&self, address: u16) -> bool {
        match address {
            0x6000..=0x7FFF if self.ram.is_some() => true,
            0x8000..=0xFFFF => true,
            _ => false,
        }
    }

    fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF if self.ram.is_some() => {
                let (size, ram) = self.ram.as_ref().unwrap();
                ram.read(address as usize & (*size - 1))
            }
            0x8000..=0xFFFF => {
                self.game.prg_rom()[(address - 0x8000) as usize & (self.game.prg_rom().len() - 1)]
            }
            _ => panic!("Mapper 000: CPU read from {:04X} out of bounds.", address),
        }
    }

    fn cpu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x6000..=0x7FFF if self.ram.is_some() => {
                let (size, ram) = self.ram.as_mut().unwrap();
                ram.write(address as usize & (*size - 1), byte)
            }
            0x8000..=0xFFFF => {
                eprintln!("Mapper 000: CPU write to {:04X} ignored.", address);
            }
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
            0x0000..=0x1FFF => self.game.chr_rom().unwrap()[address as usize],
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
        if self.game.mirroring_vertical {
            address & 0b0000_0111_1111_1111
        } else {
            (address & 0b0000_0011_1111_1111) | ((address & 0b0000_1000_0000_0000) >> 1)
        }
    }
}
