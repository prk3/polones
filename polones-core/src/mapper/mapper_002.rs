use crate::game_file::GameFile;
use crate::ram::Ram;

use super::Mapper;

pub struct Mapper002 {
    game: GameFile,
    prg_rom_bank: u8,
    chr_ram: Option<Ram<{ 8 * 1024 }>>,
}

impl Mapper for Mapper002 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        if game.prg_rom().is_empty() {
            return Err("Mapper 002: Unexpected prg rom size");
        }

        Ok(Self {
            prg_rom_bank: 0,
            chr_ram: if game.chr_rom().is_some() {
                None
            } else {
                Some(Ram::new())
            },
            game,
        })
    }

    fn cpu_address_mapped(&self, address: u16) -> bool {
        (0x8000..=0xFFFF).contains(&address)
    }

    fn cpu_read(&mut self, address: u16) -> u8 {
        // TODO implement bus conflicts based on submapper and format
        match address {
            0x8000..=0xBFFF => self.game.prg_rom()[((self.prg_rom_bank as usize) << 14)
                & (self.game.prg_rom().len() - 1)
                | (address as usize & 0x3FFF)],
            0xC000..=0xFFFF => self.game.prg_rom()
                [(self.game.prg_rom().len() - 0x4000) | (address as usize & 0x3FFF)],
            _ => panic!("Mapper 002: CPU read from {:04X} out of bounds.", address),
        }
    }

    fn cpu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x8000..=0xFFFF => {
                self.prg_rom_bank = byte;
            }
            _ => panic!("Mapper 002: CPU write to {:04X} out of bounds.", address),
        }
    }

    fn ppu_address_mapped(&self, address: u16) -> bool {
        (0x0000..=0x1FFF).contains(&address)
    }

    fn ppu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF if self.game.chr_rom().is_some() => {
                self.game.chr_rom().unwrap()[address as usize]
            }
            0x0000..=0x1FFF => self.chr_ram.as_ref().unwrap().read(address as usize),
            _ => panic!("Mapper 002: PPU read of {:04X} out of bounds.", address),
        }
    }

    fn ppu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF if self.game.chr_rom().is_some() => {
                eprintln!("Mapper 002: PPU write to {:04X} ignored.", address);
            }
            0x0000..=0x1FFF => {
                self.chr_ram.as_mut().unwrap().write(address as usize, byte);
            }
            _ => panic!("Mapper 002: PPU write to {:04x} out of bounds.", address),
        }
    }

    fn ppu_nametable_address_mapped(&self, address: u16) -> u16 {
        if self.game.mirroring_vertical {
            address & 0b0000_0111_1111_1111
        } else {
            (address & 0b0000_0011_1111_1111) | ((address & 0b0000_1000_0000_0000) >> 1)
        }
    }
}
