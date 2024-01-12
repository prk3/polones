use crate::cpu::Cpu;
use crate::game_file::GameFile;
use crate::ram::Ram;

use super::{Mapper, DebugValue};

pub struct Mapper007 {
    game: GameFile,
    prg_rom_prefix: usize,
    nametable_address_prefix: u16,
    chr_ram: Ram<{ 8 * 1024 }>,
}

impl Mapper for Mapper007 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        if game.prg_rom().len() > 512 * 1024 || game.prg_rom().len() % (32 * 1024) != 0 {
            return Err("Mapper 007: Unexpected prg rom size");
        }

        Ok(Self {
            prg_rom_prefix: 0,
            nametable_address_prefix: 0,
            chr_ram: Ram::new(),
            game,
        })
    }

    fn cpu_address_mapped(&self, address: u16) -> bool {
        match address {
            0x6000..=0x7FFF => false,
            0x8000..=0xFFFF => true,
            _ => false,
        }
    }

    fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF => {
                eprintln!(
                    "Mapper 007: CPU read from unmapped address {:04X}, returning 0.",
                    address
                );
                0
            }
            0x8000..=0xFFFF => {
                self.game.prg_rom()[(address - 0x8000) as usize | self.prg_rom_prefix]
            }
            _ => panic!("Mapper 007: CPU read from {:04X} out of bounds.", address),
        }
    }

    fn cpu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x6000..=0x7FFF => {
                eprintln!("Mapper 007: CPU write to unmapped address {:04X}.", address);
            }
            0x8000..=0xFFFF => {
                self.prg_rom_prefix = {
                    let bank = (byte & 0b1111) as usize;
                    let bank_size_mask = (self.game.prg_rom().len() as usize >> 15) - 1;
                    (bank & bank_size_mask) << 15
                };
                self.nametable_address_prefix = ((byte & 0b1_0000) as u16) >> 4 << 10;
            }
            _ => panic!("Mapper 007: CPU write to {:04X} out of bounds.", address),
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
            0x0000..=0x1FFF => self.chr_ram.read(address as usize),
            _ => panic!("Mapper 007: PPU read of {:04X} out of bounds.", address),
        }
    }

    fn ppu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.chr_ram.write(address as usize, byte);
            }
            _ => panic!("Mapper 007: PPU write to {:04x} out of bounds.", address),
        }
    }

    fn ppu_nametable_address_mapped(&self, address: u16) -> u16 {
        (address & 0b11_1111_1111) | self.nametable_address_prefix
    }

    fn tick(&mut self, _cpu: &mut Cpu) {}

    fn gather_debug_info(&self) -> Vec<(&'static str, DebugValue)> {
        vec![
            ("mapper", DebugValue::Dec(self.game.mapper as u64)),
            ("prg_rom_bank", DebugValue::U16Hex((self.prg_rom_prefix >> 15) as u16)),
            ("vram_page", DebugValue::U8Hex((self.nametable_address_prefix >> 10) as u8)),
        ]
    }
}
