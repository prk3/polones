use crate::cpu::Cpu;
use crate::game_file::GameFile;
use crate::ram::Ram;

use super::Mapper;

pub struct Mapper015 {
    game: GameFile,
    prg_rom_bank_address: u32,
    nametable_mirroring_horizontal: bool,
    a13: u32,
    prg_banking_mode: u8,
    chr_ram: Ram<0x2000>,
}

impl Mapper for Mapper015 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        Ok(Self {
            prg_rom_bank_address: 0,
            nametable_mirroring_horizontal: false,
            a13: 0,
            prg_banking_mode: 0,
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
                    "Mapper 015: CPU bus read from unmapped address {address:04x}, returning 0.",
                );
                0
            }
            0x8000..=0xFFFF => {
                let prg_address = match self.prg_banking_mode {
                    0 => {
                        (self.prg_rom_bank_address & 0b1111_1000_0000_0000_0000)
                            | (address as u32 & 0b0000_0111_1111_1111_1111)
                    }
                    1 => {
                        if address & 0b0000_0100_0000_0000_0000 > 0 {
                            self.prg_rom_bank_address
                                | 0b0000_0001_1100_0000_0000_0000
                                | (address as u32)
                        } else {
                            self.prg_rom_bank_address
                                | (address as u32 & 0b0000_0011_1111_1111_1111)
                        }
                    }
                    2 => {
                        self.prg_rom_bank_address
                            | self.a13
                            | (address as u32 & 0b0000_0001_1111_1111_1111)
                    }
                    3 => {
                        self.prg_rom_bank_address | (address as u32 & 0b0000_0011_1111_1111_1111)
                    },
                    _ => unreachable!(),
                };
                self.game.prg_rom()[prg_address as usize]
            }
            _ => panic!("Mapper 015: CPU read from {address:04X} out of bounds."),
        }
    }

    fn cpu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x6000..=0x7FFF => {
                eprintln!("Mapper 015: CPU write to {address:04X} ignored.");
            }
            0x8000..=0xFFFF => {
                self.prg_rom_bank_address = ((byte & 0b0011_1111) as u32) << 14;
                self.nametable_mirroring_horizontal = byte & 0b0100_0000 > 0;
                self.a13 = ((byte & 0b1000_0000) as u32) << 6;
                self.prg_banking_mode = (address & 0b11) as u8;
            }
            _ => panic!("Mapper 015: CPU write to {address:04X} out of bounds."),
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
            _ => panic!("Mapper 015: PPU read of {address:04X} out of bounds."),
        }
    }

    fn ppu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.chr_ram.write(address as usize, byte);
            }
            _ => panic!("Mapper 015: PPU write to {address:04x} out of bounds."),
        }
    }

    fn ppu_nametable_address_mapped(&self, address: u16) -> u16 {
        // address bits 11-8
        // address   vertical   horizontal
        //    00XX       00XX         00XX
        //    01XX       01XX         00XX
        //    10XX       00XX         01XX
        //    10XX       01XX         01XX
        if !self.nametable_mirroring_horizontal {
            address & 0b0000_0111_1111_1111
        } else {
            (address & 0b0000_0011_1111_1111) | ((address & 0b0000_1000_0000_0000) >> 1)
        }
    }

    fn tick(&mut self, _cpu: &mut Cpu) {}
}
