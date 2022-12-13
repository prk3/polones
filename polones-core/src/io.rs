use crate::cpu::Cpu;
use crate::nes::{Peripherals, PortState};

pub struct Io {
    latch: u8,
    port_1: PortState,
    port_2: PortState,
    port_1_latched: PortState,
    port_2_latched: PortState,
    /// Holds buttons pressed on gamepad 1. Bits are negated (0 - pressed, 1 - not pressed).
    /// Buttons in MSB to LSB order - A, B, Select, Start, Up, Down, Left, Right.
    gamepad_shift_register_1: u8,
    /// Holds buttons pressed on gamepad 2. Bits are negated (0 - pressed, 1 - not pressed).
    /// Buttons in MSB to LSB order - A, B, Select, Start, Up, Down, Left, Right.
    gamepad_shift_register_2: u8,
    read_version_increment: u32,
}

impl Io {
    pub fn new() -> Self {
        Self {
            latch: 0,
            port_1: PortState::Unplugged,
            port_2: PortState::Unplugged,
            port_1_latched: PortState::Unplugged,
            port_2_latched: PortState::Unplugged,
            gamepad_shift_register_1: 0,
            gamepad_shift_register_2: 0,
            read_version_increment: 0,
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match 0x4016 + (address & 1) {
            0x4016 => match self.port_1_latched {
                PortState::Unplugged => 0,
                PortState::Gamepad { .. } => {
                    // Negate the pattern in shift register to return 1 after 8th read.
                    let result = 0x40 | (!self.gamepad_shift_register_1 & 0b10000000) >> 7;
                    self.gamepad_shift_register_1 <<= 1;
                    result
                }
            },
            0x4017 => match self.port_2_latched {
                PortState::Unplugged => 0,
                PortState::Gamepad { .. } => {
                    // Negate the pattern in shift register to return 1 after 8th read.
                    let result = 0x40 | (!self.gamepad_shift_register_2 & 0b10000000) >> 7;
                    self.gamepad_shift_register_2 <<= 1;
                    result
                }
            },
            _ => unreachable!("IO: Read of {address:04X}"),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match 0x4016 + (address & 1) {
            0x4016 => {
                if self.latch & 1 == 1 && value & 1 == 0 {
                    self.port_1_latched = self.port_1.clone();
                    self.port_2_latched = self.port_2.clone();
                    self.read_version_increment = 1;
                    match &self.port_1_latched {
                        PortState::Unplugged => {}
                        PortState::Gamepad(gamepad) => {
                            // We negate the pattern in shift register to return 1 after 8th read.
                            self.gamepad_shift_register_1 = !gamepad.to_byte();
                        }
                    }
                    match &self.port_2_latched {
                        PortState::Unplugged => {}
                        PortState::Gamepad(gamepad) => {
                            // We negate the pattern in shift register to return 1 after 8th read.
                            self.gamepad_shift_register_2 = !gamepad.to_byte();
                        }
                    }
                }
                self.latch = value & 0b111;
            }
            0x4017 => {
                unreachable!("IO: Write to 4017 should be handled by the Apu.");
            }
            _ => unreachable!("IO: Write to {address:04X}"),
        }
    }

    pub fn tick(&mut self, _cpu: &mut Cpu, peripherals: &mut Peripherals) {
        self.port_1 = peripherals.input.port_1.clone();
        self.port_2 = peripherals.input.port_2.clone();
        peripherals.input.read_version = peripherals
            .input
            .read_version
            .wrapping_add(self.read_version_increment);
        self.read_version_increment = 0;
    }
}
