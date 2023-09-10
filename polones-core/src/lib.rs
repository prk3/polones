/// Macro running a block of code in a nested, named stack frame.
/// Useful for distinguishing parts of a function on flamegraphs.
#[allow(unused_macros)]
macro_rules! name_block {
    ($name:ident, $block:block) => {{
        let b = || $block;
        fn $name(b: impl FnOnce()) {
            b();
        }
        $name(b);
    }};
}

pub mod apu;
pub mod cpu;
pub mod game_file;
pub mod io;
pub mod mapper;
pub mod nes;
pub mod ppu;
pub mod ram;
