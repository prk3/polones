use polones_core::nes::Nes;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;
use std::ops::RangeInclusive;
use std::rc::Rc;

use crate::text_area::{Color::*, TextArea};
use crate::EmulatorState;

pub struct SdlMemoryDebugger {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    text_area: TextArea<{ Self::WIDTH as usize / 8 }, { Self::HEIGHT as usize / 8 }>,
    page: u16,
}

impl SdlMemoryDebugger {
    pub const WIDTH: u32 = 384;
    pub const HEIGHT: u32 = 360;

    pub fn new(canvas: sdl2::render::WindowCanvas) -> Self {
        let mut canvas = canvas;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let texture_creator = Rc::new(canvas.texture_creator());
        let mut texture = texture_creator
            .create_texture_streaming(canvas.default_pixel_format(), Self::WIDTH, Self::HEIGHT)
            .unwrap();
        texture
            .with_lock(None, |data, _pitch| {
                for byte in data {
                    *byte = 0;
                }
            })
            .unwrap();
        canvas.clear();
        canvas.present();

        Self {
            canvas,
            texture: unsafe { std::mem::transmute(texture) },
            _texture_creator: texture_creator,
            text_area: TextArea::new(),
            page: 0,
        }
    }

    pub fn show(&mut self, nes: &mut Nes) {
        let (cpu, mut cpu_bus) = nes.split_into_cpu_and_bus();
        let ta = &mut self.text_area;
        ta.clear();

        match self.page {
            0x00..=0xFF => {
                ta.write_str_with_color("CPU BUS", 0, 0, Yellow);
                ta.write_u16_with_color(256 * self.page, 0, 8, White);
            }
            0x100..=0x13F => {
                ta.write_str_with_color("PPU BUS", 0, 0, Yellow);
                ta.write_u16_with_color(256 * (self.page - 0x100), 0, 8, White);
            }
            0x140 => {
                ta.write_str_with_color("OAM", 0, 0, Yellow);
            }
            _ => unreachable!(),
        }

        ta.write_str_with_color("< >", 0, 13, Yellow);

        self.text_area.write_str_with_color(
            " 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F",
            2,
            0,
            Yellow,
        );

        for row in 0..16u8 {
            self.text_area.write_char_with_color(
                char::from_u32(if row < 10 {
                    '0' as u32 + row as u32
                } else {
                    'A' as u32 + row as u32 - 10
                })
                .unwrap(),
                3 + row * 2,
                0,
                Yellow,
            );
        }

        for y in 0..16u8 {
            for x in 0..16u8 {
                self.text_area.write_u8_with_color(
                    match self.page {
                        0x00..=0xFF => {
                            cpu_bus.read((256 * self.page) + (y as u16 * 16) + x as u16)
                        }
                        0x100..=0x13F => {
                            let (_, mut ppu_bus) = cpu_bus.split_into_ppu_and_bus();
                            ppu_bus.read(256 * (self.page - 0x100) + (y as u16 * 16) + x as u16)
                        }
                        0x140 => cpu_bus.ppu.oam[y as usize * 16 + x as usize],
                        _ => unreachable!(),
                    },
                    3 + y * 2,
                    1 + x * 3,
                    if x % 2 == 0 { White } else { Cyan },
                );
            }
        }

        if self.page == 0x01 {
            let sp = cpu.stack_pointer;
            let y = sp >> 4;
            let x = sp & 0x0F;
            self.text_area.write_u8_with_color(
                cpu_bus.read(0x0100 + (y as u16 * 16) + x as u16),
                3 + y * 2,
                1 + x * 3,
                Magenta,
            );
        }

        if self.page == cpu.program_counter & 0xFF00 {
            let pc = cpu.program_counter;
            let y = pc as u8 >> 4;
            let x = pc as u8 & 0x0F;
            self.text_area
                .write_u8_with_color(cpu_bus.read(pc), 3 + y * 2, 1 + x * 3, Red);
        }

        if self.page == 0x140 {
            let address = cpu_bus.ppu.oam_address;
            let y = address >> 4;
            let x = address & 0x0F;
            self.text_area.write_u8_with_color(
                cpu_bus.ppu.oam[address as usize],
                3 + y * 2,
                1 + x * 3,
                Magenta,
            );
        }

        self.texture
            .with_lock(None, |data, _pitch| {
                self.text_area.draw_to_texture(data);
            })
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

    pub fn handle_event(&mut self, event: Event, _nes: &mut Nes, state: &mut EmulatorState) {
        let page_ranges = [0x00..=0x19, 0x80..=0x140];
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
                keycode: _k @ Some(Keycode::Up),
                ..
            } => {
                self.page = increase_in_ranges(&page_ranges, self.page);
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Down),
                ..
            } => {
                self.page = decrease_in_ranges(&page_ranges, self.page);
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::C),
                ..
            } => {
                self.page = 0x00;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::P),
                ..
            } => {
                self.page = 0x100;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::O),
                ..
            } => {
                self.page = 0x140;
            }
            _ => {}
        }
    }
}

fn decrease_in_ranges(ranges: &[RangeInclusive<u16>], value: u16) -> u16 {
    for i in 0..ranges.len() {
        if ranges[i].contains(&value) {
            return if value > *ranges[i].start() {
                value - 1
            } else if i > 0 {
                *ranges[i - 1].end()
            } else {
                value
            };
        }
    }
    value
}

fn increase_in_ranges(ranges: &[RangeInclusive<u16>], value: u16) -> u16 {
    for i in 0..ranges.len() {
        if ranges[i].contains(&value) {
            return if value < *ranges[i].end() {
                value + 1
            } else if i < ranges.len() - 1 {
                *ranges[i + 1].start()
            } else {
                value
            };
        }
    }
    value
}
