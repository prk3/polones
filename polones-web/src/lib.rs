mod utils;

use polones_core::game_file::GameFile;
use polones_core::nes::{Display, Frame, Input, Nes, PortState};

use wasm_bindgen::{prelude::*, Clamped};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut NES: Option<Nes> = None;

#[wasm_bindgen]
extern "C" {
    fn polones_display_draw(frame: Clamped<Vec<u8>>);
}

#[wasm_bindgen]
pub fn polones_start(rom: Vec<u8>) -> Option<String> {

    let game = match GameFile::read("rom".into(), rom) {
        Ok(game) => game,
        Err(err) => return Some("could not read game".into()),
    };
    let display = CanvasDisplay {};
    let input = WebInput {};
    let nes = match Nes::new(game, display, input) {
        Ok(nes) => nes,
        Err(err) => return Some("Could not start NES".into()),
    };
    unsafe {
        NES = Some(nes);
    }
    None
}

#[wasm_bindgen]
pub fn polones_tick() -> Option<String> {
    if let Some(nes) = unsafe { &mut NES } {
        nes.run_one_cpu_tick();
        None
    } else {
        Some("NES not initialized".into())
    }
}

struct CanvasDisplay {}

impl Display for CanvasDisplay {
    fn draw(&mut self, frame: Box<Frame>) {
        let mut output = Vec::with_capacity(256 * 240 * 4);
        for row in frame.into_iter() {
            for pixel in row {
                output.push(pixel.0);
                output.push(pixel.1);
                output.push(pixel.2);
                output.push(255);
            }
        }
        polones_display_draw(Clamped(output));
    }
}

struct WebInput {}

impl Input for WebInput {
    fn read_port_1(&mut self) -> polones_core::nes::PortState {
        PortState::Unplugged
    }

    fn read_port_2(&mut self) -> polones_core::nes::PortState {
        PortState::Unplugged
    }
}
