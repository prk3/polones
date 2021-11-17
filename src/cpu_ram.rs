pub struct CpuRam {
    data: [u8; 0x0800], // 2KiB
}

impl CpuRam {
    pub fn new() -> Self {
        Self { data: [0; 0x0800] }
    }
}

impl CpuRam {
    pub fn read(&mut self, address: u16) -> u8 {
        self.data[address as usize & 0x07FF]
    }
    pub fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize & 0x07FF] = value;
    }
}
