use crate::text_area::{Color, TextArea};
use crate::EmulatorState;
use polones_core::mapper::DebugValue;
use polones_core::nes::Nes;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;
use std::rc::Rc;

pub struct SdlMapperDebugger {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    text_area: TextArea<{ Self::WIDTH as usize / 8 }, { Self::HEIGHT as usize / 8 }>,
}

impl SdlMapperDebugger {
    pub const WIDTH: u32 = 256;
    pub const HEIGHT: u32 = 240;

    pub fn new(canvas: sdl2::render::WindowCanvas) -> Self {
        let mut canvas = canvas;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let texture_creator = Rc::new(canvas.texture_creator());
        let texture = texture_creator
            .create_texture_streaming(canvas.default_pixel_format(), Self::WIDTH, Self::HEIGHT)
            .unwrap();
        canvas.clear();
        canvas.present();
        Self {
            canvas,
            texture: unsafe { std::mem::transmute(texture) },
            _texture_creator: texture_creator,
            text_area: TextArea::new(),
        }
    }

    pub fn handle_event(&mut self, _nes: &mut Nes, event: Event, state: &mut EmulatorState) {
        match event {
            Event::Window { win_event: WindowEvent::Close, .. } => {
                state.exit = true;
            }
            Event::Quit { .. } => {
                state.exit = true;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Escape),
                ..
            } => {
                state.exit = true;
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, nes: &mut Nes) {
        let debug_info = nes.mapper.gather_debug_info();

        for (line, (property, value)) in debug_info.into_iter().take(30).enumerate() {
            self.text_area
                .write_str_with_color(property, line as u8, 0, Color::Yellow);
            match value {
                DebugValue::U8Hex(v) => {
                    self.text_area
                        .write_u8_with_color(v, line as u8, 20, Color::White);
                }
                DebugValue::U16Hex(v) => {
                    self.text_area
                        .write_u16_with_color(v, line as u8, 20, Color::White);
                }
                DebugValue::Dec(v) => {
                    self.text_area
                        .write_dec_with_color(v, line as u8, 20, Color::White);
                }
            }
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
