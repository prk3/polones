use crate::EmulatorState;
use polones_core::nes::Nes;
use polones_core::ppu::PALLETTE;
use sdl2::event::Event;
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
    graphics_state: GraphicsState,
}

struct GraphicsState {
    oam: [u8; 256],
    palette: [u8; 32],
    pattern_tables: [[u8; 4096]; 2],
    nametables: [[u8; 1024]; 4],
    background_tile_select: bool,
    sprite_height: bool,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            oam: [0; 256],
            palette: [0; 32],
            pattern_tables: [[0; 4096]; 2],
            nametables: [[0; 1024]; 4],
            background_tile_select: false,
            sprite_height: false,
        }
    }
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
            graphics_state: GraphicsState::default(),
        }
    }

    pub fn handle_event(&mut self, event: Event, state: &mut EmulatorState) {
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

    pub fn update(&mut self, nes: &mut Nes) {
        let (_, mut cpu_bus) = nes.split_into_cpu_and_bus();
        let (ppu, mut ppu_bus) = cpu_bus.split_into_ppu_and_bus();

        let s = &mut self.graphics_state;

        s.background_tile_select = ppu.control_register.get_background_tile_select();
        s.sprite_height = ppu.control_register.get_sprite_height();

        match self.mode {
            1 | 2 => {
                if s.background_tile_select {
                    for i in 0..4096 {
                        s.pattern_tables[1][i] = ppu_bus.read(0x1000 + i as u16);
                    }
                } else {
                    for i in 0..4096 {
                        s.pattern_tables[0][i] = ppu_bus.read(i as u16);
                    }
                }
                for i in 0..1024 {
                    s.nametables[0][i] = ppu_bus.read(0x2000 + i as u16);
                }
                for i in 0..1024 {
                    s.nametables[1][i] = ppu_bus.read(0x2400 + i as u16);
                }
                for i in 0..1024 {
                    s.nametables[2][i] = ppu_bus.read(0x2800 + i as u16);
                }
                for i in 0..1024 {
                    s.nametables[3][i] = ppu_bus.read(0x2C00 + i as u16);
                }
                if self.mode == 1 {
                    for i in 0..32 {
                        s.palette[i] = ppu_bus.ppu_palette_ram.read(i);
                    }
                }
            }
            3 | 4 => {
                for i in 0..4096 {
                    s.pattern_tables[0][i] = ppu_bus.read(i as u16);
                }
                for i in 0..4096 {
                    s.pattern_tables[1][i] = ppu_bus.read(0x1000 + i as u16);
                }
                for i in 0..256 {
                    s.oam[i] = ppu.oam[i];
                }
                for i in 0..32 {
                    s.palette[i] = ppu_bus.ppu_palette_ram.read(i);
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn draw(&mut self) {
        let s = &mut self.graphics_state;

        if self.mode == 1 || self.mode == 2 {
            let pt = s.background_tile_select as usize;
            self.texture
                .with_lock(None, |data, _| {
                    // draw background from 4 nametables
                    for yn in 0..2 {
                        for xn in 0..2 {
                            for yc in 0..30 {
                                for yf in 0..8 {
                                    for xc in 0..32 {
                                        let index =
                                            s.nametables[yn * 2 + xn][yc * 32 + xc] as usize;
                                        let mut low = s.pattern_tables[pt]
                                            [(index >> 0 << 4) | (0b0000) | (yf)];
                                        let mut high = s.pattern_tables[pt]
                                            [(index >> 0 << 4) | (0b1000) | (yf)];

                                        let attribute_byte = s.nametables[yn * 2 + xn]
                                            [0x3C0 | (yc >> 2 << 3) | (xc >> 2)];
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
                                                    PALLETTE[(s.palette[0]) as usize]
                                                } else {
                                                    let b = s.palette[((attribute as usize) << 2)
                                                        + (((high as usize) >> 7 << 1)
                                                            | low as usize >> 7)];
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
                    for pt in 0..2 {
                        for yc in 0..16 {
                            for xc in 0..16 {
                                for yf in 0..8 {
                                    let mut low =
                                        s.pattern_tables[pt][(yc << 8) | (xc << 4) | (0b0000) | yf];
                                    let mut high =
                                        s.pattern_tables[pt][(yc << 8) | (xc << 4) | (0b1000) | yf];
                                    for xf in 0..8 {
                                        let i: usize = pt * 256
                                            + yc * 512 * 8 * 2
                                            + yf * 512 * 2
                                            + xc * 8 * 2
                                            + xf * 2;

                                        let (r, g, b) = if self.mode == 3 {
                                            if ((high >> 7 << 1) | low >> 7) == 0 {
                                                PALLETTE[s.palette[0] as usize]
                                            } else {
                                                let b = s.palette[(self.pattern_palette as usize)
                                                    << 2
                                                    | ((high as usize) >> 7 << 1)
                                                    | low as usize >> 7];
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
                    if !s.sprite_height {
                        let pt = !s.background_tile_select as usize;

                        for yc in 0..8 {
                            for yf in 0..8 {
                                for xc in 0..8 {
                                    let index = s.oam[(yc * 8 + xc) * 4 + 1];
                                    let palette = s.oam[(yc * 8 + xc) * 4 + 2] & 0b11;
                                    let tile = index;

                                    let mut low = s.pattern_tables[pt]
                                        [((tile as usize) << 4) | (0b0000) | yf];
                                    let mut high = s.pattern_tables[pt]
                                        [((tile as usize) << 4) | (0b1000) | yf];

                                    for xf in 0..8usize {
                                        let (r, g, b) = if ((high >> 7 << 1) | low >> 7) == 0 {
                                            PALLETTE[s.palette[0] as usize]
                                        } else {
                                            let b = s.palette[0x10
                                                | ((palette as usize) << 2)
                                                | (((high as usize) >> 7 << 1)
                                                    | low as usize >> 7)];
                                            PALLETTE[b as usize]
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
                        for yc in 0..8 {
                            for yf in 0..16 {
                                for xc in 0..8 {
                                    let index = s.oam[(yc * 8 + xc) * 4 + 1];
                                    let palette = s.oam[(yc * 8 + xc) * 4 + 2] & 0b11;

                                    let pt = (index & 1) as usize;
                                    let tile = if yf <= 7 {
                                        index & 0b11111110
                                    } else {
                                        index | 0b00000001
                                    };

                                    let mut low = s.pattern_tables[pt]
                                        [((tile as usize) << 4) | (0b0000) | (yf & 0b111)];
                                    let mut high = s.pattern_tables[pt]
                                        [((tile as usize) << 4) | (0b1000) | (yf & 0b111)];

                                    for xf in 0..8usize {
                                        let (r, g, b) = if ((high >> 7 << 1) | low >> 7) == 0 {
                                            PALLETTE[s.palette[0] as usize]
                                        } else {
                                            let b = s.palette[0x10
                                                | ((palette as usize) << 2)
                                                | (((high as usize) >> 7 << 1)
                                                    | low as usize >> 7)];
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

                        for yf in 0..8 {
                            for xc in 0..4 {
                                let byte = s.palette[(yc << 2) | xc];
                                let (r, g, b) = PALLETTE[byte as usize];
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
