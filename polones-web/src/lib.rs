mod utils;
use utils::set_panic_hook;
use wasm_bindgen::{prelude::*, Clamped};

use polones_core::game_file::GameFile;
use polones_core::nes::{Display, Frame, Input, Nes, PortState};
use serde::Deserialize;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut NES: Option<Nes> = None;

#[wasm_bindgen]
extern "C" {
    fn polones_display_draw(frame: Clamped<Vec<u8>>);
    fn polones_input_read_port_1() -> String;
    fn polones_input_read_port_2() -> String;
}

#[wasm_bindgen]
pub fn polones_start(rom: Vec<u8>) -> Option<String> {
    set_panic_hook();

    let game = match GameFile::read("rom".into(), rom) {
        Ok(game) => game,
        Err(_err) => return Some("could not read game".into()),
    };
    let display = CanvasDisplay {};
    let input = WebInput {};
    let nes = match Nes::new(game, display, input) {
        Ok(nes) => nes,
        Err(_err) => return Some("Could not start NES".into()),
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
        for row in frame.iter() {
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

#[derive(Deserialize)]
#[serde(tag = "type")]
enum PortStateExternal {
    #[serde(rename = "unplugged")]
    Unplugged {},
    #[serde(rename = "gamepad")]
    Gamepad {
        a: bool,
        b: bool,
        select: bool,
        start: bool,
        up: bool,
        down: bool,
        left: bool,
        right: bool,
    },
}

fn port_state_external_string_to_port_state(string: String) -> PortState {
    let port_state_external = serde_json::from_str(&string).unwrap();
    match port_state_external {
        PortStateExternal::Unplugged { .. } => PortState::Unplugged,
        PortStateExternal::Gamepad {
            a,
            b,
            select,
            start,
            up,
            down,
            left,
            right,
        } => PortState::Gamepad {
            a,
            b,
            select,
            start,
            up,
            down,
            left,
            right,
        },
    }
}

impl Input for WebInput {
    fn read_port_1(&mut self) -> PortState {
        let port_state_external_string = polones_input_read_port_1();
        port_state_external_string_to_port_state(port_state_external_string)
    }

    fn read_port_2(&mut self) -> PortState {
        let port_state_external_string = polones_input_read_port_2();
        port_state_external_string_to_port_state(port_state_external_string)
    }
}
