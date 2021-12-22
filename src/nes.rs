use crate::cpu::Cpu;
use crate::game_file::GameFile;
use crate::io::Io;
use crate::mapper::{mapper_from_game_file, Mapper};
use crate::ppu::Ppu;
use crate::ram::Ram;
use std::cell::RefCell;

pub type Frame = [[(u8, u8, u8); 256]; 240];

/// Abstraction over main display device.
pub trait Display {
    /// Draws a frame to the screen.
    fn display(&mut self, frame: Box<Frame>);
}

/// Abstraction over input sources (pads).
pub trait Input {
    fn read_pad_1(&mut self) -> Option<u8>;
    fn read_pad_2(&mut self) -> Option<u8>;
}

/// Abstraction over audio interface.
pub trait Audio {
    /// Plays audio passed as a parameter.
    fn play(&mut self, audio: ());
}

/// Structure representing the entire console.
pub struct Nes {
    pub mapper: RefCell<Box<dyn Mapper>>,

    pub cpu: RefCell<Cpu>,
    pub cpu_ram: RefCell<Ram<0x800>>,

    pub ppu: RefCell<Ppu>,
    pub ppu_nametable_ram: RefCell<Ram<0x800>>,
    pub ppu_palette_ram: RefCell<Ram<0x20>>,

    display: RefCell<Box<dyn Display>>,
    io: RefCell<Io>,
    pub input: RefCell<Box<dyn Input>>,
}

impl Nes {
    pub fn new<D: Display + 'static, I: Input + 'static>(
        game: GameFile,
        display: D,
        input: I,
    ) -> Result<Self, &'static str> {
        let mapper = RefCell::new(mapper_from_game_file(game)?);
        let cpu = RefCell::new(Cpu::new());
        let cpu_ram = RefCell::new(Ram::new());
        let ppu = RefCell::new(Ppu::new());
        let ppu_nametable_ram = RefCell::new(Ram::new());
        let ppu_palette_ram = RefCell::new(Ram::new());
        let display = RefCell::new(Box::new(display));
        let io = RefCell::new(Io::new());
        let input = RefCell::new(Box::new(input));

        let nes = Self {
            mapper,
            cpu,
            cpu_ram,
            ppu,
            ppu_nametable_ram,
            ppu_palette_ram,
            display,
            io,
            input,
        };

        nes.cpu.borrow_mut().reset(&nes);

        Ok(nes)
    }
    pub fn cpu_bus_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.cpu_ram.borrow_mut().read(address),
            0x2000..=0x3FFF => self.ppu.borrow_mut().cpu_read(self, address),
            0x4016..=0x4017 => self.io.borrow_mut().read(self, address),
            address if self.mapper.borrow().cpu_address_mapped(address) => {
                self.mapper.borrow_mut().cpu_read(address)
            }
            _ => {
                eprintln!(
                    "Nes: CPU bus read from unmapped address {:04x}, returning 0.",
                    address
                );
                0
            }
        }
    }
    pub fn cpu_bus_write(&self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.cpu_ram.borrow_mut().write(address, value),
            0x2000..=0x3FFF => self.ppu.borrow_mut().cpu_write(self, address, value),
            0x4016..=0x4017 => self.io.borrow_mut().write(self, address, value),
            address if self.mapper.borrow().cpu_address_mapped(address) => {
                self.mapper.borrow_mut().cpu_write(address, value)
            }
            _ => {
                eprintln!(
                    "Nes: CPU bus write to unmapped address {:04x} ignored.",
                    address
                );
            }
        }
    }
    pub fn ppu_bus_read(&self, address: u16) -> u8 {
        match address & 0x3FFF {
            0x3F00..=0x3FFF => self.ppu_palette_ram.borrow().read(address),
            _a if self.mapper.borrow().ppu_address_mapped(address) => {
                self.mapper.borrow_mut().ppu_read(address)
            }
            0x2000..=0x3EFF => self
                .ppu_nametable_ram
                .borrow()
                .read(self.mapper.borrow().ppu_nametable_address_mapped(address)),
            _ => unreachable!(),
        }
    }
    pub fn ppu_bus_write(&self, address: u16, value: u8) {
        match address & 0x3FFF {
            0x3F00..=0x3FFF => self.ppu_palette_ram.borrow_mut().write(address, value),
            _a if self.mapper.borrow().ppu_address_mapped(address) => {
                self.mapper.borrow_mut().ppu_write(address, value)
            }
            0x2000..=0x3EFF => self.ppu_nametable_ram.borrow_mut().write(
                self.mapper.borrow().ppu_nametable_address_mapped(address),
                value,
            ),
            _ => unreachable!(),
        }
    }
    pub fn run_one_cpu_tick(&mut self) {
        self.cpu.borrow_mut().tick(&self);
        self.ppu.borrow_mut().tick(&self);
        self.ppu.borrow_mut().tick(&self);
        self.ppu.borrow_mut().tick(&self);
    }
    pub fn run_one_cpu_instruction(&mut self) {
        while !self.cpu.borrow().finished_instruction() {
            self.run_one_cpu_tick();
        }
        self.run_one_cpu_tick();
        while !self.cpu.borrow().finished_instruction() {
            self.run_one_cpu_tick();
        }
    }
}
