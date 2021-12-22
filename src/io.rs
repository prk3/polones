use crate::nes::Nes;

pub struct Io {
    latch: u8,
    shift_register_1: u8,
    shift_register_2: u8,
}

impl Io {
    pub fn new() -> Self {
        Self {
            latch: 0,
            shift_register_1: 0,
            shift_register_2: 0,
        }
    }

    pub fn read(&mut self, nes: &Nes, address: u16) -> u8 {
        match 0x4016 + (address & 1) {
            0x4016 => {
                let result = self.shift_register_1 & 1;
                self.shift_register_1 >>= 1;
                result
            }
            0x4017 => {
                let result = self.shift_register_2 & 1;
                self.shift_register_2 >>= 1;
                result
            }
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, nes: &Nes, address: u16, value: u8) {
        match 0x4016 + (address & 1) {
            0x4016 => {
                self.latch = value & 0b111;
                if self.latch & 1 == 1 {
                    let mut input = nes.input.borrow_mut();
                    self.shift_register_1 = input.read_pad_1().unwrap_or(0);
                    self.shift_register_2 = input.read_pad_2().unwrap_or(0);
                }
            }
            0x4017 => {
                eprintln!("Write to 4017 ignored.");
            }
            _ => unreachable!(),
        }
    }
}
