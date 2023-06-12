use crate::apu::Apu;
use crate::cpu::Cpu;
use crate::game_file::GameFile;
use crate::io::Io;
use crate::mapper::{mapper_from_game_file, Mapper};
use crate::ppu::Ppu;
use crate::ram::Ram;

pub type Frame = [[(u8, u8, u8); 256]; 240];
pub type Sample = u16;

pub struct Display {
    pub frame: Box<Frame>,
    pub cpu_cycle: u64,
    pub version: u32,
}

impl Display {
    fn new() -> Self {
        Self {
            frame: Box::new([[(0, 0, 0); 256]; 240]),
            cpu_cycle: 0,
            version: 0,
        }
    }
}

#[derive(Default, Clone)]
pub enum PortState {
    #[default]
    Unplugged,
    Gamepad(GamepadState),
}

#[derive(Default, Clone)]
pub struct GamepadState {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl GamepadState {
    pub fn to_byte(&self) -> u8 {
        (self.a as u8) << 7
            | (self.b as u8) << 6
            | (self.select as u8) << 5
            | (self.start as u8) << 4
            | (self.up as u8) << 3
            | (self.down as u8) << 2
            | (self.left as u8) << 1
            | (self.right as u8) << 0
    }
    pub fn from_byte(byte: u8) -> Self {
        Self {
            a: byte & 0b10000000 > 0,
            b: byte & 0b01000000 > 0,
            select: byte & 0b00100000 > 0,
            start: byte & 0b00010000 > 0,
            up: byte & 0b00001000 > 0,
            down: byte & 0b00000100 > 0,
            left: byte & 0b00000010 > 0,
            right: byte & 0b00000001 > 0,
        }
    }
}

#[derive(Default)]
pub struct Input {
    pub port_1: PortState,
    pub port_2: PortState,
    /// Integer updated every time ports are read.
    pub read_version: u32,
}

impl Input {
    fn new() -> Self {
        Self {
            port_1: PortState::Unplugged,
            port_2: PortState::Unplugged,
            read_version: 0,
        }
    }
}

pub struct Audio {
    pub samples: Vec<u16>,
    pub version: u32,
}

impl Audio {
    fn new() -> Self {
        Self {
            samples: Vec::new(),
            version: 0,
        }
    }
}

/// Structure representing the entire console.
pub struct Nes {
    pub cpu: Cpu,
    pub oam_dma: OamDma,
    pub apu: Apu,
    pub io: Io,
    pub ppu: Ppu,
    pub mapper: Box<dyn Mapper + Send + 'static>,
    pub cpu_ram: Ram<{ 2 * 1024 }>,
    pub ppu_nametable_ram: Ram<{ 2 * 1024 }>,
    pub ppu_palette_ram: Ram<32>,
    pub display: Display,
    pub input: Input,
    pub audio: Audio,
}

pub struct OamDma {
    pub page: Option<u8>,
}

impl OamDma {
    pub fn new() -> Self {
        Self { page: None }
    }
    pub fn read(&mut self, _address: u16) -> u8 {
        eprintln!("OamDma: Unexpected read. Returning 0.");
        0
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
    pub fn new(game: GameFile) -> Result<Self, &'static str> {
        let mut nes = Self {
            mapper: mapper_from_game_file(game)?,
            cpu: Cpu::new(),
            cpu_ram: Ram::new(),
            oam_dma: OamDma::new(),
            ppu: Ppu::new(),
            ppu_nametable_ram: Ram::new(),
            ppu_palette_ram: Ram::new(),
            apu: Apu::new(),
            io: Io::new(),
            display: Display::new(),
            input: Input::new(),
            audio: Audio::new(),
        };

        let (cpu, mut cpu_bus) = nes.split_into_cpu_and_bus();
        cpu.reset(&mut cpu_bus);

        Ok(nes)
    }

    pub fn run_one_cpu_tick(&mut self) {
        let Nes {
            mapper,
            cpu,
            cpu_ram,
            oam_dma,
            ppu,
            ppu_nametable_ram,
            ppu_palette_ram,
            apu,
            io,
            display,
            input,
            audio,
        } = self;

        let mut peripherals = Peripherals {
            display,
            input,
            audio,
        };
        let mut cpu_bus = CpuBus {
            oam_dma,
            apu,
            io,
            ppu,
            mapper,
            cpu_ram,
            ppu_nametable_ram,
            ppu_palette_ram,
        };

        cpu.tick(&mut cpu_bus);
        cpu_bus.oam_dma.tick(cpu);
        cpu_bus.io.tick(cpu, &mut peripherals);
        cpu_bus.apu.tick(cpu, &mut peripherals);

        let mut ppu_bus = PpuBus {
            mapper,
            ppu_nametable_ram,
            ppu_palette_ram,
        };
        ppu.tick(cpu, &mut ppu_bus, &mut peripherals);
        ppu.tick(cpu, &mut ppu_bus, &mut peripherals);
        ppu.tick(cpu, &mut ppu_bus, &mut peripherals);
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
            apu,
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
                apu,
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
    pub apu: &'a mut Apu,
    pub io: &'a mut Io,
    pub ppu: &'a mut Ppu,
    pub mapper: &'a mut Box<dyn Mapper + Send + 'static>,
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
            0x4014 => self.oam_dma.read(address),
            0x4000..=0x4015 => self.apu.read(address),
            0x4016..=0x4017 => self.io.read(address),
            address if self.mapper.cpu_address_mapped(address) => self.mapper.cpu_read(address),
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
            0x4016 => self.io.write(address, value),
            0x4000..=0x4017 => self.apu.write(address, value),
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
    pub mapper: &'a mut Box<dyn Mapper + Send + 'static>,
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
                        .write(address as usize & 0b11101111, value & 0b00111111);
                    self.ppu_palette_ram
                        .write(address as usize | 0b00010000, value & 0b00111111);
                } else {
                    self.ppu_palette_ram
                        .write(address as usize, value & 0b00111111);
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

pub struct Peripherals<'a> {
    pub display: &'a mut Display,
    pub input: &'a mut Input,
    pub audio: &'a mut Audio,
}
