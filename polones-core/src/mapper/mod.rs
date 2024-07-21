use crate::cpu::Cpu;
use crate::game_file::GameFile;

mod mapper_000;
mod mapper_001;
mod mapper_002;
mod mapper_003;
mod mapper_004;
mod mapper_007;
mod mapper_009;

type DynMapper = Box<dyn Mapper + Send + 'static>;

pub enum DebugValue {
    Dec(u64),
    U8Hex(u8),
    U16Hex(u16),
}

pub trait Mapper {
    fn from_game(game: GameFile) -> Result<Self, &'static str>
    where
        Self: Sized;
    fn cpu_address_mapped(&self, address: u16) -> bool;
    fn cpu_read(&mut self, address: u16) -> u8;
    fn cpu_write(&mut self, address: u16, byte: u8);
    fn ppu_address_mapped(&self, address: u16) -> bool;
    fn ppu_read(&mut self, address: u16) -> u8;
    fn ppu_write(&mut self, address: u16, byte: u8);
    fn ppu_nametable_address_mapped(&self, address: u16) -> u16;
    fn tick(&mut self, cpu: &mut Cpu);
    fn gather_debug_info(&self) -> Vec<(&'static str, DebugValue)> { Vec::new() }
}

pub fn mapper_from_game_file(game: GameFile) -> Result<Box<dyn Mapper + Send + 'static>, &'static str> {
    match (game.mapper, game.submapper) {
        (0, _) => {
            mapper_000::Mapper000::from_game(game).map(|mapper| Box::new(mapper) as DynMapper)
        }
        (1, Some(5)) => Err("unsupported mapper"), // todo mapper 155
        (1, _) => {
            mapper_001::Mapper001::from_game(game).map(|mapper| Box::new(mapper) as DynMapper)
        }
        (2, _) => {
            mapper_002::Mapper002::from_game(game).map(|mapper| Box::new(mapper) as DynMapper)
        }
        (3, _) => {
            mapper_003::Mapper003::from_game(game).map(|mapper| Box::new(mapper) as DynMapper)
        }
        (4, _) => {
            mapper_004::Mapper004::from_game(game).map(|mapper| Box::new(mapper) as DynMapper)
        }
        (7, _) => {
            mapper_007::Mapper007::from_game(game).map(|mapper| Box::new(mapper) as DynMapper)
        }
        (9, _) => {
            mapper_009::Mapper009::from_game(game).map(|mapper| Box::new(mapper) as DynMapper)
        }
        _ => Err("unsupported mapper"),
    }
}
