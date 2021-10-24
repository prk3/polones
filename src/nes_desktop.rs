use nes_lib::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use std::rc::Rc;
use std::time::Duration;

const FONT: &[u8; 16 * 32] = include_bytes!("../resources/comodore-64-font.bin");

const BLACK: (u8, u8, u8) = (0, 0, 0);
const RED: (u8, u8, u8) = (255, 0, 0);
const GREEN: (u8, u8, u8) = (0, 255, 0);
const BLUE: (u8, u8, u8) = (0, 0, 255);
const YELLOW: (u8, u8, u8) = (255, 255, 0);
const CYAN: (u8, u8, u8) = (0, 255, 255);
const MAGENTA: (u8, u8, u8) = (255, 0, 255);
const WHITE: (u8, u8, u8) = (255, 255, 255);

struct SdlDisplay {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
}

impl SdlDisplay {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;

    fn new(canvas: sdl2::render::WindowCanvas) -> Self {
        let mut canvas = canvas;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let texture_creator = Rc::new(canvas.texture_creator());
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
        }
    }
}

impl Display for SdlDisplay {
    fn display(&mut self, frame: Box<Frame>) {
        self.canvas.clear();

        let mut data = [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4];
        for y in 0..Self::HEIGHT as usize {
            for x in 0..Self::WIDTH as usize {
                data[4 * (y * Self::WIDTH as usize + x) + 0] = frame[y][x].2;
                data[4 * (y * Self::WIDTH as usize + x) + 1] = frame[y][x].1;
                data[4 * (y * Self::WIDTH as usize + x) + 2] = frame[y][x].0;
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

        self.canvas
            .copy(&self.texture, frame_rect, scaled_frame_rect)
            .unwrap();
        self.canvas.present();
    }
}

struct SdlCpuDebugDisplay {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    buffer: [u8; Self::WIDTH as usize * Self::HEIGHT as usize * 4],
}

impl SdlCpuDebugDisplay {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 300;

    fn new(canvas: sdl2::render::WindowCanvas) -> Self {
        let mut canvas = canvas;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let texture_creator = Rc::new(canvas.texture_creator());
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
        canvas.present();
        Self {
            canvas,
            texture: unsafe { std::mem::transmute(texture) },
            _texture_creator: texture_creator,
            buffer: [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4],
        }
    }

    fn write_char_with_color(&mut self, value: char, line: u8, col: u8, color: (u8, u8, u8)) {
        if line < 25 && col < 50 {
            let (char_row, char_col) = match value {
                _c @ 'A'..='O' => (0, 1 + (value as u8 - 'A' as u8)),
                _c @ 'a'..='o' => (0, 1 + (value as u8 - 'a' as u8)),
                _c @ 'P'..='Z' => (1, value as u8 - 'P' as u8),
                _c @ 'p'..='z' => (1, value as u8 - 'p' as u8),
                _c @ '0'..='9' => (3, value as u8 - '0' as u8),
                '@' => (0, 0),
                '[' => (1, 11),
                '£' => (1, 12),
                ']' => (1, 13),
                '↑' => (1, 14),
                '←' => (1, 15),
                ' ' => (2, 0),
                '!' => (2, 1),
                '"' => (2, 2),
                '#' => (2, 3),
                '$' => (2, 4),
                '%' => (2, 5),
                '&' => (2, 6),
                '\'' => (2, 7),
                '(' => (2, 8),
                ')' => (2, 9),
                '*' => (2, 10),
                '+' => (2, 11),
                ',' => (2, 12),
                '-' => (2, 13),
                '.' => (2, 14),
                '/' => (2, 15),
                ':' => (3, 10),
                ';' => (3, 11),
                '<' => (3, 12),
                '=' => (3, 13),
                '>' => (3, 14),
                '?' => (3, 15),
                _ => (3, 15),
            };

            for y in 0..8 {
                let mut row = FONT[((char_row as usize * 8) + y) * 16 + char_col as usize];
                for x in 0..8 {
                    let (r, g, b) = if row & 0b10000000 > 0 {
                        color
                    } else {
                        (0, 0, 0)
                    };
                    self.buffer[4 * ((line as usize * 8 + y) * 400 + (col as usize * 8) + x) + 0] =
                        b;
                    self.buffer[4 * ((line as usize * 8 + y) * 400 + (col as usize * 8) + x) + 1] =
                        g;
                    self.buffer[4 * ((line as usize * 8 + y) * 400 + (col as usize * 8) + x) + 2] =
                        r;
                    row = row << 1;
                }
            }
        }
    }

    fn write_str_with_color(&mut self, value: &str, line: u8, col: u8, color: (u8, u8, u8)) {
        let mut col = col;
        for c in value.chars() {
            self.write_char_with_color(c, line, col, color);
            col = col.saturating_add(1);
        }
    }

    fn write_u8_with_color(&mut self, value: u8, line: u8, col: u8, color: (u8, u8, u8)) {
        let hex2 = value >> 4 & 0b1111;
        let hex1 = value >> 0 & 0b1111;
        let num_to_char = |c| {
            char::from_u32(if c < 10 {
                '0' as u32 + c as u32
            } else {
                'A' as u32 + c as u32 - 10
            })
            .unwrap()
        };
        self.write_char_with_color(num_to_char(hex2), line, col, color);
        self.write_char_with_color(num_to_char(hex1), line, col.saturating_add(1), color);
    }

    fn write_u16_with_color(&mut self, value: u16, line: u8, col: u8, color: (u8, u8, u8)) {
        let hex4 = value >> 12 & 0b1111;
        let hex3 = value >> 8 & 0b1111;
        let hex2 = value >> 4 & 0b1111;
        let hex1 = value >> 0 & 0b1111;
        let num_to_char = |c| {
            char::from_u32(if c < 10 {
                '0' as u32 + c as u32
            } else {
                'A' as u32 + c as u32 - 10
            })
            .unwrap()
        };
        self.write_char_with_color(num_to_char(hex4), line, col, color);
        self.write_char_with_color(num_to_char(hex3), line, col.saturating_add(1), color);
        self.write_char_with_color(num_to_char(hex2), line, col.saturating_add(2), color);
        self.write_char_with_color(num_to_char(hex1), line, col.saturating_add(3), color);
    }
}

impl CpuDebugDisplay for SdlCpuDebugDisplay {
    fn display<B: bus::Bus>(&mut self, cpu: &cpu::Cpu<B>) {
        self.canvas.clear();

        self.write_str_with_color("hej rafal!", 1, 0, WHITE);
        self.write_str_with_color("co masz w lodowce?", 2, 0, YELLOW);
        self.write_str_with_color("abc", 3, 10, CYAN);
        self.write_u8_with_color(65, 4, 0, BLUE);
        self.write_u16_with_color(u16::MAX/ 2 +144, 5, 0, BLUE);

        self.write_str_with_color("kocham angelisie", 20, 30, RED);

        self.texture
            .update(
                Rect::new(0, 0, Self::WIDTH, Self::HEIGHT),
                &self.buffer,
                Self::WIDTH as usize * 4,
            )
            .unwrap();

        self.canvas
            .copy(
                &self.texture,
                Rect::new(0, 0, Self::WIDTH, Self::HEIGHT),
                Rect::new(
                    0,
                    0,
                    self.canvas.window().size().0,
                    self.canvas.window().size().1,
                ),
            )
            .unwrap();
        self.canvas.present();
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("nes display", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let mut display = SdlDisplay::new(canvas);

    let window2 = video_subsystem
        .window("cpu inspector", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let canvas2 = window2.into_canvas().build().unwrap();

    let mut cpu_debug_display = SdlCpuDebugDisplay::new(canvas2);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut c: u8 = 0;

    'running: loop {
        c = c.wrapping_add(1);
        display.display(Box::new([[(c, 0, 0); 256]; 240]));

        let cpu = cpu::Cpu::<bus::MainBus>::new();
        cpu_debug_display.display(&cpu);

        for event in event_pump.poll_iter() {
            event.get_window_id();
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
