use crate::text_area::{Color::*, TextArea};
use crate::EmulatorState;
use polones_core::apu::Pulse;
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
        }
    }

    pub fn show(&mut self, nes: &mut Nes) {
        let (_cpu, cpu_bus) = nes.split_into_cpu_and_bus();

        self.canvas.clear();
        self.text_area.clear();
        let ta = &mut self.text_area;
        let apu = cpu_bus.apu;

        fn draw_pulse_data<const W: usize, const H: usize>(pulse: &Pulse, ta: &mut TextArea<W, H>) {
            ta.write_str_with_color("ENABLED", 1, 0, Red);

            ta.write_bool_with_color(pulse.enabled, 1, 29, White);
            ta.write_str_with_color("EN DIVIDER PERIOD", 2, 0, Blue);
            ta.write_u8_with_color(pulse.envelope_divider_period, 2, 28, White);
            ta.write_str_with_color("EN DIVIDER COUNTER", 3, 0, Blue);
            ta.write_u8_with_color(pulse.envelope_divider_counter, 3, 28, White);
            ta.write_str_with_color("EN START FLAG", 4, 0, Blue);
            ta.write_bool_with_color(pulse.envelope_start_flag, 4, 29, White);
            ta.write_str_with_color("EN DECAY LEVEL COUNTER", 5, 0, Blue);
            ta.write_u8_with_color(pulse.envelope_decay_level_counter, 5, 28, White);
            ta.write_str_with_color("EN LOOP FLAG", 6, 0, Blue);
            ta.write_bool_with_color(pulse.sweep_enabled, 6, 29, White);
            ta.write_str_with_color("EN CONSTANT VOLUME FLAG", 7, 0, Blue);
            ta.write_bool_with_color(pulse.envelope_constant_volume_flag, 7, 29, White);

            ta.write_str_with_color("SW ENABLED", 8, 0, Green);
            ta.write_bool_with_color(pulse.sweep_enabled, 8, 29, White);
            ta.write_str_with_color("SW DIVIDER PERIOD", 9, 0, Green);
            ta.write_u8_with_color(pulse.sweep_divider_period, 9, 28, White);
            ta.write_str_with_color("SW DIVIDER COUNTER", 10, 0, Green);
            ta.write_u8_with_color(pulse.sweep_divider_counter, 10, 28, White);
            ta.write_str_with_color("SW NEGATE FLAG", 11, 0, Green);
            ta.write_bool_with_color(pulse.sweep_negate_flag, 11, 29, White);
            ta.write_str_with_color("SW SHIFT COUNT", 12, 0, Green);
            ta.write_u8_with_color(pulse.sweep_shift_count, 12, 28, White);
            ta.write_str_with_color("SW RELOAD FLAG", 13, 0, Green);
            ta.write_bool_with_color(pulse.sweep_reload_flag, 13, 29, White);

            ta.write_str_with_color("TI DIVIDER PERIOD", 14, 0, Yellow);
            ta.write_u16_with_color(pulse.timer_divider_period, 14, 26, White);
            ta.write_str_with_color("TI DIVIDER COUNTER", 15, 0, Yellow);
            ta.write_u16_with_color(pulse.timer_divider_counter, 15, 26, White);

            ta.write_str_with_color("SEQUENCER DUTY", 16, 0, Magenta);
            ta.write_u8_with_color(pulse.sequencer_duty, 16, 28, White);
            ta.write_str_with_color("SEQUENCER STEP", 17, 0, Magenta);
            ta.write_u8_with_color(pulse.sequencer_step, 17, 28, White);

            ta.write_str_with_color("LENGTH COUNTER", 18, 0, Cyan);
            ta.write_u8_with_color(pulse.length_counter, 18, 28, White);
            ta.write_str_with_color("LENGTH COUNTER HALT", 19, 0, Cyan);
            ta.write_bool_with_color(pulse.length_counter_halt, 19, 29, White);

            ta.write_str_with_color("EN", 21, 5, Blue);
            ta.write_str_with_color("SW", 21, 10, Green);
            ta.write_str_with_color("SE", 21, 25, Magenta);
            ta.write_str_with_color("LC", 21, 20, Cyan);

            ta.write_u8_with_color(pulse.volume(), 22, 5, White);
            ta.write_bool_with_color(!pulse.sweep_mutes_channel(), 22, 11, White);
            ta.write_bool_with_color(!pulse.sequencer_mutes_channel(), 22, 16, White);
            ta.write_bool_with_color(!pulse.length_counter_mutes_channel(), 22, 21, White);
        }

        match self.mode {
            1 => {
                ta.write_str_with_color("PULSE 1", 0, 0, Yellow);
                draw_pulse_data(&apu.pulse1, ta);
            }
            2 => {
                ta.write_str_with_color("PULSE 2", 0, 0, Yellow);
                draw_pulse_data(&apu.pulse2, ta);
            }
            _ => {},
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
        match event {
            Event::Quit { .. } => {
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
}
