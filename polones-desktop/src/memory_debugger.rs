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
    page_number: u16,
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
            page_number: 0,
        }
    }

    pub fn handle_event(&mut self, _nes: &mut Nes, event: Event, state: &mut EmulatorState) {
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
                self.page_number = increase_in_ranges(&page_ranges, self.page_number);
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Down),
                ..
            } => {
                self.page_number = decrease_in_ranges(&page_ranges, self.page_number);
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::C),
                ..
            } => {
                self.page_number = 0x00;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::P),
                ..
            } => {
                self.page_number = 0x100;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::O),
                ..
            } => {
                self.page_number = 0x140;
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, nes: &mut Nes) {
        let ta = &mut self.text_area;
        ta.clear();

        match self.page_number {
            0x00..=0xFF => {
                ta.write_str_with_color("CPU BUS", 0, 0, Yellow);
                ta.write_u16_with_color(256 * self.page_number, 0, 8, White);
            }
            0x100..=0x13F => {
                ta.write_str_with_color("PPU BUS", 0, 0, Yellow);
                ta.write_u16_with_color(256 * (self.page_number - 0x100), 0, 8, White);
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

        match self.page_number {
            0x00..=0xFF => {
                let (cpu, mut cpu_bus) = nes.split_into_cpu_and_bus();
                for y in 0..16 {
                    for x in 0..16 {
                        self.text_area.write_u8_with_color(
                            cpu_bus.read((256 * self.page_number) + (y as u16 * 16) + x as u16),
                            3 + y as u8 * 2,
                            1 + x as u8 * 3,
                            if x % 2 == 0 { White } else { Cyan },
                        );
                    }
                }
                if self.page_number == 0x01 {
                    let y = cpu.stack_pointer >> 4;
                    let x = cpu.stack_pointer & 0x0F;
                    self.text_area.write_u8_with_color(
                        cpu_bus.read((256 * self.page_number) + (y as u16 * 16) + x as u16),
                        3 + y * 2,
                        1 + x * 3,
                        Magenta,
                    );
                }
                if self.page_number == cpu.program_counter & 0xFF00 {
                    let y = cpu.program_counter as u8 >> 4;
                    let x = cpu.program_counter as u8 & 0x0F;
                    self.text_area.write_u8_with_color(
                        cpu_bus.read((256 * self.page_number) + (y as u16 * 16) + x as u16),
                        3 + y * 2,
                        1 + x * 3,
                        Red,
                    );
                }
            }
            0x100..=0x13F => {
                let (_cpu, mut cpu_bus) = nes.split_into_cpu_and_bus();
                let (_ppu, mut ppu_bus) = cpu_bus.split_into_ppu_and_bus();
                for y in 0..16 {
                    for x in 0..16 {
                        self.text_area.write_u8_with_color(
                            ppu_bus.read(
                                256 * (self.page_number - 0x100) + (y as u16 * 16) + x as u16,
                            ),
                            3 + y * 2,
                            1 + x * 3,
                            if x % 2 == 0 { White } else { Cyan },
                        );
                    }
                }
            }
            0x140 => {
                let ppu = &nes.ppu;
                for y in 0..16 {
                    for x in 0..16 {
                        self.text_area.write_u8_with_color(
                            ppu.oam[(y as usize * 16) + x as usize],
                            3 + y * 2,
                            1 + x * 3,
                            if x % 2 == 0 { White } else { Cyan },
                        );
                    }
                }
                let y = ppu.oam_address >> 4;
                let x = ppu.oam_address & 0x0F;
                self.text_area.write_u8_with_color(
                    ppu.oam[(y as usize * 16) + x as usize],
                    3 + y * 2,
                    1 + x * 3,
                    Magenta,
                );
            }
            _ => unreachable!(),
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
