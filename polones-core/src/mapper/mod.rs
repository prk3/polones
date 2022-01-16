use crate::game_file::GameFile;

mod mapper_000;
mod mapper_001;
mod mapper_002;

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
}

pub fn mapper_from_game_file(game: GameFile) -> Result<Box<dyn Mapper>, &'static str> {
    match (game.mapper, game.submapper) {
        (0, _) => mapper_000::Mapper000::from_game(game).map(|mapper| Box::new(mapper) as Box<dyn Mapper>),
        (1, Some(5)) => Err("unsupported mapper"), // todo mapper 155
        (1, _) => mapper_001::Mapper001::from_game(game).map(|mapper| Box::new(mapper) as Box<dyn Mapper>),
        (2, _) => mapper_002::Mapper002::from_game(game).map(|mapper| Box::new(mapper) as Box<dyn Mapper>),
        _ => Err("unsupported mapper"),
    }
}
