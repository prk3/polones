/// Random access memory with SIZE bytes of data. Uses SIZE-1 as address mask.
pub struct Ram<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize> Ram<SIZE> {
    pub fn new() -> Self {
        Self { data: [0; SIZE] }
    }
    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize & (SIZE - 1)]
    }
    pub fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize & (SIZE - 1)] = value;
    }
}
