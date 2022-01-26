use crate::cpu::Cpu;
use crate::game_file::GameFile;
use crate::io::Io;
use crate::mapper::{mapper_from_game_file, Mapper};
use crate::ppu::Ppu;
use crate::ram::Ram;

pub type Frame = [[(u8, u8, u8); 256]; 240];

/// Abstraction over main display device.
pub trait Display {
    /// Draws a frame to the screen.
    fn draw(&mut self, frame: Box<Frame>);
}

pub enum PortState {
    Unplugged,
    Gamepad {
        up: bool,
        down: bool,
        left: bool,
        right: bool,
        select: bool,
        start: bool,
        a: bool,
        b: bool,
    },
}

/// Abstraction over input sources (pads).
pub trait Input {
    fn read_port_1(&mut self) -> PortState;
    fn read_port_2(&mut self) -> PortState;
}

/// Abstraction over audio interface.
pub trait Audio {
    /// Plays audio passed as a parameter.
    fn play(&mut self, audio: ());
}

/// Structure representing the entire console.
pub struct Nes {
    pub cpu: Cpu,
    pub oam_dma: OamDma,
    pub io: Io,
    pub ppu: Ppu,
    pub mapper: Box<dyn Mapper>,
    pub cpu_ram: Ram<{ 2 * 1024 }>,
    pub ppu_nametable_ram: Ram<{ 2 * 1024 }>,
    pub ppu_palette_ram: Ram<32>,
}

pub struct OamDma {
    pub page: Option<u8>,
}

impl OamDma {
    pub fn new() -> Self {
        Self { page: None }
    }
    pub fn write(&mut self, _address: u16, value: u8) {
        self.page = Some(value);
    }
    pub fn tick(&mut self, cpu: &mut Cpu) {
        if let Some(page) = self.page {
            cpu.dma(page);
            self.page = None;
        }
    }
}

impl Nes {
    pub fn new<D: Display + 'static, I: Input + 'static>(
        game: GameFile,
        display: D,
        input: I,
    ) -> Result<Self, &'static str> {
        let mapper = mapper_from_game_file(game)?;
        let cpu = Cpu::new();
        let cpu_ram = Ram::new();
        let oam_dma = OamDma::new();
        let ppu = Ppu::new(Box::new(display));
        let ppu_nametable_ram = Ram::new();
        let ppu_palette_ram = Ram::new();
        let io = Io::new(Box::new(input));

        let mut nes = Self {
            mapper,
            cpu,
            cpu_ram,
            oam_dma,
            ppu,
            ppu_nametable_ram,
            ppu_palette_ram,
            io,
        };

        let (cpu, mut cpu_bus) = nes.split_into_cpu_and_bus();
        cpu.reset(&mut cpu_bus);

        Ok(nes)
    }

    pub fn run_one_cpu_tick(&mut self) {
        let (mut cpu, mut cpu_bus) = self.split_into_cpu_and_bus();
        cpu.tick(&mut cpu_bus);
        cpu_bus.oam_dma.tick(cpu);

        let (ppu, mut ppu_bus) = cpu_bus.split_into_ppu_and_bus();
        ppu.tick(&mut cpu, &mut ppu_bus);
        ppu.tick(&mut cpu, &mut ppu_bus);
        ppu.tick(&mut cpu, &mut ppu_bus);
    }

    pub fn run_one_cpu_instruction(&mut self) {
        while !self.cpu.finished_instruction() {
            self.run_one_cpu_tick();
        }
        self.run_one_cpu_tick();
        while !self.cpu.finished_instruction() {
            self.run_one_cpu_tick();
        }
    }

    pub fn split_into_cpu_and_bus(&mut self) -> (&mut Cpu, CpuBus) {
        let Nes {
            cpu,
            oam_dma,
            io,
            ppu,
            mapper,
            cpu_ram,
            ppu_nametable_ram,
            ppu_palette_ram,
            ..
        } = self;
        (
            cpu,
            CpuBus {
                oam_dma,
                io,
                ppu,
                mapper,
                cpu_ram,
                ppu_nametable_ram,
                ppu_palette_ram,
            },
        )
    }
}

pub struct CpuBus<'a> {
    pub oam_dma: &'a mut OamDma,
    pub io: &'a mut Io,
    pub ppu: &'a mut Ppu,
    pub mapper: &'a mut Box<dyn Mapper>,
    pub cpu_ram: &'a mut Ram<{ 2 * 1024 }>,
    pub ppu_nametable_ram: &'a mut Ram<{ 2 * 1024 }>,
    pub ppu_palette_ram: &'a mut Ram<32>,
}

impl<'a> CpuBus<'a> {
    pub fn split_into_ppu_and_bus(&mut self) -> (&mut Ppu, PpuBus) {
        let CpuBus {
            ppu,
            mapper,
            ppu_nametable_ram,
            ppu_palette_ram,
            ..
        } = self;
        (
            ppu,
            PpuBus {
                mapper,
                ppu_nametable_ram,
                ppu_palette_ram,
            },
        )
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.cpu_ram.read(address as usize),
            0x2000..=0x3FFF => {
                let (ppu, mut ppu_bus) = self.split_into_ppu_and_bus();
                ppu.read(address, &mut ppu_bus)
            }
            0x4016..=0x4017 => self.io.read(address),
            address if self.mapper.cpu_address_mapped(address) => {
                self.mapper.cpu_read(address)
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

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.cpu_ram.write(address as usize, value),
            0x2000..=0x3FFF => {
                let (ppu, mut ppu_bus) = self.split_into_ppu_and_bus();
                ppu.write(address, value, &mut ppu_bus);
            }
            0x4014 => self.oam_dma.write(address, value),
            0x4016..=0x4017 => self.io.write(address, value),
            address if self.mapper.cpu_address_mapped(address) => {
                self.mapper.cpu_write(address, value)
            }
            _ => {
                eprintln!(
                    "CpuBus: CPU bus write to unmapped address {:04x} ignored.",
                    address
                );
            }
        }
    }
}

pub struct PpuBus<'a> {
    pub mapper: &'a mut Box<dyn Mapper>,
    pub ppu_nametable_ram: &'a mut Ram<{ 2 * 1024 }>,
    pub ppu_palette_ram: &'a mut Ram<32>,
}

impl<'a> PpuBus<'a> {
    pub fn read(&mut self, address: u16) -> u8 {
        match address & 0x3FFF {
            0x3F00..=0x3FFF => self.ppu_palette_ram.read(address as usize),
            _a if self.mapper.ppu_address_mapped(address) => self.mapper.ppu_read(address),
            0x2000..=0x3EFF => self
                .ppu_nametable_ram
                .read(self.mapper.ppu_nametable_address_mapped(address) as usize),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address & 0x3FFF {
            _a @ 0x3F00..=0x3FFF => {
                if address & 0b11 == 0 {
                    self.ppu_palette_ram
                        .write(address as usize & 0b11101111, value);
                    self.ppu_palette_ram
                        .write(address as usize | 0b00010000, value);
                } else {
                    self.ppu_palette_ram.write(address as usize, value);
                }
            }
            _a if self.mapper.ppu_address_mapped(address) => self.mapper.ppu_write(address, value),
            0x2000..=0x3EFF => self.ppu_nametable_ram.write(
                self.mapper.ppu_nametable_address_mapped(address) as usize,
                value,
            ),
            _ => unreachable!(),
        }
    }
}
