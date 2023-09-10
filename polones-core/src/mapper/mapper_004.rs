use crate::cpu::Cpu;
use crate::game_file::{FileFormat, GameFile};
use crate::ram::Ram;

use super::{DebugValue, Mapper};

pub struct Mapper004 {
    game: GameFile,
    ram: Option<Ram<{ 8 * 1024 }>>,
    bank_to_update: u8,
    prg_rom_bank_mode: bool,
    chr_a12_inversion: bool,
    nametable_mirroring: bool,
    ram_write_protection: bool,
    ram_enable: bool,
    irq_latch: u8,
    irq_counter: u8,
    irq_enabled: bool,
    irq_reload: bool,
    irq_requested: bool,
    r0: u8,
    r1: u8,
    r2: u8,
    r3: u8,
    r4: u8,
    r5: u8,
    r6: u8,
    r7: u8,
    cycle_count_a12_1: u64,
    cycle_count: u64,
}

impl Mapper004 {
    fn update_a12(&mut self, address: u16) {
        let a12_1 = address & 0b00010000_00000000 > 0;

        if a12_1 && self.cycle_count.abs_diff(self.cycle_count_a12_1) > 4 {
            if self.irq_counter == 0 || self.irq_reload {
                self.irq_counter = self.irq_latch;
                self.irq_reload = false;
            } else {
                self.irq_counter = self.irq_counter.saturating_sub(1);
            }
            if self.irq_counter == 0 && self.irq_enabled {
                self.irq_requested = true;
            }
        }

        if a12_1 {
            self.cycle_count_a12_1 = self.cycle_count;
        }
    }
}

impl Mapper for Mapper004 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        Ok(Self {
            // Only Nes 2.0 can tell us if ram is present. For other formats assume present.
            ram: (game.format == FileFormat::Nes20 && game.prg_ram_size.is_some()
                || game.format != FileFormat::Nes20)
                .then(|| Ram::new()),
            bank_to_update: 0,
            prg_rom_bank_mode: false,
            chr_a12_inversion: false,
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            nametable_mirroring: false,
            ram_write_protection: false,
            ram_enable: false,
            irq_latch: 0,
            irq_counter: 0,
            irq_enabled: false,
            irq_reload: false,
            irq_requested: false,

            // cycle count
            cycle_count_a12_1: 0,
            cycle_count: 10,

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
        let rel = (address & 0x1FFF) as usize;
        match address {
            0x6000..=0x7FFF => match &self.ram {
                Some(ram) if self.ram_enable => ram.read((address - 0x6000) as usize),
                _ => 0,
            },
            0x8000..=0x9FFF => {
                if self.prg_rom_bank_mode {
                    self.game.prg_rom()[(self.game.prg_rom().len() - 0x4000) as usize | rel]
                } else {
                    self.game.prg_rom()[((self.r6 as usize) << 13) | rel]
                }
            }
            0xA000..=0xBFFF => self.game.prg_rom()[(self.r7 as usize) << 13 | rel],
            0xC000..=0xDFFF => {
                if self.prg_rom_bank_mode {
                    self.game.prg_rom()[(self.r6 as usize) << 13 | rel]
                } else {
                    self.game.prg_rom()[(self.game.prg_rom().len() - 0x4000) as usize | rel]
                }
            }
            0xE000..=0xFFFF => {
                self.game.prg_rom()[(self.game.prg_rom().len() - 0x2000) as usize | rel]
            }
            _ => panic!("Mapper 004: CPU read from {:04X} out of bounds.", address),
        }
    }

    fn cpu_write(&mut self, address: u16, byte: u8) {
        match address {
            0x6000..=0x7FFF => match &mut self.ram {
                Some(ram) if self.ram_enable && !self.ram_write_protection => {
                    ram.write((address - 0x6000) as usize, byte);
                }
                _ => {}
            },
            0x8000..=0x9FFF if address % 2 == 0 => {
                self.bank_to_update = byte & 0b111;
                self.prg_rom_bank_mode = byte & 0b0100_0000 > 0;
                self.chr_a12_inversion = byte & 0b1000_0000 > 0;
            }
            0x8000..=0x9FFF => match self.bank_to_update {
                0 => self.r0 = byte & 0b1111_1110,
                1 => self.r1 = byte & 0b1111_1110,
                2 => self.r2 = byte,
                3 => self.r3 = byte,
                4 => self.r4 = byte,
                5 => self.r5 = byte,
                6 => self.r6 = byte & 0b0011_1111,
                7 => self.r7 = byte & 0b0011_1111,
                _ => unreachable!(),
            },
            0xA000..=0xBFFF if address % 2 == 0 => {
                self.nametable_mirroring = byte & 1 > 0;
            }
            0xA000..=0xBFFF => {
                self.ram_write_protection = byte & 0b0100_0000 > 0;
                self.ram_enable = byte & 0b1000_0000 > 0;
            }
            0xC000..=0xDFFF if address % 2 == 0 => {
                self.irq_latch = byte;
            }
            0xC000..=0xDFFF => {
                self.irq_reload = true;
            }
            0xE000..=0xFFFF if address % 2 == 0 => {
                self.irq_enabled = false;
            }
            0xE000..=0xFFFF => {
                self.irq_enabled = true;
            }
            _ => panic!("Mapper 004: CPU write to {:04X} out of bounds.", address),
        }
    }

    fn ppu_address_mapped(&self, address: u16) -> bool {
        match address {
            0x0000..=0x1FFF => true,
            _ => false,
        }
    }

    fn ppu_read(&mut self, address: u16) -> u8 {
        let rel = (address & 0x03FF) as usize;
        let mask = self.game.chr_rom().unwrap().len() - 1;
        let byte = match address {
            0x0000..=0x03FF => {
                if self.chr_a12_inversion {
                    self.game.chr_rom().unwrap()[((self.r2 as usize) << 10 | rel) & mask]
                } else {
                    self.game.chr_rom().unwrap()[((self.r0 as usize) << 10 | rel) & mask]
                }
            }
            0x0400..=0x07FF => {
                if self.chr_a12_inversion {
                    self.game.chr_rom().unwrap()[((self.r3 as usize) << 10 | rel) & mask]
                } else {
                    self.game.chr_rom().unwrap()[(((self.r0 | 1) as usize) << 10 | rel) & mask]
                }
            }
            0x0800..=0x0BFF => {
                if self.chr_a12_inversion {
                    self.game.chr_rom().unwrap()[((self.r4 as usize) << 10 | rel) & mask]
                } else {
                    self.game.chr_rom().unwrap()[((self.r1 as usize) << 10 | rel) & mask]
                }
            }
            0x0C00..=0x0FFF => {
                if self.chr_a12_inversion {
                    self.game.chr_rom().unwrap()[((self.r5 as usize) << 10 | rel) & mask]
                } else {
                    self.game.chr_rom().unwrap()[(((self.r1 | 1) as usize) << 10 | rel) & mask]
                }
            }
            0x1000..=0x13FF => {
                if self.chr_a12_inversion {
                    self.game.chr_rom().unwrap()[((self.r0 as usize) << 10 | rel) & mask]
                } else {
                    self.game.chr_rom().unwrap()[((self.r2 as usize) << 10 | rel) & mask]
                }
            }
            0x1400..=0x17FF => {
                if self.chr_a12_inversion {
                    self.game.chr_rom().unwrap()[(((self.r0 | 1) as usize) << 10 | rel) & mask]
                } else {
                    self.game.chr_rom().unwrap()[((self.r3 as usize) << 10 | rel) & mask]
                }
            }
            0x1800..=0x1BFF => {
                if self.chr_a12_inversion {
                    self.game.chr_rom().unwrap()[((self.r1 as usize) << 10 | rel) & mask]
                } else {
                    self.game.chr_rom().unwrap()[((self.r4 as usize) << 10 | rel) & mask]
                }
            }
            0x1C00..=0x1FFF => {
                if self.chr_a12_inversion {
                    self.game.chr_rom().unwrap()[(((self.r1 | 1) as usize) << 10 | rel) & mask]
                } else {
                    self.game.chr_rom().unwrap()[((self.r5 as usize) << 10 | rel) & mask]
                }
            }
            _ => panic!("Mapper 004: PPU read of {:04X} out of bounds.", address),
        };
        self.update_a12(address);
        byte
    }

    fn ppu_write(&mut self, address: u16, _byte: u8) {
        match address {
            0x0000..=0x1FFF => {
                eprintln!("Mapper 004: PPU write to {:04X} ignored.", address);
            }
            _ => panic!("Mapper 004: PPU write to {:04x} out of bounds.", address),
        };
        self.update_a12(address);
    }

    fn ppu_nametable_address_mapped(&self, address: u16) -> u16 {
        if self.nametable_mirroring == false {
            address & 0b0000_0111_1111_1111
        } else {
            (address & 0b0000_0011_1111_1111) | ((address & 0b0000_1000_0000_0000) >> 1)
        }
    }

    fn tick(&mut self, cpu: &mut Cpu) {
        if self.irq_requested {
            self.irq_requested = false;
            cpu.irq();
        }
        self.cycle_count += 1;
    }

    fn gather_debug_info(&self) -> Vec<(&'static str, DebugValue)> {
        vec![
            ("mapper", DebugValue::Dec(self.game.mapper as u64)),
            ("ram", DebugValue::Dec(self.ram.is_some() as u64)),
            (
                "bank_to_update",
                DebugValue::Dec(self.bank_to_update as u64),
            ),
            (
                "prg_rom_bank_mode",
                DebugValue::Dec(self.prg_rom_bank_mode as u64),
            ),
            (
                "chr_a12_inversion",
                DebugValue::Dec(self.chr_a12_inversion as u64),
            ),
            (
                "nametable_mirroring",
                DebugValue::Dec(self.nametable_mirroring as u64),
            ),
            (
                "ram_write_protection",
                DebugValue::Dec(self.ram_write_protection as u64),
            ),
            ("ram_enable", DebugValue::Dec(self.ram_enable as u64)),
            ("irq_latch", DebugValue::U8Hex(self.irq_latch)),
            ("irq_counter", DebugValue::U8Hex(self.irq_counter)),
            ("irq_enabled", DebugValue::Dec(self.irq_enabled as u64)),
            ("irq_requested", DebugValue::Dec(self.irq_requested as u64)),
            ("r0", DebugValue::U8Hex(self.r0)),
            ("r1", DebugValue::U8Hex(self.r1)),
            ("r2", DebugValue::U8Hex(self.r2)),
            ("r3", DebugValue::U8Hex(self.r3)),
            ("r4", DebugValue::U8Hex(self.r4)),
            ("r5", DebugValue::U8Hex(self.r5)),
            ("r6", DebugValue::U8Hex(self.r6)),
            ("r7", DebugValue::U8Hex(self.r7)),
            (
                "cycle_count_a12_1",
                DebugValue::Dec(self.cycle_count_a12_1 as u64),
            ),
            ("cycle_count", DebugValue::Dec(self.cycle_count as u64)),
        ]
    }
}
