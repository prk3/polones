use crate::cpu::Cpu;
use crate::game_file::GameFile;
use crate::mapper::{mapper_from_game_file, Mapper};
use crate::cpu_ram::CpuRam;
use crate::ppu::Ppu;
use crate::ppu_ram::PpuRam;
use std::cell::RefCell;

pub type Frame = [[(u8, u8, u8); 256]; 240];

/// Abstraction over main display device.
pub trait Display {
    /// Draws a frame to the screen.
    fn display(&mut self, frame: Box<Frame>);
}

pub struct InputData {
    pub pad1: Option<u8>,
    pub pad2: Option<u8>,
}

/// Abstraction over input sources (pads).
pub trait Input {
    /// Returns the state of input devices.
    fn read(&mut self) -> InputData;
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
    pub cpu_ram: RefCell<CpuRam>,

    pub ppu: RefCell<Ppu>,
    pub ppu_ram: RefCell<PpuRam>,

    display: RefCell<Box<dyn Display>>,
    input: RefCell<Box<dyn Input>>,
}

impl Nes {
    pub fn new<D: Display + 'static, I: Input + 'static>(
        game: GameFile,
        display: D,
        input: I,
    ) -> Result<Self, &'static str> {
        let mapper = RefCell::new(mapper_from_game_file(game)?);
        let cpu = RefCell::new(Cpu::new());
        let cpu_ram = RefCell::new(CpuRam::new());
        let ppu = RefCell::new(Ppu::new());
        let ppu_ram = RefCell::new(PpuRam::new());
        let display = RefCell::new(Box::new(display));
        let input = RefCell::new(Box::new(input));

        let nes = Self {
            mapper,
            cpu,
            cpu_ram,
            ppu,
            ppu_ram,
            display,
            input,
        };

        nes.cpu.borrow_mut().reset(&nes);

        Ok(nes)
    }
    pub fn cpu_bus_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.cpu_ram.borrow_mut().read(address),
            0x2000..=0x3FFF => self.ppu.borrow_mut().cpu_read(self, address),
            address if self.mapper.borrow().cpu_address_mapped(address) => {
                self.mapper.borrow_mut().cpu_read(address)
            }
            _ => {
                eprintln!("reading from unmapped address on cpu bus: {:04x}, returning 0.", address);
                0
            }
        }
    }
    pub fn cpu_bus_write(&self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.cpu_ram.borrow_mut().write(address, value),
            0x2000..=0x3FFF => self.ppu.borrow_mut().cpu_write(self, address, value),
            address if self.mapper.borrow().cpu_address_mapped(address) => {
                self.mapper.borrow_mut().cpu_write(address, value)
            }
            _ => {
                eprintln!("writing to unmapped address on cpu bus: {:04x}. Ignoring.", address);
            }
        }
    }
    pub fn ppu_bus_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.mapper.borrow_mut().ppu_read(address),
            0x2000..=0x3EFF => self.ppu_ram.borrow().read(address),
            0x3F00..=0x3FFF => todo!(),
            _ => {
                eprintln!("PPU bus read out of bounds. Returning 0.");
                0
            }
        }
    }
    pub fn ppu_bus_write(&self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.mapper.borrow_mut().ppu_write(address, value),
            0x2000..=0x3EFF => self.ppu_ram.borrow_mut().write(address, value),
            0x3F00..=0x3FFF => todo!(),
            _ => {
                eprintln!("PPU write read out of bounds.");
            }
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
