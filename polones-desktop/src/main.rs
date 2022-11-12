use apu_debugger::SdlApuDebugger;
use clap::Parser;
use cpu_debugger::SdlCpuDebugger;
use graphics_debugger::SdlGraphicsDebugger;
use memory_debugger::SdlMemoryDebugger;
use polones_core::game_file::GameFile;
use polones_core::nes::{Audio, Display, Frame, Input, Nes, PortState};
use ppu_debugger::SdlPpuDebugger;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::time::Duration;

mod apu_debugger;
mod cpu_debugger;
mod graphics_debugger;
mod memory_debugger;
mod ppu_debugger;
mod text_area;

struct SdlGameWindow {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: sdl2::render::TextureCreator<WindowContext>,
    texture: sdl2::render::Texture<'static>,
    gamepad_1_up: bool,
    gamepad_1_down: bool,
    gamepad_1_left: bool,
    gamepad_1_right: bool,
    gamepad_1_select: bool,
    gamepad_1_start: bool,
    gamepad_1_a: bool,
    gamepad_1_b: bool,
    gamepad_2_up: bool,
    gamepad_2_down: bool,
    gamepad_2_left: bool,
    gamepad_2_right: bool,
    gamepad_2_select: bool,
    gamepad_2_start: bool,
    gamepad_2_a: bool,
    gamepad_2_b: bool,
    frame: Box<Frame>,
    frame_generation_time: std::time::Instant,
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
            gamepad_1_up: false,
            gamepad_1_down: false,
            gamepad_1_left: false,
            gamepad_1_right: false,
            gamepad_1_select: false,
            gamepad_1_start: false,
            gamepad_1_a: false,
            gamepad_1_b: false,
            gamepad_2_up: false,
            gamepad_2_down: false,
            gamepad_2_left: false,
            gamepad_2_right: false,
            gamepad_2_select: false,
            gamepad_2_start: false,
            gamepad_2_a: false,
            gamepad_2_b: false,
            frame: Box::new([[(0, 0, 0); 256]; 240]),
            frame_generation_time: std::time::Instant::now(),
        }
    }

    fn handle_event(&mut self, event: Event, _nes: &mut Nes, state: &mut EmulatorState) {
        match event {
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Escape),
                ..
            } => {
                state.exit = true;
            }
            Event::Quit { .. } => {
                state.exit = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::W),
                ..
            } => {
                self.gamepad_1_up = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::S),
                ..
            } => {
                self.gamepad_1_down = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::A),
                ..
            } => {
                self.gamepad_1_left = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::D),
                ..
            } => {
                self.gamepad_1_right = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::R),
                ..
            } => {
                self.gamepad_1_select = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::T),
                ..
            } => {
                self.gamepad_1_start = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::F),
                ..
            } => {
                self.gamepad_1_b = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::G),
                ..
            } => {
                self.gamepad_1_a = true;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::W),
                ..
            } => {
                self.gamepad_1_up = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::S),
                ..
            } => {
                self.gamepad_1_down = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::A),
                ..
            } => {
                self.gamepad_1_left = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::D),
                ..
            } => {
                self.gamepad_1_right = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::R),
                ..
            } => {
                self.gamepad_1_select = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::T),
                ..
            } => {
                self.gamepad_1_start = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::F),
                ..
            } => {
                self.gamepad_1_b = false;
            }
            Event::KeyUp {
                keycode: _k @ Some(Keycode::G),
                ..
            } => {
                self.gamepad_1_a = false;
            }
            _ => {}
        }
    }

    fn show(&mut self) {
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

struct SdlDisplay {
    inner: Rc<RefCell<SdlGameWindow>>,
}

impl SdlDisplay {
    fn new(inner: Rc<RefCell<SdlGameWindow>>) -> Self {
        Self { inner }
    }
}

impl Display for SdlDisplay {
    fn draw(&mut self, frame: Box<Frame>) {
        let mut inner = self.inner.borrow_mut();
        inner.frame = frame;
        inner.frame_generation_time = std::time::Instant::now();
    }
}

struct SdlInput {
    inner: Rc<RefCell<SdlGameWindow>>,
    record_inputs_writer: Option<std::io::BufWriter<std::fs::File>>,
}

impl SdlInput {
    pub fn new(inner: Rc<RefCell<SdlGameWindow>>, record_inputs_file: Option<&str>) -> Self {
        Self {
            inner,
            record_inputs_writer: record_inputs_file.map(|file| {
                std::io::BufWriter::with_capacity(
                    1024 * 16,
                    std::fs::File::create(file).expect("Could not open input file for writing"),
                )
            }),
        }
    }
}

impl Input for SdlInput {
    fn read_port_1(&mut self) -> PortState {
        let inner = self.inner.borrow_mut();
        if let Some(writer) = &mut self.record_inputs_writer {
            writer
                .write(&[(inner.gamepad_1_a as u8) << 7
                    | (inner.gamepad_1_b as u8) << 6
                    | (inner.gamepad_1_select as u8) << 5
                    | (inner.gamepad_1_start as u8) << 4
                    | (inner.gamepad_1_up as u8) << 3
                    | (inner.gamepad_1_down as u8) << 2
                    | (inner.gamepad_1_left as u8) << 1
                    | (inner.gamepad_1_right as u8)])
                .expect("Could not write input from port 1");
        }
        PortState::Gamepad {
            up: inner.gamepad_1_up,
            down: inner.gamepad_1_down,
            left: inner.gamepad_1_left,
            right: inner.gamepad_1_right,
            select: inner.gamepad_1_select,
            start: inner.gamepad_1_start,
            a: inner.gamepad_1_a,
            b: inner.gamepad_1_b,
        }
    }
    fn read_port_2(&mut self) -> PortState {
        let inner = self.inner.borrow_mut();
        if let Some(writer) = &mut self.record_inputs_writer {
            writer
                .write(&[(inner.gamepad_2_a as u8) << 7
                    | (inner.gamepad_2_b as u8) << 6
                    | (inner.gamepad_2_select as u8) << 5
                    | (inner.gamepad_2_start as u8) << 4
                    | (inner.gamepad_2_up as u8) << 3
                    | (inner.gamepad_2_down as u8) << 2
                    | (inner.gamepad_2_left as u8) << 1
                    | (inner.gamepad_2_right as u8)])
                .expect("Could not write input from port 2");
        }
        PortState::Gamepad {
            up: inner.gamepad_2_up,
            down: inner.gamepad_2_down,
            left: inner.gamepad_2_left,
            right: inner.gamepad_2_right,
            select: inner.gamepad_2_select,
            start: inner.gamepad_2_start,
            a: inner.gamepad_2_a,
            b: inner.gamepad_2_b,
        }
    }
}

pub struct EmulatorState {
    running: bool,
    exit: bool,
    one_step: bool,
}

struct DummyAudio {
    audio_tx: std::sync::mpsc::Sender<Vec<u16>>,
    record_samples: Vec<u16>,
    record: bool,
}

impl Audio for DummyAudio {
    fn play(&mut self, samples: &[u16; 64]) {
        let _ = self.audio_tx.send(samples.to_vec());

        // 1 minute of audio = 0.66MiB
        if self.record && self.record_samples.len() < 60 * 44_100 {
            self.record_samples.extend(&samples[..]);
        }
    }
}

#[derive(Parser, Debug)]
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
    record_inputs_file: Option<String>,
}

fn main() {
    let args = Args::parse();

    let file_contents = std::fs::read(&args.rom).expect("could not read the file");
    let game_file =
        GameFile::read(args.rom.clone(), file_contents).expect("file does not contain a nes game");

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
    let game_canvas = game_window.into_canvas().build().unwrap();

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
        let cpu_debugger_canvas = cpu_debugger_window.into_canvas().build().unwrap();
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
        let ppu_debugger_canvas = ppu_debugger_window.into_canvas().build().unwrap();
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
        let apu_debugger_canvas = apu_debugger_window.into_canvas().build().unwrap();
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
        let memory_debugger_canvas = memory_debugger_window.into_canvas().build().unwrap();
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
        let graphics_debugger_window_canvas =
            graphics_debugger_window.into_canvas().build().unwrap();
        (
            graphics_debugger_window_id,
            SdlGraphicsDebugger::new(graphics_debugger_window_canvas),
        )
    });

    let (audio_tx, audio_rx) = std::sync::mpsc::channel();
    // There is no easy way of removing audio_rx from one AudioRunner
    // and adding it to another. So instead we share audio_rx with an
    // Arc and Mutex. This is probably fine as there should only be
    // one AudioRunner at one time.
    let audio_rx = std::sync::Arc::new(std::sync::Mutex::new(audio_rx));
    let (notification_tx, notification_rx) = std::sync::mpsc::channel();

    let audio = DummyAudio {
        audio_tx,
        record_samples: Vec::new(),
        record: false,
    };

    let game_window = Rc::new(RefCell::new(SdlGameWindow::new(game_canvas)));
    let display = SdlDisplay::new(game_window.clone());
    let input = SdlInput::new(game_window.clone(), args.record_inputs_file.as_deref());
    let mut nes = Nes::new(game_file, display, input, audio).expect("Could not start the game");

    game_window.borrow_mut().show();

    if let Some((_id, debugger)) = &mut cpu_debugger {
        debugger.update(&mut nes);
        debugger.show(&mut nes);
    }
    if let Some((_id, debugger)) = &mut ppu_debugger {
        debugger.show(&mut nes);
    }
    if let Some((_id, debugger)) = &mut apu_debugger {
        debugger.show(&mut nes);
    }
    if let Some((_id, debugger)) = &mut memory_debugger {
        debugger.show(&mut nes);
    }
    if let Some((_id, debugger)) = &mut graphics_debugger {
        debugger.show(&mut nes);
    }

    let create_playback = move |notification_tx, audio_rx| {
        let a = audio_subsystem
            .open_playback(
                audio_subsystem
                    .audio_playback_device_name(0)
                    .ok()
                    .as_deref() as Option<&str>,
                &AudioSpecDesired {
                    freq: Some(44100),
                    channels: Some(1),
                    samples: Some(64),
                },
                |_| {
                    struct AudioRunner {
                        notification_tx: std::sync::mpsc::Sender<u8>,
                        audio_rx:
                            std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Vec<u16>>>>,
                    }
                    impl AudioCallback for AudioRunner {
                        type Channel = u16;
                        fn callback(&mut self, buffer: &mut [Self::Channel]) {
                            dbg!(1);
                            if self.notification_tx.send(1).is_ok() {
                                dbg!(2);
                                if let Ok(rx) = self.audio_rx.lock() {
                                    dbg!(3);
                                    if let Ok(received) = rx.recv_timeout(std::time::Duration::from_secs(1)) {
                                        dbg!(4);
                                        for i in 0..64 {
                                            buffer[i] = received[i];
                                        }
                                    }
                                }
                            }
                        }
                    }
                    AudioRunner {
                        notification_tx,
                        audio_rx,
                    }
                },
            )
            .unwrap();
        a.resume();
        a
    };

    let mut state = EmulatorState {
        running: true,
        exit: false,
        one_step: false,
    };

    let mut playback = if state.running {
        Some(create_playback(notification_tx.clone(), audio_rx.clone()))
    } else {
        None
    };

    let mut last_frame_time = std::time::Instant::now();

    loop {
        let mut game_window_ref = game_window.borrow_mut();

        if !state.running || game_window_ref.frame_generation_time != last_frame_time {
            let old_running = state.running;

            for event in event_pump.poll_iter() {
                let event_id = event.get_window_id();
                if event.get_window_id() == Some(game_window_id) {
                    game_window_ref.handle_event(event, &mut nes, &mut state);
                    continue;
                }
                match &mut cpu_debugger {
                    Some((id, debugger)) if event_id == Some(*id) => {
                        debugger.handle_event(event, &mut nes, &mut state);
                        continue;
                    }
                    _ => {}
                }
                match &mut ppu_debugger {
                    Some((id, debugger)) if event_id == Some(*id) => {
                        debugger.handle_event(event, &mut nes, &mut state);
                        continue;
                    }
                    _ => {}
                }
                match &mut apu_debugger {
                    Some((id, debugger)) if event_id == Some(*id) => {
                        debugger.handle_event(event, &mut nes, &mut state);
                        continue;
                    }
                    _ => {}
                }
                match &mut memory_debugger {
                    Some((id, debugger)) if event_id == Some(*id) => {
                        debugger.handle_event(event, &mut nes, &mut state);
                        continue;
                    }
                    _ => {}
                }
                match &mut graphics_debugger {
                    Some((id, debugger)) if event_id == Some(*id) => {
                        debugger.handle_event(event, &mut nes, &mut state);
                        continue;
                    }
                    _ => {}
                }
            }

            if state.exit {
                break;
            }

            if old_running != state.running {
                if state.running {
                    playback = Some(create_playback(notification_tx.clone(), audio_rx.clone()));
                } else {
                    playback = None;
                }
            }

            if !state.running && state.one_step {
                nes.run_one_cpu_instruction();
                if let Some((_id, debugger)) = &mut cpu_debugger {
                    debugger.update(&mut nes);
                }
                state.one_step = false;
            }

            game_window_ref.show();

            if let Some((_id, debugger)) = &mut cpu_debugger {
                debugger.show(&mut nes);
            }
            if let Some((_id, debugger)) = &mut ppu_debugger {
                debugger.show(&mut nes);
            }
            if let Some((_id, debugger)) = &mut apu_debugger {
                debugger.show(&mut nes);
            }
            if let Some((_id, debugger)) = &mut memory_debugger {
                debugger.show(&mut nes);
            }
            if let Some((_id, debugger)) = &mut graphics_debugger {
                debugger.show(&mut nes);
            }

            last_frame_time = game_window_ref.frame_generation_time;
        }

        let refresh_rate = game_window_ref
            .canvas
            .window()
            .display_mode()
            .map_or(60, |m| m.refresh_rate);

        drop(game_window_ref);

        // If emulation is running, sleep the thread until more samples are requested.
        if state.running {
            match notification_rx.recv() {
                Ok(_) => {
                    if state.running {
                        for _ in 0..(64 * 1_789_773 / 44_100) {
                            nes.run_one_cpu_tick();
                            if let Some((_id, debugger)) = &mut cpu_debugger {
                                debugger.update(&mut nes);

                                if debugger.breakpoints.contains(&nes.cpu.program_counter) {
                                    while !nes.cpu.finished_instruction() {
                                        nes.run_one_cpu_tick();
                                        debugger.update(&mut nes);
                                    }
                                    state.running = false;
                                    playback = None;
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(_) => break,
            };
        // If emulation is not running, sleep the thread for approx. one frame.
        } else {
            std::thread::sleep(Duration::from_nanos(1_000_000_000 / refresh_rate as u64));
        }
    }

    // TODO playback might be stuck forever waiting for data on audio_rx
    drop(playback);
}

impl Drop for DummyAudio {
    fn drop(&mut self) {
        if self.record {
            match write_wave("audio.wav", self.record_samples.as_slice()) {
                Ok(_) => println!("Saved samples to audio.wav"),
                Err(error) => eprintln!("Failed to save samples to audio.wav: {error}"),
            }
        }
    }
}

fn write_wave(path: &str, samples: &[u16]) -> std::io::Result<()> {
    use std::io::Write;
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, file);

    writer.write(b"RIFF")?;
    writer.write(&(samples.len() as u32 * 2 + 36).to_le_bytes())?;
    writer.write(b"WAVE")?;
    writer.write(b"fmt ")?;
    writer.write(&16u32.to_le_bytes())?;
    writer.write(&1u16.to_le_bytes())?;
    writer.write(&1u16.to_le_bytes())?;
    writer.write(&44_100u32.to_le_bytes())?;
    writer.write(&(44_100u32 * 2 * 1 / 8).to_le_bytes())?;
    writer.write(&(2u16).to_le_bytes())?;
    writer.write(&(16u16).to_le_bytes())?;
    writer.write(b"data")?;
    writer.write(&(samples.len() as u32 * 1 * 16 / 8).to_le_bytes())?;

    for s in samples.iter() {
        writer.write(&s.to_le_bytes())?;
    }
    Ok(())
}
