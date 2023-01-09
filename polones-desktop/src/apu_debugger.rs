use crate::text_area::{Color::*, TextArea};
use crate::EmulatorState;
use polones_core::apu::{Pulse, Triangle};
use polones_core::nes::Nes;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;
use std::rc::Rc;

pub struct SdlApuDebugger {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    text_area: TextArea<{ Self::WIDTH as usize / 8 }, { Self::HEIGHT as usize / 8 }>,
    mode: u8,
    apu_state: ApuState,
}

struct ApuState {
    pulse: Pulse,
    triangle: Triangle,
}

impl Default for ApuState {
    fn default() -> Self {
        Self {
            pulse: Pulse::new_with_complement(),
            triangle: Triangle::default(),
        }
    }
}

impl SdlApuDebugger {
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
            mode: 1,
            apu_state: ApuState::default(),
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
                keycode: _k @ Some(Keycode::Num5),
                ..
            } => {
                self.mode = 5;
            }
            _ => {}
        }
    }

    pub fn update(&mut self, nes: &Nes) {
        match self.mode {
            1 => {
                self.apu_state.pulse = nes.apu.pulse1.clone();
            }
            2 => {
                self.apu_state.pulse = nes.apu.pulse2.clone();
            }
            3 => {
                self.apu_state.triangle = nes.apu.triangle.clone();
            }
            _ => {}
        }
    }

    pub fn draw(&mut self) {
        self.canvas.clear();
        self.text_area.clear();
        let ta = &mut self.text_area;

        fn draw_pulse_data<const W: usize, const H: usize>(pulse: &Pulse, ta: &mut TextArea<W, H>) {
            ta.write_str_with_color("EN DIVIDER PERIOD", 1, 0, Blue);
            ta.write_u8_with_color(pulse.envelope_divider_period, 1, 28, White);
            ta.write_str_with_color("EN DIVIDER COUNTER", 2, 0, Blue);
            ta.write_u8_with_color(pulse.envelope_divider_counter, 2, 28, White);
            ta.write_str_with_color("EN START FLAG", 3, 0, Blue);
            ta.write_bool_with_color(pulse.envelope_start_flag, 3, 29, White);
            ta.write_str_with_color("EN DECAY LEVEL COUNTER", 4, 0, Blue);
            ta.write_u8_with_color(pulse.envelope_decay_level_counter, 4, 28, White);
            ta.write_str_with_color("EN LOOP FLAG", 5, 0, Blue);
            ta.write_bool_with_color(pulse.sweep_enabled, 5, 29, White);
            ta.write_str_with_color("EN CONSTANT VOLUME FLAG", 6, 0, Blue);
            ta.write_bool_with_color(pulse.envelope_constant_volume_flag, 6, 29, White);

            ta.write_str_with_color("SWEEP ENABLED", 7, 0, Green);
            ta.write_bool_with_color(pulse.sweep_enabled, 7, 29, White);
            ta.write_str_with_color("SWEEP DIVIDER PERIOD", 8, 0, Green);
            ta.write_u8_with_color(pulse.sweep_divider_period, 8, 28, White);
            ta.write_str_with_color("SWEEP DIVIDER COUNTER", 9, 0, Green);
            ta.write_u8_with_color(pulse.sweep_divider_counter, 9, 28, White);
            ta.write_str_with_color("SWEEP NEGATE FLAG", 10, 0, Green);
            ta.write_bool_with_color(pulse.sweep_negate_flag, 10, 29, White);
            ta.write_str_with_color("SWEEP SHIFT COUNT", 11, 0, Green);
            ta.write_u8_with_color(pulse.sweep_shift_count, 11, 28, White);
            ta.write_str_with_color("SWEEP RELOAD FLAG", 12, 0, Green);
            ta.write_bool_with_color(pulse.sweep_reload_flag, 12, 29, White);

            ta.write_str_with_color("TIMER DIVIDER PERIOD", 13, 0, Yellow);
            ta.write_u16_with_color(pulse.timer_divider_period, 13, 26, White);
            ta.write_str_with_color("TIMER DIVIDER COUNTER", 14, 0, Yellow);
            ta.write_u16_with_color(pulse.timer_divider_counter, 14, 26, White);

            ta.write_str_with_color("SEQUENCER DUTY", 15, 0, Magenta);
            ta.write_u8_with_color(pulse.sequencer_duty, 15, 28, White);
            ta.write_str_with_color("SEQUENCER STEP", 16, 0, Magenta);
            ta.write_u8_with_color(pulse.sequencer_step, 16, 28, White);

            ta.write_str_with_color("LENGTH COUNTER", 17, 0, Cyan);
            ta.write_u8_with_color(pulse.length_counter, 17, 28, White);
            ta.write_str_with_color("LENGTH COUNTER HALT", 18, 0, Cyan);
            ta.write_bool_with_color(pulse.length_counter_halt, 18, 29, White);
            ta.write_str_with_color("LENGTH COUNTER EN", 19, 0, Cyan);
            ta.write_bool_with_color(pulse.length_counter_enabled, 19, 29, White);

            ta.write_str_with_color("EN", 21, 0, Blue);
            ta.write_str_with_color("SW", 21, 5, Green);
            ta.write_str_with_color("SE", 21, 10, Magenta);
            ta.write_str_with_color("LE", 21, 15, Cyan);

            ta.write_u8_with_color(pulse.volume(), 22, 0, White);
            ta.write_bool_with_color(!pulse.sweep_mutes_channel(), 22, 6, White);
            ta.write_bool_with_color(!pulse.sequencer_mutes_channel(), 22, 11, White);
            ta.write_bool_with_color(!pulse.length_counter_mutes_channel(), 22, 16, White);
        }

        fn draw_triangle_data<const W: usize, const H: usize>(
            triangle: &Triangle,
            ta: &mut TextArea<W, H>,
        ) {
            ta.write_str_with_color("TIMER", 1, 0, Blue);
            ta.write_u16_with_color(triangle.timer, 1, 26, White);
            ta.write_str_with_color("TIMER LOAD", 2, 0, Blue);
            ta.write_u16_with_color(triangle.timer_load, 2, 26, White);

            ta.write_str_with_color("LINEAR COUNTER", 3, 0, Green);
            ta.write_u8_with_color(triangle.linear_counter, 3, 28, White);
            ta.write_str_with_color("LINEAR COUNTER LOAD", 4, 0, Green);
            ta.write_u8_with_color(triangle.linear_counter_load, 4, 28, White);
            ta.write_str_with_color("LINEAR COUNTER RELOAD", 5, 0, Green);
            ta.write_bool_with_color(triangle.linear_counter_reload, 5, 29, White);

            ta.write_str_with_color("LENGTH COUNTER", 6, 0, Cyan);
            ta.write_u8_with_color(triangle.length_counter, 6, 28, White);
            ta.write_str_with_color("LENGTH COUNTER HALT", 7, 0, Cyan);
            ta.write_bool_with_color(triangle.length_counter_halt, 7, 29, White);
            ta.write_str_with_color("LENGTH COUNTER ENABLED", 8, 0, Cyan);
            ta.write_bool_with_color(triangle.length_counter_enabled, 8, 29, White);

            ta.write_str_with_color("SEQUENCER STEP", 9, 0, Magenta);
            ta.write_u8_with_color(triangle.sequencer_step, 9, 28, White);

            ta.write_str_with_color("LI", 11, 0, Green);
            ta.write_str_with_color("LE", 11, 5, Cyan);
            ta.write_str_with_color("SE", 11, 10, Magenta);

            ta.write_bool_with_color(triangle.linear_counter != 0, 14, 1, White);
            ta.write_bool_with_color(triangle.length_counter != 0, 14, 6, White);
            ta.write_u8_with_color(triangle.volume(), 14, 10, White);
        }

        match self.mode {
            1 => {
                ta.write_str_with_color("PULSE 1", 0, 0, Yellow);
                draw_pulse_data(&self.apu_state.pulse, ta);
            }
            2 => {
                ta.write_str_with_color("PULSE 2", 0, 0, Yellow);
                draw_pulse_data(&self.apu_state.pulse, ta);
            }
            3 => {
                ta.write_str_with_color("TRIANGLE", 0, 0, Yellow);
                draw_triangle_data(&self.apu_state.triangle, ta);
            }
            _ => {}
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
