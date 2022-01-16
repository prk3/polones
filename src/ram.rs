/// Random access memory with SIZE bytes of data. Uses SIZE-1 as address mask.
pub struct Ram<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize> Ram<SIZE> {
    pub fn new() -> Self {
        Self { data: [0; SIZE] }
    }
    pub fn read(&self, address: usize) -> u8 {
        self.data[address & (SIZE - 1)]
    }
    pub fn write(&mut self, address: usize, value: u8) {
        self.data[address & (SIZE - 1)] = value;
    }
}
