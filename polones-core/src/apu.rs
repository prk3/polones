use crate::cpu::Cpu;
use crate::nes::Audio;

pub struct Pulse<const COMPLEMENT_EXTRA: u16> {
    enabled: bool,

    envelope_divider_period: u8,  // 4 bits
    envelope_divider_counter: u8, // 4 bits
    envelope_start_flag: bool,
    envelope_decay_level_counter: u8, // 4 bits
    envelope_loop_flag: bool,
    envelope_constant_volume_flag: bool,

    sweep_enabled: bool,
    sweep_divider_period: u8,  // 3 bits
    sweep_divider_counter: u8, // 3 bits
    sweep_negate_flag: bool,
    sweep_shift_count: u8, // 2 bits
    sweep_reload_flag: bool,

    timer_divider_period: u16,  // 11 bits
    timer_divider_counter: u16, // 11 bits

    sequencer_duty: u8, // 2 bit
    sequencer_step: u8, // 3 bits

    length_counter: u8, // 5 bits
    length_counter_halt: bool,
}

impl<const COMPLEMENT_EXTRA: u16> Pulse<COMPLEMENT_EXTRA> {
    const WAVEFORM: [[u8; 8]; 4] = [
        [0, 1, 0, 0, 0, 0, 0, 0],
        [0, 1, 1, 0, 0, 0, 0, 0],
        [0, 1, 1, 1, 1, 0, 0, 0],
        [1, 0, 0, 1, 1, 1, 1, 1],
    ];
    fn tick(&mut self) {
        if self.timer_divider_counter != 0 {
            self.timer_divider_counter -= 1;
        } else {
            self.timer_divider_counter = self.timer_divider_period;
            self.sequencer_step = (self.sequencer_step + 1) & 0b111;
        }
    }
    fn tick_envelope(&mut self) {
        if !self.envelope_start_flag {
            // clock divider
            if self.envelope_divider_counter > 0 {
                self.envelope_divider_counter -= 1;
            } else {
                self.envelope_divider_counter = self.envelope_divider_period;
                // clock decay level
                if self.envelope_decay_level_counter > 0 {
                    self.envelope_decay_level_counter -= 1;
                } else if self.envelope_loop_flag {
                    self.envelope_decay_level_counter = 15;
                }
            }
        } else {
            self.envelope_start_flag = false;
            self.envelope_decay_level_counter = 15;
            self.envelope_divider_counter = self.envelope_divider_period;
        }
    }
    fn tick_length_counter(&mut self) {
        if self.length_counter > 0 && !self.length_counter_halt {
            self.length_counter -= 1;
        }
    }
    fn tick_sweep(&mut self) {
        if self.sweep_divider_counter == 0 && self.sweep_enabled && !self.sweep_mutes_channel() {
            self.timer_divider_period = self.sweep_target_period();
        }

        if self.sweep_divider_counter == 0 || self.sweep_reload_flag {
            self.sweep_divider_counter = self.sweep_divider_period;
            self.sweep_reload_flag = false;
        } else {
            self.sweep_divider_counter -= 1;
        }
    }
    fn sweep_mutes_channel(&self) -> bool {
        self.sweep_divider_period < 8 || self.sweep_target_period() > 0x7FF
    }
    fn sweep_target_period(&self) -> u16 {
        let change: u16 = self.timer_divider_period >> self.sweep_shift_count;

        if self.sweep_negate_flag {
            self.timer_divider_period
                .wrapping_sub(change + COMPLEMENT_EXTRA)
        } else {
            self.timer_divider_period.wrapping_add(change)
        }
    }
    /// Returns volume
    fn volume(&self) -> u8 /* 0-15 */ {
        if self.envelope_constant_volume_flag {
            self.envelope_divider_period
        } else {
            self.envelope_decay_level_counter
        }
    }
    fn muted(&self) -> bool {
        Self::WAVEFORM[self.sequencer_duty as usize][self.sequencer_step as usize] == 0
            || self.sweep_mutes_channel()
            || self.length_counter == 0
            || self.timer_divider_period < 8
    }
}

pub struct Triangle {
    enabled: bool,
    length_counter: u16,
}

impl Triangle {
    fn tick_length_counter(&mut self) {}
    fn tick_linear_counter(&mut self) {}
}

pub struct Noise {
    enabled: bool,
    length_counter: u16,
}

impl Noise {
    fn tick_length_counter(&mut self) {}
    fn tick_envelope(&mut self) {}
}

pub struct Dmc {
    enabled: bool,
    interrupt: bool,
}

pub struct Apu {
    pulse1: Pulse<1>,
    pulse2: Pulse<0>,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    cpu_cycle_odd: bool,
    frame_counter_mode: bool,
    frame_counter_interrupt: bool,
    frame_counter_interrupt_inhibit: bool,
    frame_counter: u16,

    pulse1_samples: Vec<u8>,
    audio: Box<dyn Audio>,

    #[feature(draw_audio)]
    draw_audio_samples: Vec<u8>,
}

impl Apu {
    pub fn new(audio: Box<dyn Audio>) -> Self {
        Self {
            pulse1: Pulse {
                enabled: false,

                envelope_divider_period: 0,
                envelope_divider_counter: 0,
                envelope_decay_level_counter: 0,
                envelope_start_flag: true,
                envelope_loop_flag: false,
                envelope_constant_volume_flag: false,

                sweep_enabled: false,
                sweep_negate_flag: false,
                sweep_reload_flag: false,
                sweep_divider_period: 0,
                sweep_divider_counter: 0,
                sweep_shift_count: 0,

                timer_divider_period: 0,
                timer_divider_counter: 0,

                sequencer_step: 0,
                sequencer_duty: 0,

                length_counter: 0,
                length_counter_halt: false,
            },
            pulse2: Pulse {
                enabled: false,

                envelope_divider_period: 0,
                envelope_divider_counter: 0,
                envelope_decay_level_counter: 0,
                envelope_start_flag: true,
                envelope_loop_flag: false,
                envelope_constant_volume_flag: false,

                sweep_enabled: false,
                sweep_negate_flag: false,
                sweep_reload_flag: false,
                sweep_divider_period: 0,
                sweep_divider_counter: 0,
                sweep_shift_count: 0,

                timer_divider_period: 0,
                timer_divider_counter: 0,

                sequencer_step: 0,
                sequencer_duty: 0,

                length_counter: 0,
                length_counter_halt: false,
            },
            triangle: Triangle {
                enabled: false,
                length_counter: 0,
            },
            noise: Noise {
                enabled: false,
                length_counter: 0,
            },
            dmc: Dmc {
                enabled: false,
                interrupt: false,
            },
            cpu_cycle_odd: false,
            frame_counter_mode: false,
            frame_counter_interrupt: false,
            frame_counter_interrupt_inhibit: false,
            frame_counter: 0,

            pulse1_samples: Vec::with_capacity(11000), // CPU cycles in 256 audio cycles
            audio,

            #[feature(draw_audio)]
            draw_audio_samples: Vec::with_capacity(5 * 1_700_000),
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x4015 => {
                let result = (((self.pulse1.length_counter > 0) as u8) << 0) |
                    (((self.pulse2.length_counter > 0) as u8) << 1) |
                    (((self.triangle.length_counter > 0) as u8) << 2) |
                    (((self.noise.length_counter > 0) as u8) << 3) |
                    // (((self.dmc.bytes.len() > 0) as u8) << 4) | // TODO
                    ((self.frame_counter_interrupt as u8) << 6) |
                    ((self.dmc.interrupt as u8) << 7);

                self.frame_counter_interrupt = false;
                result
            }
            _ => {
                eprintln!("Apu: read from address {address:04X}");
                0
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if (0x4000..=0x4003).contains(&address) {
        println!("address: {address:04X}, value: {value:02X}");
        }
        match address {
            0x4000 => {
                self.pulse1.sequencer_duty = value >> 6;
                self.pulse1.envelope_loop_flag = (value >> 5) & 1 == 1;
                self.pulse1.length_counter_halt = self.pulse1.envelope_loop_flag;
                self.pulse1.envelope_constant_volume_flag = (value >> 4) & 1 == 1;
                self.pulse1.envelope_divider_period = value & 0b1111;
            }
            0x4001 => {
                self.pulse1.sweep_enabled = (value & 0b10000000) > 0;
                self.pulse1.sweep_divider_period = ((value >> 4) & 0b111) + 1;
                self.pulse1.sweep_negate_flag = (value & 0b1000) > 0;
                self.pulse1.sweep_shift_count = value & 0b111;
                self.pulse1.sweep_reload_flag = true;
            }
            0x4002 => {
                self.pulse1.timer_divider_period &= 0xFF00;
                self.pulse1.timer_divider_period |= value as u16;
            }
            0x4003 => {
                self.pulse1.timer_divider_period &= 0x00FF;
                self.pulse1.timer_divider_period |= (value as u16 & 0b111) << 8;
                self.pulse1.length_counter = value >> 3;
                self.pulse1.sequencer_step = 0;
                self.pulse1.envelope_start_flag = true;
            }
            0x4004 => {
                self.pulse2.sequencer_duty = value >> 6;
                self.pulse2.envelope_loop_flag = (value >> 5) & 1 == 1;
                self.pulse2.length_counter_halt = self.pulse2.envelope_loop_flag;
                self.pulse2.envelope_constant_volume_flag = (value >> 4) & 1 == 1;
                self.pulse2.envelope_divider_period = value & 0b1111;
            }
            0x4005 => {
                self.pulse2.sweep_enabled = (value & 0b10000000) > 0;
                self.pulse2.sweep_divider_period = (value >> 4) & 0b111;
                self.pulse2.sweep_negate_flag = (value & 0b1000) > 0;
                self.pulse2.sweep_shift_count = value & 0b111;
                self.pulse2.sweep_reload_flag = true;
            }
            0x4006 => {
                self.pulse2.timer_divider_period &= 0xFF00;
                self.pulse2.timer_divider_period |= value as u16;
            }
            0x4007 => {
                self.pulse2.timer_divider_period &= 0x00FF;
                self.pulse2.timer_divider_period |= (value as u16 & 0b111) << 8;
                self.pulse2.length_counter = value >> 3;
                self.pulse2.sequencer_step = 0;
                self.pulse2.envelope_start_flag = true;
            }
            0x4015 => {
                self.pulse1.enabled = (value & 1) > 0;
                if !self.pulse1.enabled {
                    self.pulse1.length_counter = 0;
                }
                self.pulse2.enabled = (value & 2) > 0;
                if !self.pulse2.enabled {
                    self.pulse2.length_counter = 0;
                }
                self.triangle.enabled = (value & 4) > 0;
                if !self.triangle.enabled {
                    self.triangle.length_counter = 0;
                }
                self.noise.enabled = (value & 8) > 0;
                if !self.noise.enabled {
                    self.noise.length_counter = 0;
                }
                self.dmc.enabled = (value & 16) > 0;
                self.dmc.interrupt = false;
                // TODO update other dmc fields
            }
            0x4017 => {
                self.frame_counter_mode = (value & 0b10000000) > 0;
                self.frame_counter_interrupt_inhibit = (value & 0b01000000) > 0;
                if self.frame_counter_interrupt_inhibit {
                    self.frame_counter_interrupt = false;
                }
                self.frame_counter = 0;
            }
            _ => {
                eprintln!("Apu: write to address {address:04X}")
            }
        }
    }

    pub fn tick(&mut self, cpu: &mut Cpu) {
        if self.frame_counter_mode {
            // 5 step
            match self.frame_counter {
                7457 => {
                    self.pulse1.tick_envelope();
                    self.pulse2.tick_envelope();
                    self.triangle.tick_linear_counter();
                    self.noise.tick_envelope();
                    self.frame_counter += 1;
                }
                14913 => {
                    self.pulse1.tick_envelope();
                    self.pulse1.tick_length_counter();
                    self.pulse1.tick_sweep();
                    self.pulse2.tick_envelope();
                    self.pulse2.tick_length_counter();
                    self.pulse2.tick_sweep();
                    self.triangle.tick_linear_counter();
                    self.triangle.tick_length_counter();
                    self.noise.tick_envelope();
                    self.noise.tick_length_counter();
                    self.frame_counter += 1;
                }
                22371 => {
                    self.pulse1.tick_envelope();
                    self.pulse2.tick_envelope();
                    self.triangle.tick_linear_counter();
                    self.noise.tick_envelope();
                    self.frame_counter += 1;
                }
                37281 => {
                    self.pulse1.tick_envelope();
                    self.pulse1.tick_length_counter();
                    self.pulse1.tick_sweep();
                    self.pulse2.tick_envelope();
                    self.pulse2.tick_length_counter();
                    self.pulse2.tick_sweep();
                    self.triangle.tick_linear_counter();
                    self.triangle.tick_length_counter();
                    self.noise.tick_envelope();
                    self.noise.tick_length_counter();
                    self.frame_counter = 0;
                }
                _ => {
                    self.frame_counter += 1;
                }
            }
        } else {
            // 4 step
            match self.frame_counter {
                7457 => {
                    self.pulse1.tick_envelope();
                    self.pulse2.tick_envelope();
                    self.triangle.tick_linear_counter();
                    self.noise.tick_envelope();
                    self.frame_counter += 1;
                }
                14913 => {
                    self.pulse1.tick_envelope();
                    self.pulse1.tick_length_counter();
                    self.pulse1.tick_sweep();
                    self.pulse2.tick_envelope();
                    self.pulse2.tick_length_counter();
                    self.pulse2.tick_sweep();
                    self.triangle.tick_linear_counter();
                    self.triangle.tick_length_counter();
                    self.noise.tick_envelope();
                    self.noise.tick_length_counter();
                    self.frame_counter += 1;
                }
                22371 => {
                    self.pulse1.tick_envelope();
                    self.pulse2.tick_envelope();
                    self.triangle.tick_linear_counter();
                    self.noise.tick_envelope();
                    self.frame_counter += 1;
                }
                29828 => {
                    if !self.frame_counter_interrupt_inhibit {
                        // cpu.irq();
                        self.frame_counter_interrupt = true;
                    }
                    self.frame_counter += 1;
                }
                29829 => {
                    self.pulse1.tick_envelope();
                    self.pulse1.tick_length_counter();
                    self.pulse1.tick_sweep();
                    self.pulse2.tick_envelope();
                    self.pulse2.tick_length_counter();
                    self.pulse2.tick_sweep();
                    self.triangle.tick_linear_counter();
                    self.triangle.tick_length_counter();
                    self.noise.tick_envelope();
                    self.noise.tick_length_counter();
                    self.frame_counter = 0;
                }
                _ => {
                    self.frame_counter += 1;
                }
            }
        }

        // do some more stuff here
        if !self.cpu_cycle_odd {
            self.pulse1.tick();
            self.pulse2.tick();
        }

        let sample = !self.pulse1.muted() as u8 * self.pulse1.volume();
        self.pulse1_samples.push(sample);

        #[feature(draw_audio)]
        self.draw_audio_samples.push(sample);

        if self.pulse1_samples.len() == (64.0f64 * (1_789_773.0 / 44_100.0)).round() as usize { // 64 * (CPU freq / audio freq)
            let mut output_samples = [0u16; 64];
            (0..64).for_each(|i| {
                output_samples[i] = self.pulse1_samples[(i as f64 * (1_789_773.0 / 44_100.0)).round() as usize] as u16 * 1024;
            });
            self.pulse1_samples.clear();
            self.audio.play(output_samples);
        }

        self.cpu_cycle_odd = !self.cpu_cycle_odd;
    }
}

#[feature(draw_audio)]
impl Drop for Apu {
    fn drop(&mut self) {
        let result = (|| {
            use std::io::Write;
            use std::io::BufWriter;

            let f = std::fs::File::create("./audio.pgm")?;
            let mut f = BufWriter::with_capacity(1_000_000, f);
            f.write(format!("P5 {} 15 255\n", self.draw_audio_samples.len()).as_bytes())?;
            for i in (1..=15).rev() {
                for s in &self.draw_audio_samples {
                    f.write(&[if *s >= i { 0 } else { 255 }])?;
                }
            }

            Result::<(), std::io::Error>::Ok(())
        })();
        println!("draw audio result: {result:?}");
    }
}
