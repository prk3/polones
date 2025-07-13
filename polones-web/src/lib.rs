mod utils;

use serde::Deserialize;
use wasm_bindgen::prelude::*;

use polones_core::game_file::GameFile;
use polones_core::nes::{GamepadState, Nes, PortState};
use utils::set_panic_hook;

static mut STATE: Option<State> = None;

struct State {
    nes: Nes,
    video_version: u32,
    audio_version: u32,
}

#[wasm_bindgen]
pub fn polones_init(rom: Vec<u8>) -> Result<(), String> {
    set_panic_hook();

    let game = match GameFile::read("rom".into(), rom) {
        Ok(game) => game,
        Err(_err) => return Err("could not read game".into()),
    };
    let nes = match Nes::new(game) {
        Ok(nes) => nes,
        Err(_err) => return Err("Could not start NES".into()),
    };
    unsafe {
        STATE = Some(State {
            nes,
            video_version: 0,
            audio_version: 0,
        });
    }
    Ok(())
}

#[wasm_bindgen]
pub fn polones_tick(times: u32) -> Result<(), String> {
    if let Some(state) = unsafe { &mut STATE } {
        for _ in 0..times {
            state.nes.run_one_cpu_tick();
        }
        Ok(())
    } else {
        Err("NES not initialized".into())
    }
}

#[wasm_bindgen]
pub fn polones_get_video_frame() -> Result<Option<Vec<u8>>, String> {
    if let Some(state) = unsafe { &mut STATE } {
        if state.video_version != state.nes.display.version {
            state.video_version = state.nes.display.version;
            let mut output = Vec::with_capacity(256 * 240 * 4);
            for row in state.nes.display.frame.iter() {
                for pixel in row {
                    output.push(pixel.0);
                    output.push(pixel.1);
                    output.push(pixel.2);
                    output.push(255);
                }
            }
            Ok(Some(output))
        } else {
            Ok(None)
        }
    } else {
        Err("NES not initialized".into())
    }
}

#[wasm_bindgen]
pub fn polones_get_audio_samples() -> Result<Option<Vec<u16>>, String> {
    if let Some(state) = unsafe { &mut STATE } {
        if state.audio_version != state.nes.audio.version {
            state.audio_version = state.nes.audio.version;
            Ok(Some(std::mem::take(&mut state.nes.audio.samples)))
        } else {
            Ok(None)
        }
    } else {
        Err("NES not initialized".into())
    }
}

#[wasm_bindgen]
pub fn polones_set_input(port_1: String, port_2: String) -> Result<(), String> {
    if let Some(state) = unsafe { &mut STATE } {
        state.nes.input.port_1 = port_state_external_string_to_port_state(port_1);
        state.nes.input.port_2 = port_state_external_string_to_port_state(port_2);
        Ok(())
    } else {
        Err("NES not initialized".into())
    }
}

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
        } => PortState::Gamepad(GamepadState {
            a,
            b,
            select,
            start,
            up,
            down,
            left,
            right,
        }),
    }
}
