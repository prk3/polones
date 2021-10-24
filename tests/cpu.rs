use nes_lib::bus::Bus;
use nes_lib::cpu::Cpu;
use std::cell::RefCell;

struct CpuTestBus {
    data: RefCell<[u8; 65536]>,
    debug_range: (u16, u16),
}

impl CpuTestBus {
    fn with_program(program: &[u8], debug_range: (u16, u16)) -> Self {
        let bus = Self {
            data: RefCell::new([0; 65536]),
            debug_range,
        };
        let mut data = bus.data.borrow_mut();
        for (i, byte) in program.iter().enumerate().take(data.len()) {
            data[i] = *byte;
        }
        drop(data);
        bus
    }
}

impl Bus for CpuTestBus {
    fn read(&self, address: u16) -> u8 {
        self.data.borrow()[address as usize]
    }
    fn write(&self, address: u16, value: u8) {
        self.data.borrow_mut()[address as usize] = value;
    }
}

impl std::fmt::Debug for CpuTestBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let mut out = format!("@{:04X}:{:04X}", self.debug_range.0, self.debug_range.1);
        let data = self.data.borrow();
        let data_in_range = &data[(self.debug_range.0 as usize)..=(self.debug_range.1 as usize)];
        for byte in data_in_range {
            write!(out, " {:02X}", byte).unwrap();
        }
        f.write_str(&out)
    }
}

#[test]
fn math_ops() {
    let mut cpu = Cpu::<CpuTestBus>::new();
    let bus = CpuTestBus::with_program(include_bytes!("programs/math_ops.6502.nes"), (200, 205));
    cpu.reset(&bus);

    for _ in 0..20 {
        println!("CPU=({:?}) RAM=({:?})", &cpu, &bus);
        cpu.run_one_instruction(&bus);
    }
}
