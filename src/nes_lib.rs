pub mod bus;
pub mod cpu;
pub mod ram;

pub type Frame = [[(u8, u8, u8); 256]; 240];

pub trait Display {
    fn display(&mut self, frame: Box<Frame>);
}

pub trait CpuDebugDisplay {
    fn display<B: bus::Bus>(&mut self, cpu: &cpu::Cpu<B>);
}
