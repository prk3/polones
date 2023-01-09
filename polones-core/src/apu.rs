use crate::cpu::Cpu;
use crate::nes::{Peripherals, Samples};

const LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const PULSE_MIX_TABLE: [u16; 32] = [
    0, 749, 1479, 2193, 2889, 3569, 4234, 4884, 5519, 6140, 6748, 7342, 7924, 8493, 9050, 9596,
    10131, 10654, 11168, 11670, 12163, 12647, 13121, 13585, 14041, 14489, 14928, 15359, 15782,
    16197, 16605, 17006,
];

const TRIANGLE_SEQUENCE_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

const NOISE_TIMER_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

const OTHER_MIX_TABLE: [u16; 204] = [
    0, 432, 861, 1286, 1707, 2125, 2540, 2952, 3360, 3765, 4167, 4565, 4961, 5353, 5743, 6129,
    6513, 6893, 7271, 7645, 8017, 8386, 8752, 9116, 9477, 9835, 10190, 10543, 10893, 11241, 11586,
    11928, 12268, 12606, 12941, 13274, 13604, 13932, 14258, 14581, 14902, 15221, 15538, 15852,
    16164, 16474, 16782, 17088, 17391, 17693, 17993, 18290, 18586, 18879, 19171, 19460, 19748,
    20033, 20317, 20599, 20879, 21157, 21434, 21708, 21981, 22252, 22522, 22789, 23055, 23319,
    23582, 23842, 24101, 24359, 24615, 24869, 25122, 25373, 25622, 25870, 26117, 26362, 26605,
    26847, 27087, 27326, 27564, 27800, 28035, 28268, 28500, 28730, 28959, 29187, 29413, 29638,
    29862, 30085, 30306, 30525, 30744, 30961, 31177, 31392, 31605, 31818, 32029, 32239, 32447,
    32655, 32861, 33066, 33270, 33473, 33675, 33875, 34075, 34273, 34470, 34667, 34862, 35056,
    35249, 35441, 35631, 35821, 36010, 36198, 36385, 36570, 36755, 36939, 37122, 37304, 37484,
    37664, 37843, 38021, 38198, 38375, 38550, 38724, 38897, 39070, 39242, 39412, 39582, 39751,
    39919, 40087, 40253, 40419, 40583, 40747, 40910, 41073, 41234, 41395, 41555, 41714, 41872,
    42029, 42186, 42342, 42497, 42652, 42805, 42958, 43110, 43262, 43413, 43562, 43712, 43860,
    44008, 44155, 44302, 44447, 44592, 44737, 44880, 45023, 45166, 45307, 45448, 45588, 45728,
    45867, 46005, 46143, 46280, 46417, 46553, 46688, 46822, 46956, 47090, 47222, 47355, 47486,
    47617, 47747, 47877, 48006,
];

#[derive(Clone)]
pub struct Pulse {
    pub envelope_divider_period: u8,  // 4 bits
    pub envelope_divider_counter: u8, // 4 bits
    pub envelope_start_flag: bool,
    pub envelope_decay_level_counter: u8, // 4 bits
    pub envelope_loop_flag: bool,
    pub envelope_constant_volume_flag: bool,

    pub sweep_enabled: bool,
    pub sweep_divider_period: u8,  // 3/4 bits
    pub sweep_divider_counter: u8, // 3/4 bits
    pub sweep_negate_flag: bool,
    pub sweep_shift_count: u8, // 2 bits
    pub sweep_reload_flag: bool,

    pub timer_divider_period: u16,  // 11 bits
    pub timer_divider_counter: u16, // 11 bits

    pub sequencer_duty: u8, // 2 bit
    pub sequencer_step: u8, // 3 bits

    pub length_counter: u8, // 5 bits
    pub length_counter_halt: bool,
    pub length_counter_enabled: bool,

    pub complement_extra: u16,
}

impl Pulse {
    const WAVEFORM: [[u8; 8]; 4] = [
        [0, 1, 0, 0, 0, 0, 0, 0],
        [0, 1, 1, 0, 0, 0, 0, 0],
        [0, 1, 1, 1, 1, 0, 0, 0],
        [1, 0, 0, 1, 1, 1, 1, 1],
    ];
    pub fn new_with_complement() -> Self {
        Self {
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
            length_counter_enabled: false,

            complement_extra: 1,
        }
    }
    pub fn new_without_complement() -> Self {
        Self {
            complement_extra: 0,
            ..Self::new_with_complement()
        }
    }
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
    pub fn sweep_mutes_channel(&self) -> bool {
        self.timer_divider_period < 8 || self.sweep_target_period() > 0x7FF
    }
    pub fn sequencer_mutes_channel(&self) -> bool {
        Self::WAVEFORM[self.sequencer_duty as usize][self.sequencer_step as usize] == 0
    }
    pub fn length_counter_mutes_channel(&self) -> bool {
        self.length_counter == 0
    }
    fn sweep_target_period(&self) -> u16 {
        let change: u16 = self.timer_divider_period >> self.sweep_shift_count;

        let new = if self.sweep_negate_flag {
            self.timer_divider_period
                .wrapping_sub(change + self.complement_extra)
        } else {
            self.timer_divider_period.wrapping_add(change)
        };

        new & 0b111_11111111
    }
    /// Returns volume
    pub fn volume(&self) -> u8 /* 0-15 */ {
        if self.envelope_constant_volume_flag {
            self.envelope_divider_period
        } else {
            self.envelope_decay_level_counter
        }
    }
    fn muted(&self) -> bool {
        self.sequencer_mutes_channel()
            || self.sweep_mutes_channel()
            || self.length_counter_mutes_channel()
    }
}

#[derive(Default, Clone)]
pub struct Triangle {
    pub enabled: bool,
    pub timer: u16,
    pub timer_load: u16,
    pub linear_counter: u8,
    pub linear_counter_load: u8,
    pub linear_counter_reload: bool,
    pub length_counter: u8,
    pub length_counter_halt: bool,
    pub length_counter_enabled: bool,
    pub sequencer_step: u8,
    pub debug_v: u8,
    pub debug_c: u32,
}

impl Triangle {
    fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            self.timer = self.timer_load;
            if self.linear_counter != 0 && self.length_counter != 0 {
                self.sequencer_step = (self.sequencer_step + 1) & 0b11111;
            }
        }
    }
    fn tick_length_counter(&mut self) {
        if self.length_counter > 0 && !self.length_counter_halt {
            self.length_counter -= 1;
        }
    }
    fn tick_linear_counter(&mut self) {
        if self.linear_counter_reload {
            self.linear_counter = self.linear_counter_load;
        } else {
            self.linear_counter = self.linear_counter.saturating_sub(1);
        }
        if !self.length_counter_halt {
            self.linear_counter_reload = false;
        }
    }
    pub fn volume(&self) -> u8 /* 0-15 */ {
        TRIANGLE_SEQUENCE_TABLE[self.sequencer_step as usize]
    }
}

pub struct Noise {
    pub envelope_divider_period: u8,  // 4 bits
    pub envelope_divider_counter: u8, // 4 bits
    pub envelope_start_flag: bool,
    pub envelope_decay_level_counter: u8, // 4 bits
    pub envelope_loop_flag: bool,
    pub envelope_constant_volume_flag: bool,

    pub length_counter: u8,
    pub length_counter_halt: bool,
    pub length_counter_enabled: bool,

    pub mode: bool,
    pub timer: u16,
    pub timer_load: u16,

    pub linear_feedback_shift_register: u16,
}

impl Noise {
    fn tick(&mut self) {
        let feedback = ((self.linear_feedback_shift_register >> ((self.mode as u8) * 5) >> 1)
            ^ self.linear_feedback_shift_register)
            & 1;
        self.linear_feedback_shift_register >>= 1;
        self.linear_feedback_shift_register |= feedback << 14;
    }
    fn tick_length_counter(&mut self) {
        if self.length_counter > 0 && !self.length_counter_halt {
            self.length_counter -= 1;
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
    fn volume(&self) -> u8 /* 0-15 */ {
        if self.linear_feedback_shift_register & 1 == 1 {
            return 0;
        }
        if self.length_counter == 0 {
            return 0;
        }

        if self.envelope_constant_volume_flag {
            self.envelope_divider_period
        } else {
            self.envelope_decay_level_counter
        }
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            envelope_divider_period: 0,
            envelope_divider_counter: 0,
            envelope_decay_level_counter: 0,
            envelope_start_flag: true,
            envelope_loop_flag: false,
            envelope_constant_volume_flag: false,

            length_counter: 0,
            length_counter_halt: false,
            length_counter_enabled: false,

            mode: false,
            timer: 0,
            timer_load: 0,

            linear_feedback_shift_register: 1,
        }
    }
}

pub struct Dmc {
    enabled: bool,
    interrupt: bool,
}

pub struct Apu {
    pub pulse1: Pulse,
    pub pulse2: Pulse,
    pub triangle: Triangle,
    pub noise: Noise,
    pub dmc: Dmc,
    pub cpu_cycle_odd: bool,
    pub frame_counter_mode: bool,
    pub frame_counter_interrupt: bool,
    pub frame_counter_interrupt_inhibit: bool,
    pub frame_counter: u16,

    pulse1_samples: Vec<u8>,
    pulse2_samples: Vec<u8>,
    triangle_samples: Vec<u8>,
    noise_samples: Vec<u8>,
    output_samples: Box<Samples>,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            pulse1: Pulse::new_with_complement(),
            pulse2: Pulse::new_without_complement(),
            triangle: Triangle::default(),
            noise: Noise::default(),
            dmc: Dmc {
                enabled: false,
                interrupt: false,
            },
            cpu_cycle_odd: false,
            frame_counter_mode: false,
            frame_counter_interrupt: false,
            frame_counter_interrupt_inhibit: false,
            frame_counter: 0,

            pulse1_samples: Vec::with_capacity(2900),
            pulse2_samples: Vec::with_capacity(2900),
            triangle_samples: Vec::with_capacity(2900),
            noise_samples: Vec::with_capacity(2900),
            output_samples: Box::new([0; 64]),
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
        match address {
            0x4000 => {
                self.pulse1.sequencer_duty = value >> 6;
                self.pulse1.envelope_loop_flag = (value & 0b100000) > 0;
                self.pulse1.length_counter_halt = self.pulse1.envelope_loop_flag;
                self.pulse1.envelope_constant_volume_flag = (value & 0b10000) > 0;
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
                if self.pulse1.length_counter_enabled {
                    self.pulse1.length_counter = LENGTH_COUNTER_TABLE[(value >> 3) as usize];
                }
                self.pulse1.sequencer_step = 0;
                self.pulse1.envelope_start_flag = true;
            }
            0x4004 => {
                self.pulse2.sequencer_duty = value >> 6;
                self.pulse2.envelope_loop_flag = (value & 0b100000) > 0;
                self.pulse2.length_counter_halt = self.pulse2.envelope_loop_flag;
                self.pulse2.envelope_constant_volume_flag = (value & 0b10000) > 0;
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
                if self.pulse2.length_counter_enabled {
                    self.pulse2.length_counter = LENGTH_COUNTER_TABLE[(value >> 3) as usize];
                }
                self.pulse2.sequencer_step = 0;
                self.pulse2.envelope_start_flag = true;
            }
            0x4008 => {
                self.triangle.length_counter_halt = value & 0b10000000 > 0;
                self.triangle.linear_counter_load = value & 0b01111111;
            }
            0x4009 => {}
            0x400A => {
                self.triangle.timer_load =
                    (self.triangle.timer_load & 0b11111111_00000000) | value as u16;
            }
            0x400B => {
                self.triangle.timer_load = (self.triangle.timer_load & 0b00000000_11111111)
                    | ((value & 0b111) as u16) << 8;
                if self.triangle.length_counter_enabled {
                    self.triangle.length_counter = LENGTH_COUNTER_TABLE[(value >> 3) as usize];
                }
                self.triangle.linear_counter_reload = true;
            }
            0x400C => {
                self.noise.length_counter_halt = value & 0b0010_0000 > 0;
                self.noise.envelope_constant_volume_flag = value & 0b0001_0000 > 0;
                self.noise.envelope_divider_period = value & 0b0000_1111;
            }
            0x400D => {}
            0x400E => {
                self.noise.mode = value & 0b1000_0000 > 0;
                self.noise.timer_load = NOISE_TIMER_TABLE[(value & 0b1111) as usize] as u16;
            }
            0x400F => {
                if self.noise.length_counter_enabled {
                    self.noise.length_counter = LENGTH_COUNTER_TABLE[(value >> 3) as usize];
                }
                self.noise.envelope_start_flag = true;
            }
            0x4015 => {
                self.pulse1.length_counter_enabled = (value & 1) > 0;
                if !self.pulse1.length_counter_enabled {
                    self.pulse1.length_counter = 0;
                }
                self.pulse2.length_counter_enabled = (value & 2) > 0;
                if !self.pulse2.length_counter_enabled {
                    self.pulse2.length_counter = 0;
                }
                self.triangle.length_counter_enabled = (value & 4) > 0;
                if !self.triangle.length_counter_enabled {
                    self.triangle.length_counter = 0;
                }
                self.noise.length_counter_enabled = (value & 8) > 0;
                if !self.noise.length_counter_enabled {
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

    pub fn tick(&mut self, cpu: &mut Cpu, peripherals: &mut Peripherals) {
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
                        cpu.irq();
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
            self.noise.tick();
        }

        self.triangle.tick();

        self.pulse1_samples
            .push(!self.pulse1.muted() as u8 * self.pulse1.volume());
        self.pulse2_samples
            .push(!self.pulse2.muted() as u8 * self.pulse2.volume());
        self.triangle_samples.push(self.triangle.volume());
        self.noise_samples.push(self.noise.volume());

        if self.pulse1_samples.len() == (64.0f64 * (1_789_773.0 / 44_100.0)).round() as usize {
            for i in 0..64 {
                let index = (i as f64 * (1_789_773.0 / 44_100.0)).round() as usize;
                let pulse1_sample = self.pulse1_samples[index];
                let pulse2_sample = self.pulse2_samples[index];
                let triangle_sample = self.triangle_samples[index];
                let noise_sample = self.noise_samples[index];
                let dmc_sample = 0;
                self.output_samples[i] = PULSE_MIX_TABLE[(pulse1_sample + pulse2_sample) as usize]
                    + OTHER_MIX_TABLE[3 * triangle_sample as usize
                        + 2 * noise_sample as usize
                        + dmc_sample as usize];
            }
            // print_triangle_samples(&self.triangle_samples);
            self.clear_samples();
            std::mem::swap(&mut peripherals.audio.samples, &mut self.output_samples);
            peripherals.audio.version = peripherals.audio.version.wrapping_add(1);
        }

        self.cpu_cycle_odd = !self.cpu_cycle_odd;
    }

    // Removes all generated samples.
    pub fn clear_samples(&mut self) {
        self.pulse1_samples.clear();
        self.pulse2_samples.clear();
        self.triangle_samples.clear();
        self.noise_samples.clear();
    }
}

fn print_triangle_samples(s: &Vec<u8>) {
    println!("samples:");
    let mut last_value = 0;
    let mut count = 1;

    for i in s {
        if *i == last_value {
            count += 1;
        } else {
            println!("v={i}, times={count}");
            last_value = *i;
            count = 1;
        }
    }
    println!("v={last_value}, times={count}");
}
