
pub struct Ram {
    data: [u8; 65536],
}

impl Ram {
    pub fn read(&mut self, address: u16) -> u8{
        self.data.get(address as usize).cloned().unwrap_or_else(|| {
            eprintln!("Out of bound memory read: {:#04x}", address);
            0
        })
    }
    pub fn write(&mut self, address: u16, value: u8) {
        if (address as usize) < self.data.len() {
            self.data[address as usize] = value;
        } else {
            eprintln!("Out of bounds memory write: {:#04x} {:#02x}", address, value);
        }
    }
}
