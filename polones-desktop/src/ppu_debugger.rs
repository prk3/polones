use crate::text_area::{Color::*, TextArea};
use crate::EmulatorState;
use polones_core::nes::Nes;
use polones_core::ppu::{ControlRegister, Loopy, MaskRegister, StatusRegister};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;
use std::rc::Rc;

pub struct SdlPpuDebugger {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    text_area: TextArea<{ Self::WIDTH as usize / 8 }, { Self::HEIGHT as usize / 8 }>,
    ppu_state: PpuState,
}

#[derive(Default)]
struct PpuState {
    ppu_scanline: u16,
    ppu_dot: u16,
    ppu_horizontal_scroll: u8,
    ppu_vertical_scroll: u8,
    ppu_control_register: ControlRegister,
    ppu_mask_register: MaskRegister,
    ppu_status_register: StatusRegister,
    ppu_oam_address: u8,
    ppu_t: Loopy,
    ppu_w: bool,
}

impl SdlPpuDebugger {
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
            ppu_state: PpuState::default(),
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
            _ => {}
        }
    }

    pub fn update(&mut self, nes: &Nes) {
        self.ppu_state.ppu_scanline = nes.ppu.scanline;
        self.ppu_state.ppu_dot = nes.ppu.dot;
        self.ppu_state.ppu_horizontal_scroll = nes.ppu.horizontal_scroll;
        self.ppu_state.ppu_vertical_scroll = nes.ppu.vertical_scroll;
        self.ppu_state.ppu_control_register = nes.ppu.control_register;
        self.ppu_state.ppu_mask_register = nes.ppu.mask_register;
        self.ppu_state.ppu_status_register = nes.ppu.status_register;
        self.ppu_state.ppu_oam_address = nes.ppu.oam_address;
        self.ppu_state.ppu_t = nes.ppu.t;
        self.ppu_state.ppu_w = nes.ppu.w;
    }

    pub fn draw(&mut self) {
        self.canvas.clear();
        self.text_area.clear();
        let ta = &mut self.text_area;
        let s = &self.ppu_state;

        ta.write_str_with_color("SCANLINE", 0, 0, Yellow);
        ta.write_u16_with_color(s.ppu_scanline, 0, 9, White);

        ta.write_str_with_color("DOT", 1, 5, Yellow);
        ta.write_u16_with_color(s.ppu_dot, 1, 9, White);

        ta.write_str_with_color("SCROLL", 0, 14, Yellow);
        ta.write_str_with_color("H", 0, 21, Yellow);
        ta.write_u8_with_color(
            s.ppu_horizontal_scroll,
            0,
            23,
            if s.ppu_w { White } else { Magenta },
        );
        ta.write_str_with_color("V", 1, 21, Yellow);
        ta.write_u8_with_color(
            s.ppu_vertical_scroll,
            1,
            23,
            if s.ppu_w { Magenta } else { White },
        );

        ta.write_str_with_color("CTRL", 3, 2, Yellow);

        ta.write_str_with_color("NMI", 4, 3, Yellow);
        ta.write_bool_with_color(s.ppu_control_register.get_nmi_enable(), 4, 7, White);

        ta.write_str_with_color("M/S", 5, 3, Yellow);
        ta.write_bool_with_color(s.ppu_control_register.get_ppu_master_slave(), 5, 7, White);

        ta.write_str_with_color("HEIGHT", 6, 0, Yellow);
        ta.write_bool_with_color(s.ppu_control_register.get_sprite_height(), 6, 7, White);

        ta.write_str_with_color("BACK", 7, 2, Yellow);
        ta.write_bool_with_color(
            s.ppu_control_register.get_background_tile_select(),
            7,
            7,
            White,
        );

        ta.write_str_with_color("SPRITE", 8, 0, Yellow);
        ta.write_bool_with_color(s.ppu_control_register.get_sprite_tile_select(), 8, 7, White);

        ta.write_str_with_color("INC", 9, 3, Yellow);
        ta.write_bool_with_color(s.ppu_control_register.get_increment_mode(), 9, 7, White);

        ta.write_str_with_color("NTADDR", 10, 0, Yellow);
        ta.write_u8_with_color(
            s.ppu_control_register.get_name_table_address(),
            10,
            7,
            White,
        );

        ta.write_str_with_color("MASK", 3, 11, Yellow);

        ta.write_str_with_color("BLUE", 4, 11, Yellow);
        ta.write_bool_with_color(s.ppu_mask_register.get_emphasize_blue(), 4, 16, White);

        ta.write_str_with_color("GREEN", 5, 10, Yellow);
        ta.write_bool_with_color(s.ppu_mask_register.get_emphasize_green(), 5, 16, White);

        ta.write_str_with_color("RED", 6, 12, Yellow);
        ta.write_bool_with_color(s.ppu_mask_register.get_emphasize_red(), 6, 16, White);

        ta.write_str_with_color("SPR", 7, 12, Yellow);
        ta.write_bool_with_color(s.ppu_mask_register.get_show_sprites(), 7, 16, White);

        ta.write_str_with_color("BAC", 8, 12, Yellow);
        ta.write_bool_with_color(s.ppu_mask_register.get_show_background(), 8, 16, White);

        ta.write_str_with_color("SPRL", 9, 11, Yellow);
        ta.write_bool_with_color(
            s.ppu_mask_register.get_show_sprites_in_leftmost_col(),
            9,
            16,
            White,
        );

        ta.write_str_with_color("BACL", 10, 11, Yellow);
        ta.write_bool_with_color(
            s.ppu_mask_register.get_show_background_in_leftmost_col(),
            10,
            16,
            White,
        );

        ta.write_str_with_color("STATUS", 3, 18, Yellow);

        ta.write_str_with_color("VBLANK", 4, 18, Yellow);
        ta.write_bool_with_color(s.ppu_status_register.get_vblank_flag(), 4, 25, White);

        ta.write_str_with_color("S0 HIT", 5, 18, Yellow);
        ta.write_bool_with_color(s.ppu_status_register.get_sprite_0_hit_flag(), 5, 25, White);

        ta.write_str_with_color("S OVER", 6, 18, Yellow);
        ta.write_bool_with_color(
            s.ppu_status_register.get_sprite_overflow_flag(),
            6,
            25,
            White,
        );

        ta.write_str_with_color("OAM ADDR", 8, 18, Yellow);
        ta.write_u8_with_color(s.ppu_oam_address, 8, 27, White);

        ta.write_str_with_color("PPU ADDR", 9, 18, Yellow);
        ta.write_u8_with_color(
            (s.ppu_t.get_ppu_address() >> 8) as u8,
            9,
            27,
            if s.ppu_w { White } else { Magenta },
        );
        ta.write_u8_with_color(
            s.ppu_t.get_ppu_address() as u8,
            9,
            29,
            if s.ppu_w { Magenta } else { White },
        );

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
