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
    apu_state: ApuState,
}

#[derive(Default)]
struct ApuState {
    pulse_enabled: bool,
    pulse_envelope_divider_period: u8,
    pulse_envelope_divider_counter: u8,
    pulse_envelope_start_flag: bool,
    pulse_envelope_decay_level_counter: u8,
    pulse_sweep_enabled: bool,
    pulse_envelope_constant_volume_flag: bool,

    pulse_sweep_divider_period: u8,
    pulse_sweep_divider_counter: u8,
    pulse_sweep_negate_flag: bool,
    pulse_sweep_shift_count: u8,
    pulse_sweep_reload_flag: bool,

    pulse_timer_divider_period: u16,
    pulse_timer_divider_counter: u16,

    pulse_sequencer_duty: u8,
    pulse_sequencer_step: u8,

    pulse_length_counter: u8,
    pulse_length_counter_halt: bool,

    pulse_volume: u8,
    pulse_sweep_mutes_channel: bool,
    pulse_sequencer_mutes_channel: bool,
    pulse_length_counter_mutes_channel: bool,
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
        fn update_pulse_data(apu_state: &mut ApuState, pulse: &Pulse) {
            apu_state.pulse_enabled = pulse.enabled;
            apu_state.pulse_envelope_divider_period = pulse.envelope_divider_period;
            apu_state.pulse_envelope_divider_counter = pulse.envelope_divider_counter;
            apu_state.pulse_envelope_start_flag = pulse.envelope_start_flag;
            apu_state.pulse_envelope_decay_level_counter = pulse.envelope_decay_level_counter;
            apu_state.pulse_sweep_enabled = pulse.sweep_enabled;
            apu_state.pulse_envelope_constant_volume_flag = pulse.envelope_constant_volume_flag;

            apu_state.pulse_sweep_divider_period = pulse.sweep_divider_period;
            apu_state.pulse_sweep_divider_counter = pulse.sweep_divider_counter;
            apu_state.pulse_sweep_negate_flag = pulse.sweep_negate_flag;
            apu_state.pulse_sweep_shift_count = pulse.sweep_shift_count;
            apu_state.pulse_sweep_reload_flag = pulse.sweep_reload_flag;

            apu_state.pulse_timer_divider_period = pulse.timer_divider_period;
            apu_state.pulse_timer_divider_counter = pulse.timer_divider_counter;

            apu_state.pulse_sequencer_duty = pulse.sequencer_duty;
            apu_state.pulse_sequencer_step = pulse.sequencer_step;

            apu_state.pulse_length_counter = pulse.length_counter;
            apu_state.pulse_length_counter_halt = pulse.length_counter_halt;

            apu_state.pulse_volume = pulse.volume();
            apu_state.pulse_sweep_mutes_channel = pulse.sweep_mutes_channel();
            apu_state.pulse_sequencer_mutes_channel = pulse.sequencer_mutes_channel();
            apu_state.pulse_length_counter_mutes_channel = pulse.length_counter_mutes_channel();
        }

        match self.mode {
            1 => {
                update_pulse_data(&mut self.apu_state, &nes.apu.pulse1);
            }
            2 => {
                update_pulse_data(&mut self.apu_state, &nes.apu.pulse2);
            }
            _ => {}
        }
    }

    pub fn draw(&mut self) {
        self.canvas.clear();
        self.text_area.clear();
        let ta = &mut self.text_area;

        fn draw_pulse_data<const W: usize, const H: usize>(
            apu_state: &ApuState,
            ta: &mut TextArea<W, H>,
        ) {
            ta.write_str_with_color("ENABLED", 1, 0, Red);
            ta.write_bool_with_color(apu_state.pulse_enabled, 1, 29, White);
            ta.write_str_with_color("EN DIVIDER PERIOD", 2, 0, Blue);
            ta.write_u8_with_color(apu_state.pulse_envelope_divider_period, 2, 28, White);
            ta.write_str_with_color("EN DIVIDER COUNTER", 3, 0, Blue);
            ta.write_u8_with_color(apu_state.pulse_envelope_divider_counter, 3, 28, White);
            ta.write_str_with_color("EN START FLAG", 4, 0, Blue);
            ta.write_bool_with_color(apu_state.pulse_envelope_start_flag, 4, 29, White);
            ta.write_str_with_color("EN DECAY LEVEL COUNTER", 5, 0, Blue);
            ta.write_u8_with_color(apu_state.pulse_envelope_decay_level_counter, 5, 28, White);
            ta.write_str_with_color("EN LOOP FLAG", 6, 0, Blue);
            ta.write_bool_with_color(apu_state.pulse_sweep_enabled, 6, 29, White);
            ta.write_str_with_color("EN CONSTANT VOLUME FLAG", 7, 0, Blue);
            ta.write_bool_with_color(apu_state.pulse_envelope_constant_volume_flag, 7, 29, White);

            ta.write_str_with_color("SW ENABLED", 8, 0, Green);
            ta.write_bool_with_color(apu_state.pulse_sweep_enabled, 8, 29, White);
            ta.write_str_with_color("SW DIVIDER PERIOD", 9, 0, Green);
            ta.write_u8_with_color(apu_state.pulse_sweep_divider_period, 9, 28, White);
            ta.write_str_with_color("SW DIVIDER COUNTER", 10, 0, Green);
            ta.write_u8_with_color(apu_state.pulse_sweep_divider_counter, 10, 28, White);
            ta.write_str_with_color("SW NEGATE FLAG", 11, 0, Green);
            ta.write_bool_with_color(apu_state.pulse_sweep_negate_flag, 11, 29, White);
            ta.write_str_with_color("SW SHIFT COUNT", 12, 0, Green);
            ta.write_u8_with_color(apu_state.pulse_sweep_shift_count, 12, 28, White);
            ta.write_str_with_color("SW RELOAD FLAG", 13, 0, Green);
            ta.write_bool_with_color(apu_state.pulse_sweep_reload_flag, 13, 29, White);

            ta.write_str_with_color("TI DIVIDER PERIOD", 14, 0, Yellow);
            ta.write_u16_with_color(apu_state.pulse_timer_divider_period, 14, 26, White);
            ta.write_str_with_color("TI DIVIDER COUNTER", 15, 0, Yellow);
            ta.write_u16_with_color(apu_state.pulse_timer_divider_counter, 15, 26, White);

            ta.write_str_with_color("SEQUENCER DUTY", 16, 0, Magenta);
            ta.write_u8_with_color(apu_state.pulse_sequencer_duty, 16, 28, White);
            ta.write_str_with_color("SEQUENCER STEP", 17, 0, Magenta);
            ta.write_u8_with_color(apu_state.pulse_sequencer_step, 17, 28, White);

            ta.write_str_with_color("LENGTH COUNTER", 18, 0, Cyan);
            ta.write_u8_with_color(apu_state.pulse_length_counter, 18, 28, White);
            ta.write_str_with_color("LENGTH COUNTER HALT", 19, 0, Cyan);
            ta.write_bool_with_color(apu_state.pulse_length_counter_halt, 19, 29, White);

            ta.write_str_with_color("EN", 21, 5, Blue);
            ta.write_str_with_color("SW", 21, 10, Green);
            ta.write_str_with_color("SE", 21, 25, Magenta);
            ta.write_str_with_color("LC", 21, 20, Cyan);

            ta.write_u8_with_color(apu_state.pulse_volume, 22, 5, White);
            ta.write_bool_with_color(!apu_state.pulse_sweep_mutes_channel, 22, 11, White);
            ta.write_bool_with_color(!apu_state.pulse_sequencer_mutes_channel, 22, 16, White);
            ta.write_bool_with_color(!apu_state.pulse_length_counter_mutes_channel, 22, 21, White);
        }

        match self.mode {
            1 => {
                ta.write_str_with_color("PULSE 1", 0, 0, Yellow);
                draw_pulse_data(&self.apu_state, ta);
            }
            2 => {
                ta.write_str_with_color("PULSE 2", 0, 0, Yellow);
                draw_pulse_data(&self.apu_state, ta);
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
