use crate::cpu::Cpu;
use crate::game_file::GameFile;
use crate::mapper::DebugValue;
use crate::ram::Ram;

use super::Mapper;

pub struct Mapper001 {
    game: GameFile,
    control: u8,
    load_register: u8,
    load_register_bits: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
    ram: Ram<{ 32 * 1024 }>, // TODO make ram optional
    chr_ram: Ram<{ 8 * 1024 }>,
}

impl Mapper for Mapper001 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        Ok(Self {
            game,
            control: 0b01100,
            load_register: 0,
            load_register_bits: 0,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            ram: Ram::new(),
            chr_ram: Ram::new(),
        })
    }

    fn cpu_address_mapped(&self, address: u16) -> bool {
        (0x6000..=0xFFFF).contains(&address)
    }

    fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF => self.ram.read(address as usize - 0x6000),
            0x8000..=0xBFFF => match (self.control >> 2) & 0b11 {
                0 | 1 => self.game.prg_rom()
                    [((self.prg_bank as usize & 0b11110) << 14) | (address as usize & 0x3FFF)],
                2 => self.game.prg_rom()[address as usize & 0x3FFF],
                3 => self.game.prg_rom()
                    [((self.prg_bank as usize) << 14) | (address as usize & 0x3FFF)],
                _ => unreachable!(),
            },
            0xC000..=0xFFFF => match (self.control >> 2) & 0b11 {
                0 | 1 => self.game.prg_rom()
                    [((self.prg_bank as usize | 0b00001) << 14) | (address as usize & 0x3FFF)],
                2 => self.game.prg_rom()
                    [((self.prg_bank as usize) << 14) | (address as usize & 0x3FFF)],
                3 => self.game.prg_rom()
                    [self.game.prg_rom().len() - 0x4000 + (address as usize & 0x3FFF)],
                _ => unreachable!(),
            },
            _ => panic!("Mapper 001: CPU read from {:04X} out of bounds.", address),
        }
    }

    fn cpu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x6000..=0x7FFF => {
                self.ram.write(address as usize - 0x6000, byte);
            }
            0x8000..=0xFFFF => {
                if byte & 0b10000000 > 0 {
                    self.load_register = 0;
                    self.load_register_bits = 0;
                    self.control |= 0x0C;
                } else {
                    self.load_register = (self.load_register >> 1) | ((byte & 1) << 4);
                    self.load_register_bits += 1;

                    if self.load_register_bits == 5 {
                        match address {
                            0x8000..=0x9FFF => self.control = self.load_register,
                            0xA000..=0xBFFF => self.chr_bank_0 = self.load_register,
                            0xC000..=0xDFFF => self.chr_bank_1 = self.load_register,
                            0xE000..=0xFFFF => self.prg_bank = self.load_register,
                            _ => unreachable!(),
                        }
                        self.load_register = 0;
                        self.load_register_bits = 0;
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn ppu_address_mapped(&self, address: u16) -> bool {
        (0x0000..=0x1FFF).contains(&address)
    }

    fn ppu_read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF if self.game.chr_rom().is_none() => {
                self.chr_ram.read(address as usize)
            }
            0x0000..=0x0FFF => {
                self.game.chr_rom().unwrap()[self.lower_chr_bank() | (address as usize & 0x0FFF)]
            }
            0x1000..=0x1FFF => {
                self.game.chr_rom().unwrap()[self.upper_chr_bank() | (address as usize & 0x0FFF)]
            }
            _ => panic!("Mapper 001: PPU read of {:04X} out of bounds.", address),
        }
    }

    fn ppu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF if self.game.chr_rom().is_none() => {
                self.chr_ram.write(address as usize, byte);
            }
            0x0000..=0x1FFF => {
                eprintln!("Mapper 001: PPU write to {:04X} ignored.", address);
            }
            _ => panic!("Mapper 001: PPU write to {:04x} out of bounds.", address),
        }
    }

    fn ppu_nametable_address_mapped(&self, address: u16) -> u16 {
        match self.control & 0b11 {
            0 => address & 0b0000_0011_1111_1111,
            1 => (address & 0b0000_0011_1111_1111) | 0b0000_0100_0000_0000,
            2 => address & 0b0000_0111_1111_1111,
            3 => (address & 0b0000_0011_1111_1111) | ((address & 0b0000_1000_0000_0000) >> 1),
            _ => unreachable!(),
        }
    }

    fn tick(&mut self, _cpu: &mut Cpu) {}

    fn gather_debug_info(&self) -> Vec<(&'static str, DebugValue)> {
        vec![
            ("mapper", DebugValue::Dec(1)),
            ("control", DebugValue::U8Hex(self.control)),
            ("chr_bank_0", DebugValue::U8Hex(self.chr_bank_0)),
            ("chr_bank_1", DebugValue::U8Hex(self.chr_bank_1)),
            ("prg_bank", DebugValue::U8Hex(self.prg_bank)),
            ("load_register", DebugValue::U8Hex(self.load_register)),
            ("load_register_bits", DebugValue::Dec(self.load_register_bits as u64)),
        ]
    }
}

impl Mapper001 {
    fn lower_chr_bank(&self) -> usize {
        if self.control & 0b10000 > 0 {
            0x1000 * self.chr_bank_0 as usize
        } else {
            0x1000 * (self.chr_bank_0 as usize & 0b11110)
        }
    }

    fn upper_chr_bank(&self) -> usize {
        if self.control & 0b10000 > 0 {
            0x1000 * self.chr_bank_1 as usize
        } else {
            0x1000 * (self.chr_bank_0 as usize | 0b00001)
        }
    }
}
