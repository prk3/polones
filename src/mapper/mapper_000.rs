use crate::game_file::GameFile;

use super::Mapper;

pub struct Mapper000 {
    game: GameFile,
    // has_ram: bool,
}

impl Mapper for Mapper000 {
    fn from_game(game: GameFile) -> Result<Self, &'static str> {
        if game.mapper != 0 {
            return Err("Mapper 000: Unexpected mapper");
        }
        if game.prg_rom().len() != 0x2000 && game.prg_rom().len() != 0x4000 {
            return Err("Mapper 000: Unexpected prg rom size");
        }
        if game.chr_rom().len() != 0x2000 {
            return Err("Mapper 000: Unexpected chr rom size");
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
        // In vertical mirroring we zero bit 10
        // In horizontal mirroring we zero bit 11
        address & 0x2FFF & !(0b0000_0100_0000_0000 << self.game.nametable_mirroring_vertical as u8)
    }
}


#[test]
fn test_nametable_address_mapping() {
    let dk = include_bytes!("../../tests/games/Donkey Kong (JU).nes").to_vec();
    let mut game = GameFile::read("donkey kong".into(), dk.clone()).unwrap();
    game.nametable_mirroring_vertical = true;
    let mapper = Mapper000::from_game(game).unwrap();

    assert_eq!(0x2000, mapper.ppu_nametable_address_mapped(0x2000));
    assert_eq!(0x2400, mapper.ppu_nametable_address_mapped(0x2400));
    assert_eq!(0x2000, mapper.ppu_nametable_address_mapped(0x2800));
    assert_eq!(0x2400, mapper.ppu_nametable_address_mapped(0x2C00));
    assert_eq!(0x2000, mapper.ppu_nametable_address_mapped(0x3000));

    let mut game = GameFile::read("donkey kong".into(), dk).unwrap();
    game.nametable_mirroring_vertical = false;
    let mapper = Mapper000::from_game(game).unwrap();

    assert_eq!(0x2000, mapper.ppu_nametable_address_mapped(0x2000));
    assert_eq!(0x2000, mapper.ppu_nametable_address_mapped(0x2400));
    assert_eq!(0x2800, mapper.ppu_nametable_address_mapped(0x2800));
    assert_eq!(0x2800, mapper.ppu_nametable_address_mapped(0x2C00));
    assert_eq!(0x2000, mapper.ppu_nametable_address_mapped(0x3000));
}
