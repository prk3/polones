use crate::cpu::Cpu;
use crate::game_file::GameFile;
use crate::ram::Ram;

use super::{DebugValue, Mapper};

pub struct Mapper009 {
    game: GameFile,
    prg_rom_bank_select: u8,
    chr_rom_fd_0_bank_select: u8,
    chr_rom_fe_0_bank_select: u8,
    chr_rom_fd_1_bank_select: u8,
    chr_rom_fe_1_bank_select: u8,
    latch_0_fe: bool,
    latch_1_fe: bool,
    prg_ram: Ram<{ 8 * 1024 }>,
    mirroring_horizontal: bool,
}

impl Mapper for Mapper009 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        if game.prg_rom().len() != 128 * 1024 {
            return Err("Mapper 009: Unexpected prg rom size");
        }

        if game.prg_ram_size.is_some() && game.prg_ram_size.unwrap() != 8 * 1024 {
            return Err("Mapper 009: Unexpected prg ram size");
        }

        if game.chr_rom().is_none() || game.chr_rom().unwrap().len() != 128 * 1024 {
            return Err("Mapper 009: Unexpected chr rom size");
        }

        Ok(Self {
            prg_rom_bank_select: 0,
            prg_ram: Ram::new(),
            mirroring_horizontal: false,
            chr_rom_fd_0_bank_select: 0,
            chr_rom_fe_0_bank_select: 0,
            chr_rom_fd_1_bank_select: 0,
            chr_rom_fe_1_bank_select: 0,
            latch_0_fe: false,
            latch_1_fe: false,
            game,
        })
    }

    fn cpu_address_mapped(&self, address: u16) -> bool {
        match address {
            0x6000..=0x7FFF => self.game.prg_ram_size.is_some(),
            0x8000..=0xFFFF => true,
            _ => false,
        }
    }

    fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF => {
                if self.game.prg_ram_size.is_some() {
                    self.prg_ram.read((address - 0x6000) as usize)
                } else {
                    eprintln!(
                        "Mapper 009: CPU read from unmapped address {:04X}, returning 0.",
                        address
                    );
                    0
                }
            }
            0x8000..=0x9FFF => self.game.prg_rom()[((address - 0x8000) as usize
                + (self.prg_rom_bank_select as usize * 8 * 1024))
                & (self.game.prg_rom().len() - 1)],
            0xA000..=0xBFFF => self.game.prg_rom()
                [(address - 0xA000) as usize + self.game.prg_rom().len() - (3 * 8 * 1024)],
            0xC000..=0xDFFF => self.game.prg_rom()
                [(address - 0xC000) as usize + self.game.prg_rom().len() - (2 * 8 * 1024)],
            0xE000..=0xFFFF => self.game.prg_rom()
                [(address - 0xE000) as usize + self.game.prg_rom().len() - (1 * 8 * 1024)],
            _ => panic!("Mapper 009: CPU read from {:04X} out of bounds.", address),
        }
    }

    fn cpu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x6000..=0x7FFF => {
                if self.game.prg_ram_size.is_some() {
                    self.prg_ram.write((address - 0x6000) as usize, byte);
                } else {
                    eprintln!("Mapper 009: CPU write to unmapped address {:04X}.", address);
                }
            }
            0xA000..=0xAFFF => {
                self.prg_rom_bank_select = byte & 0b0000_1111;
            }
            0xB000..=0xBFFF => {
                self.chr_rom_fd_0_bank_select = byte & 0b0001_1111;
            }
            0xC000..=0xCFFF => {
                self.chr_rom_fe_0_bank_select = byte & 0b0001_1111;
            }
            0xD000..=0xDFFF => {
                self.chr_rom_fd_1_bank_select = byte & 0b0001_1111;
            }
            0xE000..=0xEFFF => {
                self.chr_rom_fe_1_bank_select = byte & 0b0001_1111;
            }
            0xF000..=0xFFFF => {
                self.mirroring_horizontal = byte & 1 > 0;
            }
            _ => panic!("Mapper 009: CPU write to {:04X} out of bounds.", address),
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
            0x0000..=0x0FFF => {
                let page = if self.latch_0_fe {
                    self.chr_rom_fe_0_bank_select
                } else {
                    self.chr_rom_fd_0_bank_select
                };
                let byte = self.game.chr_rom().unwrap()[((address as usize)
                    + page as usize * 4 * 1024)
                    & (self.game.chr_rom().unwrap().len() - 1)];

                if address == 0x0FD8 {
                    self.latch_0_fe = false;
                } else if address == 0x0FE8 {
                    self.latch_0_fe = true;
                }

                byte
            }
            0x1000..=0x1FFF => {
                let page = if self.latch_1_fe {
                    self.chr_rom_fe_1_bank_select
                } else {
                    self.chr_rom_fd_1_bank_select
                };
                let byte = self.game.chr_rom().unwrap()[((address as usize - 0x1000)
                    + page as usize * 4 * 1024)
                    & (self.game.chr_rom().unwrap().len() - 1)];

                if (0x1FD8..=0x1FDF).contains(&address) {
                    self.latch_1_fe = false;
                } else if (0x1FE8..=0x1FEF).contains(&address) {
                    self.latch_1_fe = true;
                }

                byte
            }
            _ => panic!("Mapper 009: PPU read of {:04X} out of bounds.", address),
        }
    }

    fn ppu_write(&mut self, address: u16, _byte: u8) {
        match address {
            0x0000..=0x1FFF => {
                eprintln!("Mapper 009: PPU write to {:04X} ignored.", address);
            }
            _ => panic!("Mapper 009: PPU write to {:04x} out of bounds.", address),
        }
    }

    fn ppu_nametable_address_mapped(&self, address: u16) -> u16 {
        if self.mirroring_horizontal {
            (address & 0b0000_0011_1111_1111) | ((address & 0b0000_1000_0000_0000) >> 1)
        } else {
            address & 0b0000_0111_1111_1111
        }
    }

    fn tick(&mut self, _cpu: &mut Cpu) {}

    fn gather_debug_info(&self) -> Vec<(&'static str, DebugValue)> {
        vec![
            ("mapper", DebugValue::Dec(self.game.mapper as u64)),
            (
                "prg_rom_bank_select",
                DebugValue::U8Hex(self.prg_rom_bank_select),
            ),
            (
                "chr_rom_fd_0_bank_s",
                DebugValue::U8Hex(self.chr_rom_fd_0_bank_select),
            ),
            (
                "chr_rom_fe_0_bank_s",
                DebugValue::U8Hex(self.chr_rom_fe_0_bank_select),
            ),
            (
                "chr_rom_fd_1_bank_s",
                DebugValue::U8Hex(self.chr_rom_fd_1_bank_select),
            ),
            (
                "chr_rom_fe_1_bank_s",
                DebugValue::U8Hex(self.chr_rom_fe_1_bank_select),
            ),
            ("latch_0_fe", DebugValue::Dec(self.latch_0_fe as _)),
            ("latch_1_fe", DebugValue::Dec(self.latch_1_fe as _)),
            (
                "mirroring_horizontal",
                DebugValue::Dec(self.mirroring_horizontal as _),
            ),
        ]
    }
}
