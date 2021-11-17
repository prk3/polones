use crate::game_file::GameFile;

mod mapper_000;

pub trait Mapper {
    fn from_game(game: GameFile) -> Result<Self, &'static str>
    where
        Self: Sized;
    fn cpu_address_mapped(&self, address: u16) -> bool;
    fn cpu_read(&mut self, address: u16) -> u8;
    fn cpu_write(&mut self, address: u16, byte: u8);
    fn ppu_read(&mut self, address: u16) -> u8;
    fn ppu_write(&mut self, address: u16, byte: u8);
}

pub fn mapper_from_game_file(game: GameFile) -> Result<Box<dyn Mapper>, &'static str> {
    match game.mapper {
        0 => mapper_000::Mapper000::from_game(game).map(|mapper| Box::new(mapper) as Box<dyn Mapper>),
        _ => Err("unsupported mapper"),
    }
}
