use crate::EmulatorState;
use polones_core::nes::Nes;
use polones_core::ppu::PALLETTE;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;
use std::rc::Rc;

pub struct SdlGraphicsDebugger {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    mode: u8,
    grid: bool,
    pattern_palette: u8,
}

impl SdlGraphicsDebugger {
    pub const WIDTH: u32 = 512;
    pub const HEIGHT: u32 = 512;

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
            mode: 1,
            pattern_palette: 0,
            grid: false,
        }
    }

    pub fn handle_event(&mut self, _nes: &mut Nes, event: Event, state: &mut EmulatorState) {
        match event {
            Event::Window {
                win_event: WindowEvent::Close,
                ..
            } => {
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
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Num1),
                ..
            } => {
                self.mode = 1;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Num2),
                ..
            } => {
                self.mode = 2;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Num3),
                ..
            } => {
                self.mode = 3;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Num4),
                ..
            } => {
                self.mode = 4;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::G),
                ..
            } => {
                self.grid = !self.grid;
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::P),
                ..
            } => {
                self.pattern_palette = (self.pattern_palette + 1) & 0b111;
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, nes: &mut Nes) {
        let (_cpu, mut cpu_bus) = nes.split_into_cpu_and_bus();
        let (ppu, mut ppu_bus) = cpu_bus.split_into_ppu_and_bus();

        if self.mode == 1 || self.mode == 2 {
            let nt = ppu.control_register.get_background_tile_select() as u16;
            self.texture
                .with_lock(None, |data, _| {
                    // draw background from 4 nametables
                    for yn in 0..2usize {
                        for xn in 0..2usize {
                            for yc in 0..30usize {
                                for yf in 0..8usize {
                                    for xc in 0..32usize {
                                        let index = ppu_bus.read(
                                            (0x2000 + yn * 0x0800 + xn * 0x0400 + yc * 32 + xc)
                                                as u16,
                                        );
                                        let mut low = ppu_bus.read(
                                            (nt << 12)
                                                | (index as u16 >> 0 << 4)
                                                | (0b0000)
                                                | (yf as u16),
                                        );
                                        let mut high = ppu_bus.read(
                                            (nt << 12)
                                                | (index as u16 >> 0 << 4)
                                                | (0b1000)
                                                | (yf as u16),
                                        );
                                        let attribute_byte = ppu_bus.read(
                                            ((0x23C0 + yn * 0x0800 + xn * 0x0400)
                                                | (yc >> 2 << 3)
                                                | (xc >> 2))
                                                as u16,
                                        );
                                        let attribute =
                                            (attribute_byte >> ((yc & 2) << 1) >> (xc & 2)) & 0b11;

                                        for xf in 0..8 {
                                            let i: usize = xn * 256
                                                + yn * 512 * 240
                                                + yc * 512 * 8
                                                + yf * 512
                                                + xc * 8
                                                + xf;

                                            let (r, g, b) = if self.mode == 1 {
                                                if ((high >> 7 << 1) | low >> 7) == 0 {
                                                    PALLETTE[(ppu_bus.read(0x3F00) & 0b00111111)
                                                        as usize]
                                                } else {
                                                    let b = ppu_bus.read(
                                                        0x3F00
                                                            + ((attribute as u16) << 2)
                                                            + (((high as u16) >> 7 << 1)
                                                                | low as u16 >> 7),
                                                    ) & 0b00111111;
                                                    PALLETTE[b as usize]
                                                }
                                            } else {
                                                match (high >> 7 << 1) | (low >> 7) {
                                                    0 => (0, 0, 0),
                                                    1 => (75, 75, 75),
                                                    2 => (170, 170, 170),
                                                    3 => (255, 255, 255),
                                                    _ => unreachable!(),
                                                }
                                            };
                                            data[i * 4 + 0] = b;
                                            data[i * 4 + 1] = g;
                                            data[i * 4 + 2] = r;
                                            high <<= 1;
                                            low <<= 1;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // clear the remaining screen space
                    for byte in data[(240 * 2 * 512 * 4)..].iter_mut() {
                        *byte = 0;
                    }
                })
                .unwrap();
        } else if self.mode == 3 || self.mode == 4 {
            self.texture
                .with_lock(None, |data, _pitch| {
                    // draw sprites from both pattern tables
                    // color is specified by self.pattern_palette
                    for pt in 0..2usize {
                        for yc in 0..16usize {
                            for xc in 0..16usize {
                                for yf in 0..8usize {
                                    let mut low = ppu_bus.read(
                                        ((pt << 12) | (yc << 8) | (xc << 4) | (0b0000) | yf) as u16,
                                    );
                                    let mut high = ppu_bus.read(
                                        ((pt << 12) | (yc << 8) | (xc << 4) | (0b1000) | yf) as u16,
                                    );
                                    for xf in 0..8 {
                                        let i: usize = pt * 256
                                            + yc * 512 * 8 * 2
                                            + yf * 512 * 2
                                            + xc * 8 * 2
                                            + xf * 2;

                                        let (r, g, b) = if self.mode == 3 {
                                            if ((high >> 7 << 1) | low >> 7) == 0 {
                                                PALLETTE
                                                    [(ppu_bus.read(0x3F00) & 0b00111111) as usize]
                                            } else {
                                                let b = ppu_bus.read(
                                                    0x3F00
                                                        + ((self.pattern_palette as u16) << 2)
                                                        + (((high as u16) >> 7 << 1)
                                                            | low as u16 >> 7),
                                                ) & 0b00111111;
                                                PALLETTE[b as usize]
                                            }
                                        } else {
                                            match (high >> 7 << 1) | (low >> 7) {
                                                0 => (0, 0, 0),
                                                1 => (75, 75, 75),
                                                2 => (170, 170, 170),
                                                3 => (255, 255, 255),
                                                _ => unreachable!(),
                                            }
                                        };

                                        if self.grid && (xf == 0 || yf == 0) {
                                            data[i * 4 + 0] = 0;
                                            data[i * 4 + 1] = 0;
                                            data[i * 4 + 2] = 255;
                                        } else {
                                            data[i * 4 + 0] = b;
                                            data[i * 4 + 1] = g;
                                            data[i * 4 + 2] = r;
                                        }

                                        data[(i + 1) * 4 + 0] = b;
                                        data[(i + 1) * 4 + 1] = g;
                                        data[(i + 1) * 4 + 2] = r;

                                        data[(i + 512) * 4 + 0] = b;
                                        data[(i + 512) * 4 + 1] = g;
                                        data[(i + 512) * 4 + 2] = r;

                                        data[(i + 512 + 1) * 4 + 0] = b;
                                        data[(i + 512 + 1) * 4 + 1] = g;
                                        data[(i + 512 + 1) * 4 + 2] = r;

                                        low <<= 1;
                                        high <<= 1;
                                    }
                                }
                            }
                        }
                    }

                    // clear the rest of the screen (bottom half)
                    for y in 0..256 {
                        for x in 0..512 {
                            data[((256 * 512) + y * 512 + x) * 4 + 0] = 0;
                            data[((256 * 512) + y * 512 + x) * 4 + 1] = 0;
                            data[((256 * 512) + y * 512 + x) * 4 + 2] = 0;
                            data[((256 * 512) + y * 512 + x) * 4 + 3] = 0;
                        }
                    }

                    // draw sprites in oam (if sprites are 8 pixels tall)
                    if !ppu.control_register.get_sprite_height() {
                        for yc in 0..8usize {
                            for yf in 0..8usize {
                                for xc in 0..8usize {
                                    let index = ppu.oam[(yc * 8 + xc) * 4 + 1];
                                    let palette = ppu.oam[(yc * 8 + xc) * 4 + 2] & 0b11;
                                    let pt = ppu.control_register.get_sprite_tile_select() as u8;
                                    let tile = index;

                                    let mut low = ppu_bus.read(
                                        ((pt as u16) << 12)
                                            | ((tile as u16) << 4)
                                            | (0b0000)
                                            | yf as u16,
                                    );
                                    let mut high = ppu_bus.read(
                                        ((pt as u16) << 12)
                                            | ((tile as u16) << 4)
                                            | (0b1000)
                                            | yf as u16,
                                    );

                                    for xf in 0..8usize {
                                        let color = (high >> 7 << 1) | low >> 7;
                                        let (r, g, b) = if self.mode == 4 {
                                            match color {
                                                0 => (0, 0, 0),
                                                1 => (75, 75, 75),
                                                2 => (170, 170, 170),
                                                3 => (255, 255, 255),
                                                _ => unreachable!(),
                                            }
                                        } else {
                                            if color == 0 {
                                                PALLETTE
                                                    [(ppu_bus.read(0x3F00) & 0b00111111) as usize]
                                            } else {
                                                let b = ppu_bus.read(
                                                    0x3F10 + ((palette as u16) << 2) + color as u16,
                                                ) & 0b00111111;
                                                PALLETTE[b as usize]
                                            }
                                        };

                                        let i = (256 * 512)
                                            + (yc * 512 * 8 * 4)
                                            + (yf * 512 * 4)
                                            + (xc * 8 * 4)
                                            + (xf * 4);

                                        for y in 0..4 {
                                            for x in 0..4 {
                                                if self.grid
                                                    && (xf == 0 || yf == 0)
                                                    && x == 0
                                                    && y == 0
                                                {
                                                    data[(i + y * 512 + x) * 4 + 0] = 0;
                                                    data[(i + y * 512 + x) * 4 + 1] = 0;
                                                    data[(i + y * 512 + x) * 4 + 2] = 255;
                                                } else {
                                                    data[(i + y * 512 + x) * 4 + 0] = b;
                                                    data[(i + y * 512 + x) * 4 + 1] = g;
                                                    data[(i + y * 512 + x) * 4 + 2] = r;
                                                }
                                            }
                                        }

                                        low <<= 1;
                                        high <<= 1;
                                    }
                                }
                            }
                        }
                    }
                    // draw sprites in oam (if sprites are 16 pixels tall)
                    else {
                        for yc in 0..8usize {
                            for yf in 0..16usize {
                                for xc in 0..8usize {
                                    let index = ppu.oam[(yc * 8 + xc) * 4 + 1];
                                    let palette = ppu.oam[(yc * 8 + xc) * 4 + 2] & 0b11;

                                    let pt = index & 1;
                                    let tile = if yf <= 7 {
                                        index & 0b11111110
                                    } else {
                                        index | 0b00000001
                                    };

                                    let mut low = ppu_bus.read(
                                        ((pt as u16) << 12)
                                            | ((tile as u16) << 4)
                                            | (0b0000)
                                            | (yf & 0b111) as u16,
                                    );
                                    let mut high = ppu_bus.read(
                                        ((pt as u16) << 12)
                                            | ((tile as u16) << 4)
                                            | (0b1000)
                                            | (yf & 0b111) as u16,
                                    );

                                    for xf in 0..8usize {
                                        let (r, g, b) = if ((high >> 7 << 1) | low >> 7) == 0 {
                                            PALLETTE[(ppu_bus.read(0x3F00) & 0b00111111) as usize]
                                        } else {
                                            let b = ppu_bus.read(
                                                0x3F10
                                                    + ((palette as u16) << 2)
                                                    + (((high as u16) >> 7 << 1) | low as u16 >> 7),
                                            ) & 0b00111111;
                                            PALLETTE[b as usize]
                                        };

                                        let i = (256 * 512)
                                            + (yc * 512 * 8 * 4)
                                            + (yf * 512 * 2)
                                            + (xc * 8 * 2)
                                            + (xf * 2);

                                        for y in 0..2 {
                                            for x in 0..2 {
                                                if self.grid
                                                    && (xf == 0 || yf == 0)
                                                    && x == 0
                                                    && y == 0
                                                {
                                                    data[(i + y * 512 + x) * 4 + 0] = 0;
                                                    data[(i + y * 512 + x) * 4 + 1] = 0;
                                                    data[(i + y * 512 + x) * 4 + 2] = 255;
                                                } else {
                                                    data[(i + y * 512 + x) * 4 + 0] = b;
                                                    data[(i + y * 512 + x) * 4 + 1] = g;
                                                    data[(i + y * 512 + x) * 4 + 2] = r;
                                                }
                                            }
                                        }

                                        low <<= 1;
                                        high <<= 1;
                                    }
                                }
                            }
                        }
                    }

                    // draw palettes
                    for yc in 0..8usize {
                        // draw palette indicator
                        if yc == self.pattern_palette as usize {
                            for yf in 0..8usize {
                                for xf in 0..8 {
                                    let i = (512 * 8)
                                        + (256 + 8)
                                        + 512 * 256
                                        + yc * 2 * 512 * 8
                                        + yf * 512
                                        + xf;
                                    data[i * 4 + 0] = 0;
                                    data[i * 4 + 1] = 0;
                                    data[i * 4 + 2] = 255;
                                }
                            }
                        }

                        for yf in 0..8usize {
                            for xc in 0..4usize {
                                let byte = ppu_bus.read(0x3F00 | ((yc as u16) << 2) | xc as u16);
                                let (r, g, b) = PALLETTE[byte as usize & 0b00111111];
                                for xf in 0..8 {
                                    let i = (512 * 8)
                                        + (256 + 24)
                                        + 512 * 256
                                        + yc * 2 * 512 * 8
                                        + yf * 512
                                        + xc * 8
                                        + xf;
                                    data[i * 4 + 0] = b;
                                    data[i * 4 + 1] = g;
                                    data[i * 4 + 2] = r;
                                }
                            }
                        }
                    }
                })
                .unwrap();
        }

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
