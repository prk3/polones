use cpu_debugger::SdlCpuDebugger;
use graphics_debugger::SdlGraphicsDebugger;
use memory_debugger::SdlMemoryDebugger;
use polones_core::game_file::GameFile;
use polones_core::nes::{Display, Frame, Input, Nes, PortState};
use ppu_debugger::SdlPpuDebugger;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

mod cpu_debugger;
mod graphics_debugger;
mod memory_debugger;
mod ppu_debugger;
mod text_area;

struct SdlGameWindow {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
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
}

impl SdlGameWindow {
    fn new(canvas: sdl2::render::WindowCanvas) -> Self {
        let mut canvas = canvas;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let texture_creator = Rc::new(canvas.texture_creator());
        let mut data = [0; SdlDisplay::WIDTH as usize * SdlDisplay::HEIGHT as usize * 4];
        let surface = Surface::from_data(
            &mut data[..],
            SdlDisplay::WIDTH,
            SdlDisplay::HEIGHT,
            SdlDisplay::WIDTH * 4,
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
        self.canvas.present();
    }
}

struct SdlDisplay {
    inner: Rc<RefCell<SdlGameWindow>>,
}

impl SdlDisplay {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;

    fn new(inner: Rc<RefCell<SdlGameWindow>>) -> Self {
        Self { inner }
    }
}

impl Display for SdlDisplay {
    fn draw(&mut self, frame: Box<Frame>) {
        let mut inner = self.inner.borrow_mut();

        let mut data = [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4];
        for y in 0..Self::HEIGHT as usize {
            for x in 0..Self::WIDTH as usize {
                data[4 * (y * Self::WIDTH as usize + x) + 0] = frame[y][x].2;
                data[4 * (y * Self::WIDTH as usize + x) + 1] = frame[y][x].1;
                data[4 * (y * Self::WIDTH as usize + x) + 2] = frame[y][x].0;
            }
        }

        inner
            .texture
            .update(
                Rect::new(0, 0, Self::WIDTH, Self::HEIGHT),
                &data,
                Self::WIDTH as usize * 4,
            )
            .unwrap();

        let display_size = inner.canvas.window().size();

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

        // TODO This deconstruction trick is smart as fuck, I have to write
        // a blog post about it.
        let SdlGameWindow {
            canvas, texture, ..
        } = &mut *inner;
        canvas.clear();
        canvas.copy(texture, frame_rect, scaled_frame_rect).unwrap();
    }
}

struct SdlInput {
    inner: Rc<RefCell<SdlGameWindow>>,
}

impl SdlInput {
    pub fn new(inner: Rc<RefCell<SdlGameWindow>>) -> Self {
        Self { inner }
    }
}

impl Input for SdlInput {
    fn read_port_1(&mut self) -> PortState {
        let inner = self.inner.as_ref().borrow();
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
        let inner = self.inner.as_ref().borrow();
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

fn main() {
    let show_cpu_debugger = false;
    let show_ppu_debugger = false;
    let show_graphics_debugger = false;
    let show_memory_debugger = false;

    let args = std::env::args().collect::<Vec<String>>();
    let file_contents = std::fs::read(args.get(1).expect("file argument missing"))
        .expect("could not read the file");
    let game_file = GameFile::read(args.get(1).unwrap().to_string(), file_contents)
        .expect("file does not contain a nes game");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let game_window = video_subsystem
        .window("nes display", SdlDisplay::WIDTH * 3, SdlDisplay::HEIGHT * 3)
        .position(0, 720)
        .build()
        .unwrap();
    let game_window_id = game_window.id();
    let game_canvas = game_window.into_canvas().build().unwrap();

    let mut cpu_debugger = show_cpu_debugger.then(|| {
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

    let mut ppu_debugger = show_ppu_debugger.then(|| {
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

    let mut memory_debugger = show_memory_debugger.then(|| {
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

    let mut graphics_debugger = show_graphics_debugger.then(|| {
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

    let game_window = Rc::new(RefCell::new(SdlGameWindow::new(game_canvas)));
    let display = SdlDisplay::new(game_window.clone());
    let input = SdlInput::new(game_window.clone());
    let mut nes = Nes::new(game_file, display, input).expect("Could not start the game");

    let mut state = EmulatorState {
        running: true,
        exit: false,
        one_step: false,
    };

    game_window.borrow_mut().show();

    if let Some((_id, debugger)) = &mut cpu_debugger {
        debugger.update(&mut nes);
        debugger.show(&mut nes);
    }
    if let Some((_id, debugger)) = &mut ppu_debugger {
        debugger.show(&mut nes);
    }
    if let Some((_id, debugger)) = &mut memory_debugger {
        debugger.show(&mut nes);
    }
    if let Some((_id, debugger)) = &mut graphics_debugger {
        debugger.show(&mut nes);
    }

    loop {
        let start_time = std::time::Instant::now();

        for event in event_pump.poll_iter() {
            let event_id = event.get_window_id();
            if event.get_window_id() == Some(game_window_id) {
                game_window
                    .borrow_mut()
                    .handle_event(event, &mut nes, &mut state);
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

        if !state.running && state.one_step {
            nes.run_one_cpu_instruction();
            if let Some((_id, debugger)) = &mut cpu_debugger {
                debugger.update(&mut nes);
            }
            state.one_step = false;
        } else if state.running {
            for _ in 0..29829 {
                nes.run_one_cpu_tick();
                if let Some((_id, debugger)) = &mut cpu_debugger {
                    debugger.update(&mut nes);

                    if debugger.breakpoints.contains(&nes.cpu.program_counter) {
                        while !nes.cpu.finished_instruction() {
                            nes.run_one_cpu_tick();
                            debugger.update(&mut nes);
                        }
                        state.running = false;
                        break;
                    }
                }
            }
        }

        game_window.borrow_mut().show();
        if let Some((_id, debugger)) = &mut cpu_debugger {
            debugger.show(&mut nes);
        }
        if let Some((_id, debugger)) = &mut ppu_debugger {
            debugger.show(&mut nes);
        }
        if let Some((_id, debugger)) = &mut memory_debugger {
            debugger.show(&mut nes);
        }
        if let Some((_id, debugger)) = &mut graphics_debugger {
            debugger.show(&mut nes);
        }

        // 60fps
        let nanos_to_sleep =
            Duration::from_nanos(1_000_000_000u64 / 60).saturating_sub(start_time.elapsed());
        if nanos_to_sleep != Duration::ZERO {
            std::thread::sleep(nanos_to_sleep);
        }
    }
}
