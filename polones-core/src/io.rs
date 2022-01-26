use crate::nes::{Input, PortState};

pub struct Io {
    latch: u8,
    port_1_state: PortState,
    port_2_state: PortState,
    gamepad_shift_register_1: u8,
    gamepad_shift_register_2: u8,
    input: Box<dyn Input>,
}

impl Io {
    pub fn new(input: Box<dyn Input>) -> Self {
        Self {
            latch: 0,
            port_1_state: PortState::Unplugged,
            port_2_state: PortState::Unplugged,
            gamepad_shift_register_1: 0,
            gamepad_shift_register_2: 0,
            input,
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match 0x4016 + (address & 1) {
            0x4016 => match self.port_1_state {
                PortState::Unplugged => 0,
                PortState::Gamepad { .. } => {
                    let result = (self.gamepad_shift_register_1 & 0b10000000) >> 7;
                    self.gamepad_shift_register_1 <<= 1;
                    result
                }
            },
            0x4017 => match self.port_1_state {
                PortState::Unplugged => 0,
                PortState::Gamepad { .. } => {
                    let result = (self.gamepad_shift_register_2 & 0b10000000) >> 7;
                    self.gamepad_shift_register_2 <<= 1;
                    result
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match 0x4016 + (address & 1) {
            0x4016 => {
                self.port_1_state = self.input.read_port_1();
                self.port_2_state = self.input.read_port_2();
                match self.port_1_state {
                    PortState::Unplugged => {}
                    PortState::Gamepad {
                        up,
                        down,
                        left,
                        right,
                        select,
                        start,
                        a,
                        b,
                    } => {
                        if self.latch & 1 == 1 && value & 1 == 0 {
                            self.gamepad_shift_register_1 = (a as u8) << 7
                                | (b as u8) << 6
                                | (select as u8) << 5
                                | (start as u8) << 4
                                | (up as u8) << 3
                                | (down as u8) << 2
                                | (left as u8) << 1
                                | (right as u8) << 0;
                        }
                    }
                }
                match self.port_2_state {
                    PortState::Unplugged => {}
                    PortState::Gamepad {
                        up,
                        down,
                        left,
                        right,
                        select,
                        start,
                        a,
                        b,
                    } => {
                        if self.latch & 1 == 1 && value & 1 == 0 {
                            self.gamepad_shift_register_2 = (a as u8) << 7
                                | (b as u8) << 6
                                | (select as u8) << 5
                                | (start as u8) << 4
                                | (up as u8) << 3
                                | (down as u8) << 2
                                | (left as u8) << 1
                                | (right as u8) << 0;
                        }
                    }
                }
                self.latch = value & 0b111;
            }
            0x4017 => {
                eprintln!("IO: Write to 4017 ignored.");
            }
            _ => unreachable!(),
        }
    }
}
