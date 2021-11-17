use nes_lib::cpu_bus::CpuBus;
use nes_lib::cpu::Cpu;
use std::cell::RefCell;

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
