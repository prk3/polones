use std::cell::RefCell;

use crate::cpu::Cpu;
use crate::ram::Ram;

pub trait Bus {
    fn read(&self, address: u16) -> u8;
    fn write(&self, address: u16, value: u8);
}

pub struct MainBus {
    cpu: RefCell<Cpu<Self>>,
    ram: RefCell<Ram>,
}

impl Bus for MainBus {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0xFFFF => {
                self.ram.borrow_mut().read(address)
            }
        }
    }
    fn write(&self, address: u16, value: u8) {
        match address {
            0x0000..=0xFFFF => {
                self.ram.borrow_mut().write(address, value)
            }
        }
    }
}

impl MainBus {
    pub fn tick(&self) {
        self.cpu.borrow_mut().tick(&self)
    }
}
