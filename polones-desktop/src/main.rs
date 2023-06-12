use apu_debugger::SdlApuDebugger;
use clap::Parser;
use cpu_debugger::SdlCpuDebugger;
use graphics_debugger::SdlGraphicsDebugger;
use memory_debugger::SdlMemoryDebugger;
use polones_core::game_file::GameFile;
use polones_core::nes::{Frame, GamepadState, Nes, PortState};
use ppu_debugger::SdlPpuDebugger;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use sdl_extensions::get_default_playback_device_name;
use std::collections::VecDeque;
use std::path::Component;

mod apu_debugger;
mod cpu_debugger;
mod graphics_debugger;
mod memory_debugger;
mod ppu_debugger;
mod sdl_extensions;
mod text_area;

struct SdlGameWindow {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: sdl2::render::TextureCreator<WindowContext>,
    texture: sdl2::render::Texture<'static>,
    gamepad_1: GamepadState,
    gamepad_2: GamepadState,
    frame: Box<Frame>,
    version: u32,
}

impl SdlGameWindow {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;

    fn new(canvas: sdl2::render::WindowCanvas) -> Self {
        let mut canvas = canvas;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let texture_creator = canvas.texture_creator();
        let mut data = [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4];
        let surface = Surface::from_data(
            &mut data[..],
            Self::WIDTH,
            Self::HEIGHT,
            Self::WIDTH * 4,
            PixelFormatEnum::RGB24,
        )
        .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        canvas.clear();
        Self {
            canvas,
            texture: unsafe { std::mem::transmute(texture) },
            _texture_creator: texture_creator,
            gamepad_1: GamepadState::default(),
            gamepad_2: GamepadState::default(),
            frame: Box::new([[(0, 0, 0); 256]; 240]),
            version: 0,
        }
    }

    fn handle_event(&mut self, _nes: &mut Nes, event: Event, state: &mut EmulatorState) {
        match event {
            Event::Quit { .. } => {
                state.exit = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Escape),
                ..
            } => {
                state.exit = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::W),
                ..
            } => {
                self.gamepad_1.up = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::S),
                ..
            } => {
                self.gamepad_1.down = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::A),
                ..
            } => {
                self.gamepad_1.left = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::D),
                ..
            } => {
                self.gamepad_1.right = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::R),
                ..
            } => {
                self.gamepad_1.select = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::T),
                ..
            } => {
                self.gamepad_1.start = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::F),
                ..
            } => {
                self.gamepad_1.b = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::G),
                ..
            } => {
                self.gamepad_1.a = true;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::W),
                ..
            } => {
                self.gamepad_1.up = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::S),
                ..
            } => {
                self.gamepad_1.down = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::A),
                ..
            } => {
                self.gamepad_1.left = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::D),
                ..
            } => {
                self.gamepad_1.right = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::R),
                ..
            } => {
                self.gamepad_1.select = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::T),
                ..
            } => {
                self.gamepad_1.start = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::F),
                ..
            } => {
                self.gamepad_1.b = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::G),
                ..
            } => {
                self.gamepad_1.a = false;
            }
            _ => {}
        }
    }

    fn draw_and_wait(&mut self, _nes: &mut Nes) {
        let mut data = [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4];
        for y in 0..Self::HEIGHT as usize {
            for x in 0..Self::WIDTH as usize {
                data[4 * (y * Self::WIDTH as usize + x) + 0] = self.frame[y][x].2;
                data[4 * (y * Self::WIDTH as usize + x) + 1] = self.frame[y][x].1;
                data[4 * (y * Self::WIDTH as usize + x) + 2] = self.frame[y][x].0;
            }
        }

        self.texture
            .update(
                Rect::new(0, 0, Self::WIDTH, Self::HEIGHT),
                &data,
                Self::WIDTH as usize * 4,
            )
            .unwrap();

        let display_size = self.canvas.window().size();

        let frame_ratio = Self::WIDTH as f32 / Self::HEIGHT as f32;
        let display_ratio = display_size.0 as f32 / display_size.1 as f32;

        let frame_rect = Rect::new(0, 0, Self::WIDTH, Self::HEIGHT);
        let scaled_frame_rect;

        if frame_ratio > display_ratio {
            let scale = display_size.0 as f32 / Self::WIDTH as f32;
            let scaled_frame_size = (display_size.0, (Self::HEIGHT as f32 * scale) as u32);
            let scaled_frame_pos = (0, (display_size.1 - scaled_frame_size.1) as i32 / 2);
            scaled_frame_rect = Rect::new(
                scaled_frame_pos.0,
                scaled_frame_pos.1,
                scaled_frame_size.0,
                scaled_frame_size.1,
            );
        } else {
            let scale = display_size.1 as f32 / Self::HEIGHT as f32;
            let scaled_frame_size = ((Self::WIDTH as f32 * scale) as u32, display_size.1);
            let scaled_frame_pos = ((display_size.0 - scaled_frame_size.0) as i32 / 2, 0);
            scaled_frame_rect = Rect::new(
                scaled_frame_pos.0,
                scaled_frame_pos.1,
                scaled_frame_size.0,
                scaled_frame_size.1,
            );
        }

        self.canvas.clear();
        self.canvas
            .copy(&mut self.texture, frame_rect, scaled_frame_rect)
            .unwrap();
        self.canvas.present();
    }
}

#[derive(Clone)]
pub struct EmulatorState {
    running: bool,
    exit: bool,
    one_step: bool,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    rom: String,

    #[arg(long)]
    cpu_debugger: bool,

    #[arg(long)]
    ppu_debugger: bool,

    #[arg(long)]
    apu_debugger: bool,

    #[arg(long)]
    graphics_debugger: bool,

    #[arg(long)]
    memory_debugger: bool,

    #[arg(long)]
    record_inputs: bool,
}

fn main() {
    let args = Args::parse();

    let rom_filename = {
        let rom_path = std::path::Path::new(&args.rom);
        match rom_path.components().last() {
            Some(Component::Normal(normal)) => normal.to_string_lossy().into_owned(),
            Some(_) => {
                eprintln!("Path does not end with normal");
                std::process::exit(1);
            }
            None => {
                eprintln!("Path is empty");
                std::process::exit(1);
            }
        }
    };

    let rom_data = match std::fs::read(&args.rom) {
        Ok(rom_data) => rom_data,
        Err(error) => {
            eprintln!("Could not read ROM: {error}");
            std::process::exit(1);
        }
    };

    let game_file = match GameFile::read(args.rom.clone(), rom_data) {
        Ok(game_file) => game_file,
        Err(_error) => {
            eprintln!("Could not parse ROM");
            std::process::exit(1);
        }
    };

    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let game_window = video_subsystem
        .window(
            "nes display",
            SdlGameWindow::WIDTH * 3,
            SdlGameWindow::HEIGHT * 3,
        )
        .position(0, 720)
        .build()
        .unwrap();
    let game_window_id = game_window.id();
    let game_canvas = game_window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .unwrap();

    let mut cpu_debugger = args.cpu_debugger.then(|| {
        let cpu_debugger_window = video_subsystem
            .window(
                "nes cpu debugger",
                SdlCpuDebugger::WIDTH * 3,
                SdlCpuDebugger::HEIGHT * 3,
            )
            .position(768, 0)
            .build()
            .unwrap();
        let cpu_debugger_window_id = cpu_debugger_window.id();
        let cpu_debugger_canvas = cpu_debugger_window
            .into_canvas()
            .accelerated()
            .build()
            .unwrap();
        (
            cpu_debugger_window_id,
            SdlCpuDebugger::new(cpu_debugger_canvas),
        )
    });

    let mut ppu_debugger = args.ppu_debugger.then(|| {
        let ppu_debugger_window = video_subsystem
            .window(
                "nes ppu debugger",
                SdlPpuDebugger::WIDTH * 3,
                SdlPpuDebugger::HEIGHT * 3,
            )
            .position(768, 720)
            .build()
            .unwrap();
        let ppu_debugger_window_id = ppu_debugger_window.id();
        let ppu_debugger_canvas = ppu_debugger_window
            .into_canvas()
            .accelerated()
            .build()
            .unwrap();
        (
            ppu_debugger_window_id,
            SdlPpuDebugger::new(ppu_debugger_canvas),
        )
    });

    let mut apu_debugger = args.apu_debugger.then(|| {
        let apu_debugger_window = video_subsystem
            .window(
                "nes apu debugger",
                SdlPpuDebugger::WIDTH * 3,
                SdlPpuDebugger::HEIGHT * 3,
            )
            .position(768, 720)
            .build()
            .unwrap();
        let apu_debugger_window_id = apu_debugger_window.id();
        let apu_debugger_canvas = apu_debugger_window
            .into_canvas()
            .accelerated()
            .build()
            .unwrap();
        (
            apu_debugger_window_id,
            SdlApuDebugger::new(apu_debugger_canvas),
        )
    });

    let mut memory_debugger = args.memory_debugger.then(|| {
        let memory_debugger_window = video_subsystem
            .window(
                "nes memory debugger",
                SdlMemoryDebugger::WIDTH * 2,
                SdlMemoryDebugger::HEIGHT * 2,
            )
            .position(0, 1)
            .build()
            .unwrap();
        let memory_debugger_window_id = memory_debugger_window.id();
        let memory_debugger_canvas = memory_debugger_window
            .into_canvas()
            .accelerated()
            .build()
            .unwrap();
        (
            memory_debugger_window_id,
            SdlMemoryDebugger::new(memory_debugger_canvas),
        )
    });

    let mut graphics_debugger = args.graphics_debugger.then(|| {
        let graphics_debugger_window = video_subsystem
            .window(
                "nes graphics debugger",
                SdlGraphicsDebugger::WIDTH * 2,
                SdlGraphicsDebugger::HEIGHT * 2,
            )
            .position(0, 1)
            .build()
            .unwrap();
        let graphics_debugger_window_id = graphics_debugger_window.id();
        let graphics_debugger_window_canvas = graphics_debugger_window
            .into_canvas()
            .accelerated()
            .build()
            .unwrap();
        (
            graphics_debugger_window_id,
            SdlGraphicsDebugger::new(graphics_debugger_window_canvas),
        )
    });

    let mut game_window = SdlGameWindow::new(game_canvas);
    let mut nes = Nes::new(game_file).expect("Could not start the game");

    // On every write to $4016 we push port 1 and port 2 state to this vec.
    let mut inputs: Vec<u8> = Vec::new();
    let mut inputs_version = 0;

    let mut state = EmulatorState {
        running: true,
        exit: false,
        one_step: false,
    };

    if let Some((_id, debugger)) = &mut cpu_debugger {
        debugger.update_disassembly(&mut nes);
        debugger.draw(&mut nes);
    }
    if let Some((_id, debugger)) = &mut ppu_debugger {
        debugger.draw(&mut nes);
    }
    if let Some((_id, debugger)) = &mut apu_debugger {
        debugger.draw(&mut nes);
    }
    if let Some((_id, debugger)) = &mut memory_debugger {
        debugger.draw(&mut nes);
    }
    if let Some((_id, debugger)) = &mut graphics_debugger {
        debugger.draw(&mut nes);
    }

    game_window.draw_and_wait(&mut nes);

    let (audio_sender, audio_receiver) = std::sync::mpsc::channel();

    let refresh_rate = game_window
        .canvas
        .window()
        .display_mode()
        .map(|mode| mode.refresh_rate)
        .unwrap_or(60);

    // CPU cycles per frame = 29780.5*
    let audio_samples_per_draw = 59561 * 30 / refresh_rate;

    let default_playback_device_name = get_default_playback_device_name();

    let audio_playback: AudioDevice<AudioRunner> = audio_subsystem
        .open_playback(
            default_playback_device_name.as_deref(),
            &AudioSpecDesired {
                freq: Some(44100),
                channels: Some(1),
                samples: Some(256),
            },
            |_| AudioRunner {
                source: audio_receiver,
                samples: VecDeque::with_capacity(audio_samples_per_draw as usize * 5),
            },
        )
        .unwrap();
    let mut audio_version = 0;

    // push two frame's worth of silence to the audio runner
    let _ = audio_sender.send(
        std::iter::repeat(0)
            .take(audio_samples_per_draw as usize * 2)
            .collect(),
    );
    audio_playback.resume();

    'ui_loop: loop {
        let old_state = state.clone();

        // handle window events
        for event in event_pump.poll_iter() {
            let event_id = event.get_window_id();
            if event.get_window_id() == Some(game_window_id) {
                game_window.handle_event(&mut nes, event, &mut state);
                continue;
            }
            match &mut cpu_debugger {
                Some((id, debugger)) if event_id == Some(*id) => {
                    debugger.handle_event(&mut nes, event, &mut state);
                    continue;
                }
                _ => {}
            }
            match &mut ppu_debugger {
                Some((id, debugger)) if event_id == Some(*id) => {
                    debugger.handle_event(&mut nes, event, &mut state);
                    continue;
                }
                _ => {}
            }
            match &mut apu_debugger {
                Some((id, debugger)) if event_id == Some(*id) => {
                    debugger.handle_event(&mut nes, event, &mut state);
                    continue;
                }
                _ => {}
            }
            match &mut memory_debugger {
                Some((id, debugger)) if event_id == Some(*id) => {
                    debugger.handle_event(&mut nes, event, &mut state);
                    continue;
                }
                _ => {}
            }
            match &mut graphics_debugger {
                Some((id, debugger)) if event_id == Some(*id) => {
                    debugger.handle_event(&mut nes, event, &mut state);
                    continue;
                }
                _ => {}
            }
        }

        // exit if UI thread requested exit
        if state.exit {
            audio_playback.pause();
            break 'ui_loop;
        }

        let refresh_rate = game_window
            .canvas
            .window()
            .display_mode()
            .map(|mode| mode.refresh_rate)
            .unwrap_or(60);

        // CPU cycles per frame = 29780.5*
        let audio_samples_per_draw = 59561 * 30 / refresh_rate;

        if args.record_inputs {
            while inputs_version != nes.input.read_version {
                inputs.push(if let PortState::Gamepad(g) = &nes.input.port_1 {
                    g.to_byte()
                } else {
                    panic!()
                });
                inputs.push(if let PortState::Gamepad(g) = &nes.input.port_2 {
                    g.to_byte()
                } else {
                    panic!()
                });
                inputs_version = inputs_version.wrapping_add(1);
            }
        }

        // update nes controls
        nes.input.port_1 = PortState::Gamepad(game_window.gamepad_1.clone());
        nes.input.port_2 = PortState::Gamepad(game_window.gamepad_2.clone());

        // handle one instruction step request
        if state.one_step {
            nes.run_one_cpu_instruction();
            if let Some((_id, debugger)) = &mut cpu_debugger {
                debugger.update_disassembly(&mut nes);
            }
            nes.apu.clear_samples();
            audio_version = nes.audio.version;
            state.one_step = false;
        } else if state.running {
            if !old_state.running {
                // emulation was just resumed
                // send empty vec to tell the audio runner to remove old samples
                let _ = audio_sender.send(Vec::new());
                // push two frame's worth of silence to the audio runner
                let _ = audio_sender.send(
                    std::iter::repeat(0)
                        .take(audio_samples_per_draw as usize * 2)
                        .collect(),
                );
                audio_playback.resume();
            }

            if let Some((_id, debugger)) = &mut cpu_debugger {
                for _ in 0..audio_samples_per_draw {
                    nes.run_one_cpu_tick();
                    debugger.update_disassembly(&mut nes);

                    if audio_version != nes.audio.version {
                        audio_version = nes.audio.version;
                        let _ = audio_sender.send(std::mem::take(&mut nes.audio.samples));
                    }

                    if debugger.breakpoints.contains(&nes.cpu.program_counter) {
                        while !nes.cpu.finished_instruction() {
                            nes.run_one_cpu_tick();
                            debugger.update_disassembly(&mut nes);
                        }
                        nes.apu.clear_samples();
                        audio_version = nes.audio.version;
                        state.running = false;
                        break;
                    }
                }
            } else {
                for _ in 0..audio_samples_per_draw {
                    nes.run_one_cpu_tick();

                    if audio_version != nes.audio.version {
                        audio_version = nes.audio.version;
                        let _ = audio_sender.send(std::mem::take(&mut nes.audio.samples));
                    }
                }
            }
        } else {
            // state.running == false
            if old_state.running {
                // emulation was just paused
                audio_playback.pause();
            }
        }

        if game_window.version != nes.display.version {
            game_window.version = nes.display.version;
            std::mem::swap(&mut game_window.frame, &mut nes.display.frame);
        }

        // draw new game and debugger content
        if let Some((_id, debugger)) = &mut cpu_debugger {
            debugger.draw(&mut nes);
        }
        if let Some((_id, debugger)) = &mut ppu_debugger {
            debugger.draw(&mut nes);
        }
        if let Some((_id, debugger)) = &mut apu_debugger {
            debugger.draw(&mut nes);
        }
        if let Some((_id, debugger)) = &mut memory_debugger {
            debugger.draw(&mut nes);
        }
        if let Some((_id, debugger)) = &mut graphics_debugger {
            debugger.draw(&mut nes);
        }
        game_window.draw_and_wait(&mut nes);
    }

    if args.record_inputs {
        std::fs::write(format!("../inputs/{rom_filename}.bin"), inputs).unwrap_or_else(|e| {
            eprintln!("Could not save inputs: {e}");
            std::process::exit(1);
        })
    }
}

struct AudioRunner {
    source: std::sync::mpsc::Receiver<Vec<u16>>,
    samples: VecDeque<u16>,
}

impl AudioCallback for AudioRunner {
    type Channel = u16;
    fn callback(&mut self, buffer: &mut [Self::Channel]) {
        while let Ok(samples) = self.source.try_recv() {
            if samples.is_empty() {
                self.samples.clear();
            } else {
                self.samples.extend(samples);
            }
        }

        let samples_standard = 1789772 * 256 / 44100;
        let samples_to_use = std::cmp::min(self.samples.len(), samples_standard);

        if !self.samples.is_empty() {
            for i in 0..256 {
                buffer[i] = self.samples[i * samples_to_use / 256];
            }
            self.samples.drain(0..samples_to_use);
        } else {
            for i in 0..256 {
                buffer[i] = 0;
            }
        }
    }
}

// fn write_wave(path: &str, samples: &[u16]) -> std::io::Result<()> {
//     use std::io::Write;
//
//     let file = std::fs::File::create(path)?;
//     let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, file);
//
//     writer.write(b"RIFF")?;
//     writer.write(&(samples.len() as u32 * 2 + 36).to_le_bytes())?;
//     writer.write(b"WAVE")?;
//     writer.write(b"fmt ")?;
//     writer.write(&16u32.to_le_bytes())?;
//     writer.write(&1u16.to_le_bytes())?;
//     writer.write(&1u16.to_le_bytes())?;
//     writer.write(&44_100u32.to_le_bytes())?;
//     writer.write(&(44_100u32 * 2 * 1 / 8).to_le_bytes())?;
//     writer.write(&(2u16).to_le_bytes())?;
//     writer.write(&(16u16).to_le_bytes())?;
//     writer.write(b"data")?;
//     writer.write(&(samples.len() as u32 * 1 * 16 / 8).to_le_bytes())?;
//
//     for s in samples.iter() {
//         writer.write(&s.to_le_bytes())?;
//     }
//     Ok(())
// }
