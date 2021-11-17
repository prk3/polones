pub struct PpuRam {
    data: [u8; 0x0800], // 2KiB
}

impl PpuRam {
    pub fn new() -> Self {
        Self { data: [0; 0x0800] }
    }
}

impl PpuRam {
    pub fn read(&self, address: u16) -> u8 {
        self.data[(address as usize - 0x2000) & 0x07FF]
    }
    pub fn write(&mut self, address: u16, value: u8) {
        self.data[(address as usize - 0x2000) & 0x07FF] = value;
    }
}
