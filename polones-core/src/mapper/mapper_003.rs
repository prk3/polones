use crate::cpu::Cpu;
use crate::game_file::GameFile;

use super::Mapper;

// TODO add audio support
pub struct Mapper003 {
    game: GameFile,
    chr_rom_bank: u8,
}

impl Mapper for Mapper003 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        if game.prg_rom().len() != 16 * 1024 && game.prg_rom().len() != 32 * 1024 {
            return Err("Mapper 003: Unexpected prg rom size");
        }
        if game.chr_rom().is_none() || game.chr_rom().unwrap().len() > 2048 * 1024 {
            return Err("Mapper 003: Unexpected chr rom size");
        }
        if game.prg_ram_size != None {
            return Err("Mapper 003: Unexpected prg ram size");
        }

        Ok(Self {
            game,
            chr_rom_bank: 0,
        })
    }

    fn cpu_address_mapped(&self, address: u16) -> bool {
        (0x8000..=0xFFFF).contains(&address)
    }

    fn cpu_read(&mut self, address: u16) -> u8 {
        // TODO implement bus conflicts based on submapper and format
        match address {
            0x8000..=0xFFFF => {
                self.game.prg_rom()[(address - 0x8000) as usize & (self.game.prg_rom().len() - 1)]
            }
            _ => panic!("Mapper 003: CPU read from {:04X} out of bounds.", address),
        }
    }

    fn cpu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x8000..=0xFFFF => {
                self.chr_rom_bank = byte;
            }
            _ => panic!("Mapper 003: CPU write to {:04X} out of bounds.", address),
        }
    }

    fn ppu_address_mapped(&self, address: u16) -> bool {
        (0x0000..=0x1FFF).contains(&address)
    }

    fn ppu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                let page = (8 * 1024 * self.chr_rom_bank as usize) & (self.game.chr_rom().unwrap().len() - 1);
                let byte = address as usize;
                self.game.chr_rom().unwrap()[page | byte]
            }
            _ => panic!("Mapper 003: PPU read of {:04X} out of bounds.", address),
        }
    }

    fn ppu_write(&mut self, address: u16, _byte: u8) {
        eprintln!("Mapper 003: PPU write to {:04X} out of bounds.", address);
    }

    fn ppu_nametable_address_mapped(&self, address: u16) -> u16 {
        if self.game.mirroring_vertical {
            address & 0b0000_0111_1111_1111
        } else {
            (address & 0b0000_0011_1111_1111) | ((address & 0b0000_1000_0000_0000) >> 1)
        }
    }

    fn tick(&mut self, _cpu: &mut Cpu) {}
}
