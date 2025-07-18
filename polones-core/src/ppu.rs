use crate::cpu::Cpu;
use crate::nes::{Frame, Peripherals, PpuBus};
use crate::ram::Ram;

pub static PALLETTE: [(u8, u8, u8); 64] = [
    (0x65, 0x65, 0x65),
    (0x00, 0x2D, 0x69),
    (0x13, 0x1F, 0x7F),
    (0x3C, 0x13, 0x7C),
    (0x60, 0x0B, 0x62),
    (0x73, 0x0A, 0x37),
    (0x71, 0x0F, 0x07),
    (0x5A, 0x1A, 0x00),
    (0x34, 0x28, 0x00),
    (0x0B, 0x34, 0x00),
    (0x00, 0x3C, 0x00),
    (0x00, 0x3D, 0x10),
    (0x00, 0x38, 0x40),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xAE, 0xAE, 0xAE),
    (0x0F, 0x63, 0xB3),
    (0x40, 0x51, 0xD0),
    (0x78, 0x41, 0xCC),
    (0xA7, 0x36, 0xA9),
    (0xC0, 0x34, 0x70),
    (0xBD, 0x3C, 0x30),
    (0x9F, 0x4A, 0x00),
    (0x6D, 0x5C, 0x00),
    (0x36, 0x6D, 0x00),
    (0x07, 0x77, 0x04),
    (0x00, 0x79, 0x3D),
    (0x00, 0x72, 0x7D),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xFE, 0xFE, 0xFF),
    (0x5D, 0xB3, 0xFF),
    (0x8F, 0xA1, 0xFF),
    (0xC8, 0x90, 0xFF),
    (0xF7, 0x85, 0xFA),
    (0xFF, 0x83, 0xC0),
    (0xFF, 0x8B, 0x7F),
    (0xEF, 0x9A, 0x49),
    (0xBD, 0xAC, 0x2C),
    (0x85, 0xBC, 0x2F),
    (0x55, 0xC7, 0x53),
    (0x3C, 0xC9, 0x8C),
    (0x3E, 0xC2, 0xCD),
    (0x4E, 0x4E, 0x4E),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
    (0xFE, 0xFE, 0xFF),
    (0xBC, 0xDF, 0xFF),
    (0xD1, 0xD8, 0xFF),
    (0xE8, 0xD1, 0xFF),
    (0xFB, 0xCD, 0xFD),
    (0xFF, 0xCC, 0xE5),
    (0xFF, 0xCF, 0xCA),
    (0xF8, 0xD5, 0xB4),
    (0xE4, 0xDC, 0xA8),
    (0xCC, 0xE3, 0xA9),
    (0xB9, 0xE8, 0xB8),
    (0xAE, 0xE8, 0xD0),
    (0xAF, 0xE5, 0xEA),
    (0xB6, 0xB6, 0xB6),
    (0x00, 0x00, 0x00),
    (0x00, 0x00, 0x00),
];

#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct ControlRegister(u8);

macro_rules! flag_methods {
    ($get_flag_name:ident,$set_flag_name:ident,$bit:expr) => {
        pub fn $get_flag_name(&self) -> bool {
            (self.0 & (1 << $bit)) > 0
        }
        pub fn $set_flag_name(&mut self, new: bool) {
            self.0 = (self.0 & !(1 << $bit)) | ((new as u8) << $bit);
        }
    };
}

#[rustfmt::skip]
impl ControlRegister {
    flag_methods!(get_nmi_enable,             set_nmi_enable,             7);
    flag_methods!(get_ppu_master_slave,       set_ppu_master_slave,       6);
    flag_methods!(get_sprite_height,          set_sprite_height,          5);
    flag_methods!(get_background_tile_select, set_background_tile_select, 4);
    flag_methods!(get_sprite_tile_select,     set_sprite_tile_select,     3);
    flag_methods!(get_increment_mode,         set_increment_mode,         2);
    pub fn get_name_table_address(&self) -> u8 {
        self.0 & 0b00000011
    }
    pub fn set_name_table_select(&mut self, new: u8) {
        self.0 = (self.0 & 0b11111100) | (new & 0b00000011);
    }
}

#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct MaskRegister(u8);

#[rustfmt::skip]
impl MaskRegister {
    flag_methods!(get_emphasize_blue,                  set_emphasize_blue,                  7);
    flag_methods!(get_emphasize_green,                 set_emphasize_green,                 6);
    flag_methods!(get_emphasize_red,                   set_emphasize_red,                   5);
    flag_methods!(get_show_sprites,                    set_show_sprites,                    4);
    flag_methods!(get_show_background,                 set_show_background,                 3);
    flag_methods!(get_show_sprites_in_leftmost_col,    set_show_sprites_in_leftmost_col,    2);
    flag_methods!(get_show_background_in_leftmost_col, set_show_background_in_leftmost_col, 1);
    flag_methods!(get_greyscale,                       set_greyscale,                       0);
}

#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct StatusRegister(u8);

#[rustfmt::skip]
impl StatusRegister {
    flag_methods!(get_vblank_flag,          set_vblank_flag,          7);
    flag_methods!(get_sprite_0_hit_flag,    set_sprite_0_hit_flag,    6);
    flag_methods!(get_sprite_overflow_flag, set_sprite_overflow_flag, 5);
}

// yyy NN YYYYY XXXXX
// ||| || ||||| +++++-- coarse X scroll
// ||| || +++++-------- coarse Y scroll
// ||| ++-------------- nametable select
// +++----------------- fine Y scroll
#[derive(Clone, Copy, Default)]
pub struct Loopy(pub u16);

impl Loopy {
    pub fn get_coarse_x_scroll(&self) -> u8 {
        self.0 as u8 & 0b11111
    }
    pub fn set_coarse_x_scroll(&mut self, new: u8) {
        self.0 = (self.0 & 0b111_1111_1110_0000) | (new as u16 & 0b11111);
    }
    pub fn get_coarse_y_scroll(&self) -> u8 {
        (self.0 >> 5) as u8 & 0b11111
    }
    pub fn set_coarse_y_scroll(&mut self, new: u8) {
        self.0 = (self.0 & 0b111_1100_0001_1111) | ((new as u16 & 0b11111) << 5);
    }
    pub fn get_nametable_select(&self) -> u8 {
        (self.0 >> 10) as u8 & 0b11
    }
    pub fn set_nametable_select(&mut self, new: u8) {
        self.0 = (self.0 & 0b111_0011_1111_1111) | ((new as u16 & 0b11) << 10);
    }
    pub fn get_fine_y_scroll(&self) -> u8 {
        (self.0 >> 12) as u8 & 0b111
    }
    pub fn set_fine_y_scroll(&mut self, new: u8) {
        self.0 = (self.0 & 0b000_1111_1111_1111) | ((new as u16 & 0b111) << 12);
    }
    pub fn get_ppu_address(&self) -> u16 {
        self.0
    }
    pub fn set_ppu_address(&mut self, new: u16) {
        self.0 = new
    }
}

pub struct Ppu {
    pub scanline: u16,
    pub dot: u16,

    pub control_register: ControlRegister,
    pub mask_register: MaskRegister,
    pub status_register: StatusRegister,
    pub vblank: bool,
    pub oam_address: u8,
    pub ppu_read_buffer: u8,
    pub horizontal_scroll: u8,
    pub vertical_scroll: u8,
    pub vertical_scroll_next_frame: u8,
    pub buffer: Box<Frame>,
    pub buffer_index: usize,
    pub oam: [u8; 256],

    pub odd: bool,

    pub v: Loopy,
    pub t: Loopy,
    pub x: u8,
    pub w: bool,

    pub pattern_low_shift_register: u16,
    pub pattern_high_shift_register: u16,
    pub attribute_low_shift_register: u16,
    pub attribute_high_shift_register: u16,

    pub nametable_byte: u8,
    pub attribute: u8,
    pub bg_tile_byte_low: u8,
    pub bg_tile_byte_high: u8,

    pub sprite_limit: usize,
    pub sprite_secondary_oam: [u8; 256],
    sprite_patterns_low: [u8; 64],
    sprite_patterns_high: [u8; 64],
    sprite_attributes: [u8; 64],
    sprite_counters: [u8; 64],
    sprite_0_next_scanline: bool,
    sprite_0_current_scanline: bool,

    // These variables allow us to skip processing sprites that are not visible
    // on the screen on a given frame.
    sprites_next_line: usize,
    sprites_current_line: usize,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            scanline: 0,
            dot: 0,

            control_register: ControlRegister::default(),
            mask_register: MaskRegister::default(),
            status_register: StatusRegister::default(),
            vertical_scroll: 0,
            horizontal_scroll: 0,
            vblank: false,
            oam_address: 0,
            ppu_read_buffer: 0,
            vertical_scroll_next_frame: 0, // changes to vertical scroll don't affect the current frame
            buffer: Box::new([[(0, 0, 0); 256]; 240]),
            buffer_index: 0,
            oam: [0; 256],

            odd: false,
            v: Loopy::default(),
            t: Loopy::default(),
            x: 0,
            w: false,

            pattern_low_shift_register: 0,
            pattern_high_shift_register: 0,
            attribute_low_shift_register: 0,
            attribute_high_shift_register: 0,

            nametable_byte: 0,
            attribute: 0,
            bg_tile_byte_low: 0,
            bg_tile_byte_high: 0,

            sprite_limit: 8,
            sprite_secondary_oam: [0xFF; 256],
            sprite_patterns_low: [0; 64],
            sprite_patterns_high: [0; 64],
            sprite_attributes: [0; 64],
            sprite_counters: [0xFF; 64],
            sprite_0_next_scanline: false,
            sprite_0_current_scanline: false,

            sprites_next_line: 0,
            sprites_current_line: 0,
        }
    }

    pub fn tick(&mut self, cpu: &mut Cpu, ppu_bus: &mut PpuBus, peripherals: &mut Peripherals) {
        // to call on scanline 241 dot 1
        macro_rules! start_vblank {
            () => {
                self.vblank = true;
                self.status_register.set_vblank_flag(true);
                if self.control_register.get_nmi_enable() {
                    cpu.nmi();
                }
            };
        }

        // to call on scanline 261 dot 1
        macro_rules! end_vblank {
            () => {
                self.vblank = false;
                self.status_register.set_vblank_flag(false);
                self.status_register.set_sprite_0_hit_flag(false);
                self.status_register.set_sprite_overflow_flag(false);
            };
        }

        // to call on scanlines 0..=239 and 261, dots 257..=320
        macro_rules! fetch_sprites {
            () => {
                let nth = self.dot - 257;
                let sprite_number = nth as usize >> 3;
                match nth & 0b111 {
                    0 => {
                        // garbage NT read
                        ppu_bus.read(0x2000);
                    }
                    2 => {
                        // garbage AT read
                        ppu_bus.read(0x23C0);
                        self.sprite_attributes[sprite_number] =
                            self.sprite_secondary_oam[sprite_number * 4 + 2];
                    }
                    3 => {
                        self.sprite_counters[sprite_number] =
                            self.sprite_secondary_oam[sprite_number * 4 + 3];
                    }
                    4 => {
                        self.set_oam_pattern(ppu_bus, sprite_number, false);
                    }
                    6 => {
                        self.set_oam_pattern(ppu_bus, sprite_number, true);
                    }
                    _ => {}
                }
            };
        }

        // to call on scanlines 0..=239 and 261, dot 320
        macro_rules! fetch_sprites_8_64 {
            () => {
                for sprite_number in 8..self.sprite_limit {
                    self.sprite_counters[sprite_number] =
                        self.sprite_secondary_oam[sprite_number * 4 + 3];
                    self.set_oam_pattern(ppu_bus, sprite_number, false);
                    self.set_oam_pattern(ppu_bus, sprite_number, true);
                }
                self.sprite_0_current_scanline = self.sprite_0_next_scanline;
                self.sprites_current_line = self.sprites_next_line;
            };
        }

        // to call on scanlines 0..=239, dots 1..=256
        macro_rules! draw_pixel {
            () => {
                let mut background = None;
                if self.mask_register.get_show_background()
                    && !(!self.mask_register.get_show_background_in_leftmost_col() && self.dot <= 8)
                {
                    let mask: u16 = 0b1000_0000_0000_0000 >> self.x;
                    let pattern_low = ((self.pattern_low_shift_register & mask) > 0) as u8;
                    let pattern_high = ((self.pattern_high_shift_register & mask) > 0) as u8;
                    let color = (pattern_high << 1) | pattern_low;
                    let palette_low = ((self.attribute_low_shift_register & mask) > 0) as u8;
                    let palette_high = ((self.attribute_high_shift_register & mask) > 0) as u8;
                    let palette = (palette_high << 1) | palette_low;

                    background = Some((color, palette));
                }
                let mut foreground = None;
                if self.mask_register.get_show_sprites()
                    && !(!self.mask_register.get_show_sprites() && self.dot <= 8)
                {
                    for i in 0..self.sprites_current_line {
                        if self.sprite_counters[i] == 0 {
                            let color_low = self.sprite_patterns_low[i] >> 7;
                            let color_high = self.sprite_patterns_high[i] >> 7;
                            if color_low != 0 || color_high != 0 {
                                let color = (color_high << 1) | color_low;
                                let palette = self.sprite_attributes[i] & 0b11;
                                let priority_back = (self.sprite_attributes[i] >> 5) & 1 == 1;
                                foreground = Some((color, palette, priority_back, i));
                                break;
                            }
                        }
                    }
                }
                let get_bg_color = |palette_ram: &Ram<32>, color: u8, palette: u8| {
                    if color == 0b00 {
                        palette_ram.read(0)
                    } else {
                        palette_ram.read((palette << 2) as usize | color as usize)
                    }
                };
                let get_fg_color = |palette_ram: &Ram<32>, color: u8, palette: u8| {
                    if color == 0b00 {
                        palette_ram.read(0x10)
                    } else {
                        palette_ram.read(0x10 | (palette << 2) as usize | color as usize)
                    }
                };
                let get_rgb = |color: u8| PALLETTE[color as usize];

                let rgb = match (foreground, background) {
                    (
                        Some((fg_color, fg_palette, fg_priority_back, sprite_index)),
                        Some((bg_color, bg_palette)),
                    ) => {
                        let sprite_0_hit = self.sprite_0_current_scanline
                            && self.dot != 256
                            && sprite_index == 0
                            && fg_color != 0
                            && bg_color != 0;

                        if sprite_0_hit {
                            self.sprite_0_current_scanline = false;
                            self.status_register.set_sprite_0_hit_flag(true);
                        }

                        if (!fg_priority_back && fg_color != 0) || bg_color == 0 {
                            get_rgb(get_fg_color(ppu_bus.ppu_palette_ram, fg_color, fg_palette))
                        } else {
                            get_rgb(get_bg_color(ppu_bus.ppu_palette_ram, bg_color, bg_palette))
                        }
                    }
                    (Some((fg_color, fg_palette, _fg_priority, _sprite_index)), None) => {
                        get_rgb(get_fg_color(ppu_bus.ppu_palette_ram, fg_color, fg_palette))
                    }
                    (None, Some((bg_color, bg_palette))) => {
                        get_rgb(get_bg_color(ppu_bus.ppu_palette_ram, bg_color, bg_palette))
                    }
                    (None, None) => {
                        let ppu_addr = self.v.get_ppu_address() & 0b0011_1111_1111_1111;
                        if ppu_addr >= 0x3F00 {
                            get_rgb(ppu_bus.read(ppu_addr) & 0b00111111)
                        } else {
                            get_rgb(ppu_bus.read(0x3F00) & 0b00111111)
                        }
                    }
                };
                self.buffer[self.buffer_index / 256][self.buffer_index % 256] = rgb;
                self.buffer_index += 1;

                if self.buffer_index == 256 * 240 {
                    self.buffer_index = 0;
                    std::mem::swap(&mut peripherals.display.frame, &mut self.buffer);
                    peripherals.display.cpu_cycle = cpu.cycle;
                    peripherals.display.version = peripherals.display.version.wrapping_add(1);
                }
            };
        }

        macro_rules! fetch_background_tiles {
            () => {
                match (self.dot - 1) & 0b111 {
                    0 => {
                        self.nametable_byte = ppu_bus.read(0x2000 | (self.v.0 & 0x0FFF));
                    }
                    2 => {
                        // NN 1111 YYY XXX
                        // || |||| ||| +++-- high 3 bits of coarse X (x/4)
                        // || |||| +++------ high 3 bits of coarse Y (y/4)
                        // || ++++---------- attribute offset (960 bytes)
                        // ++--------------- nametable select
                        let attribute_byte = ppu_bus.read(
                            0x2000 | // nametables starting address
                            (self.v.get_nametable_select() as u16 >> 0 << 10) | // nametable select
                            0x03C0 | // attribute offset
                            (self.v.get_coarse_y_scroll() as u16 >> 2 << 3) |
                            (self.v.get_coarse_x_scroll() as u16 >> 2),
                        );
                        self.attribute = (attribute_byte
                            >> ((self.v.get_coarse_y_scroll() & 2) << 1)
                            >> (self.v.get_coarse_x_scroll() & 2))
                            & 0b11;
                    }
                    4 => {
                        // 0HRRRR CCCCPTTT
                        // |||||| |||||+++- T: Fine Y offset, the row number within a tile
                        // |||||| ||||+---- P: Bit plane (0: "lower"; 1: "upper")
                        // |||||| ++++----- C: Tile column
                        // ||++++---------- R: Tile row
                        // |+-------------- H: Half of sprite table (0: "left"; 1: "right")
                        // +--------------- 0: Pattern table is at $0000-$1FFF
                        self.bg_tile_byte_low = ppu_bus.read(
                            self.control_register.get_background_tile_select() as u16 >> 0 << 12
                                | self.nametable_byte as u16 >> 0 << 4
                                | self.v.get_fine_y_scroll() as u16,
                        );
                    }
                    6 => {
                        // same as above, but with plane bit set
                        self.bg_tile_byte_high = ppu_bus.read(
                            self.control_register.get_background_tile_select() as u16 >> 0 << 12
                                | self.nametable_byte as u16 >> 0 << 4
                                | 0b1000
                                | self.v.get_fine_y_scroll() as u16,
                        );
                    }
                    7 if self.dot != 256 => {
                        if self.is_rendering_enabled() {
                            self.increment_v_horizontal();
                        }
                    }
                    7 if self.dot == 256 => {
                        if self.is_rendering_enabled() {
                            self.increment_v_horizontal();
                            self.increment_v_vertical();
                        }
                    }
                    _ => {}
                }
            };
        }

        macro_rules! update_scroll_horizontal {
            () => {
                if self.is_rendering_enabled() {
                    self.v.set_coarse_x_scroll(self.t.get_coarse_x_scroll());
                    self.v.set_nametable_select(
                        (self.v.get_nametable_select() & 0b10)
                            | (self.t.get_nametable_select() & 0b01),
                    );
                }
            };
        }

        macro_rules! update_scroll_vertical {
            () => {
                if self.is_rendering_enabled() {
                    self.v.set_coarse_y_scroll(self.t.get_coarse_y_scroll());
                    self.v.set_fine_y_scroll(self.t.get_fine_y_scroll());
                    self.v.set_nametable_select(
                        (self.v.get_nametable_select() & 0b01)
                            | (self.t.get_nametable_select() & 0b10),
                    );
                }
            };
        }

        // to call on scanlines 0..=239 and 261, dots 1..=256 and 321..=336
        macro_rules! rotate_pattern_and_attribute_shift_registers {
            () => {
                self.pattern_low_shift_register <<= 1;
                self.pattern_high_shift_register <<= 1;
                self.attribute_low_shift_register <<= 1;
                self.attribute_high_shift_register <<= 1;
            };
        }

        // to call on scanlines 0..=239 and 261, dots 1..=256
        macro_rules! rotate_sprite_patterns {
            () => {
                for i in 0..self.sprites_current_line {
                    if self.sprite_counters[i] > 0 {
                        self.sprite_counters[i] -= 1;
                    } else {
                        self.sprite_patterns_low[i] <<= 1;
                        self.sprite_patterns_high[i] <<= 1;
                    }
                }
            };
        }

        // to call on scanlines 0..=239 and 261, dots 1..=256 and 321..=336
        // Note: https://www.nesdev.org/wiki/PPU_rendering says that:
        // > The shifters are reloaded during ticks 9, 17, 25, ..., 257.
        // However, loading shift registers at the end of ticks 8, 16, 24, ...,
        // 256 gives the same result, and avoids additional dot comparisons.
        macro_rules! load_pattern_and_attribute_shift_registers {
            () => {
                if self.dot & 0b111 == 0 {
                    self.pattern_low_shift_register |= self.bg_tile_byte_low as u16;
                    self.pattern_high_shift_register |= self.bg_tile_byte_high as u16;
                    self.attribute_low_shift_register |= if self.attribute & 0b01 > 0 {
                        0xFF
                    } else {
                        0x00
                    };
                    self.attribute_high_shift_register |= if self.attribute & 0b10 > 0 {
                        0xFF
                    } else {
                        0x00
                    };
                }
            };
        }

        // It turns out that minimizing the number of scanline and dot
        // comparisons can save a lot of CPU time. My strategy here is to
        // divide scanline-dot space into multiple regions with distict PPU
        // logic. I check scaline boundaries first, in big-to-small range
        // order. Then I check dot boundaries, again, in big-to-small range
        // order. That way the code handling 75% of scanline-dot space
        // (A1 + B1) is reached with 2 integer comparisons.
        //
        // Visualization of regions (not to scale)
        //
        // dots -->          0-256             257-320    321-336  337-340
        // scanlines   0-239 +-----------------+----------+--------+------+
        // |                 |                 |          |        |      |
        // V                 |                 |          |        |      |
        //                   |                 |          |        |      |
        //                   |        A1       |    A2    |   A3   |  A4  |
        //                   |                 |          |        |      |
        //                   |                 |          |        |      |
        //                   |                 |          |        |      |
        //           240-260 +-----------------+----------+--------+------+
        //                   |                                            |
        //                   |                      B1                    |
        //                   |                                            |
        //               261 +-----------------+----------+--------+------+
        //                   |        C1       |    C2    |   C3   |  C4  |
        //                   +-----------------+----------+--------+------+

        if self.scanline < 240 {
            if self.dot < 257 {
                // region A1
                if self.dot > 0 {
                    if self.dot == 64 {
                        self.clear_secondary_oam();
                    } else if self.dot == 256 {
                        self.evaluate_sprites();
                    }
                    draw_pixel!();
                    fetch_background_tiles!();
                    rotate_pattern_and_attribute_shift_registers!();
                    load_pattern_and_attribute_shift_registers!();
                    rotate_sprite_patterns!();
                }
                self.dot += 1;
            } else if self.dot < 321 {
                // region A2
                fetch_sprites!();
                if self.dot == 320 {
                    fetch_sprites_8_64!();
                } else if self.dot == 257 {
                    update_scroll_horizontal!();
                }
                self.dot += 1;
            } else if self.dot < 337 {
                // region A3
                fetch_background_tiles!();
                rotate_pattern_and_attribute_shift_registers!();
                load_pattern_and_attribute_shift_registers!();
                self.dot += 1;
            } else {
                // region A4
                if self.dot & 1 == 1 {
                    self.nametable_byte = ppu_bus.read(0x2000 + (self.v.0 & 0x0FFF));
                }
                if self.dot == 340 {
                    self.dot = 0;
                    self.scanline += 1;
                } else {
                    self.dot += 1;
                }
            }
        } else if self.scanline < 261 {
            // region B1
            if self.scanline == 241 && self.dot == 1 {
                start_vblank!();
            }
            if self.dot == 340 {
                self.dot = 0;
                self.scanline += 1;
            } else {
                self.dot += 1;
            }
        } else
        /* self.scanline == 261 */
        {
            // region C1
            if self.dot < 257 {
                if self.dot > 0 {
                    if self.dot == 1 {
                        end_vblank!();
                    }
                    fetch_background_tiles!();
                    rotate_pattern_and_attribute_shift_registers!();
                    load_pattern_and_attribute_shift_registers!();
                    rotate_sprite_patterns!();
                }
                self.dot += 1;
            } else if self.dot < 321 {
                // region C2
                fetch_sprites!();
                if self.dot == 320 {
                    fetch_sprites_8_64!();
                } else if self.dot == 257 {
                    update_scroll_horizontal!();
                } else if self.dot >= 280 && self.dot <= 304 {
                    update_scroll_vertical!();
                }
                self.dot += 1;
            } else if self.dot < 337 {
                // region C3
                fetch_background_tiles!();
                rotate_pattern_and_attribute_shift_registers!();
                load_pattern_and_attribute_shift_registers!();
                self.dot += 1;
            } else {
                // region C4
                if self.dot & 1 == 1 {
                    self.nametable_byte = ppu_bus.read(0x2000 + (self.v.0 & 0x0FFF));
                }
                if self.dot == 340 {
                    self.scanline = 0;
                    // On odd frames scanline 261 dot 340 should be skipped.
                    // However, it's easier to run it normally and then skip
                    // to scanline 0 dot 1
                    self.dot = self.odd as u16;
                    self.odd = !self.odd;
                } else {
                    self.dot += 1;
                }
            }
        }
    }

    pub fn read(&mut self, address: u16, ppu_bus: &mut PpuBus) -> u8 {
        match 0x2000 + (address & 0x0007) {
            0x2002 => {
                let result = self.status_register.0;
                self.status_register.set_vblank_flag(false);
                self.w = false;
                result
            }
            0x2004 => self.oam[self.oam_address as usize],
            0x2007 => {
                let result;

                // Reading from memory other than palette RAM writes to PPU
                // read buffer and return the previous value from the buffer.
                if self.v.get_ppu_address() & 0x3FFF < 0x3F00 {
                    result = self.ppu_read_buffer;
                    self.ppu_read_buffer = ppu_bus.read(self.v.get_ppu_address());
                } else {
                    result = ppu_bus.read(self.v.get_ppu_address());
                    // Reads from palette area fill the PPU read buffer with
                    // that's "behind" palette memory - nametable data. Because
                    // that these addresses are mapped to the palette RAM, we
                    // decrement the address to read from lower mirror of nametables.
                    self.ppu_read_buffer =
                        ppu_bus.read((self.v.get_ppu_address() & 0x3FFF) - 0x1000);
                }

                self.v.set_ppu_address(
                    (self.v.get_ppu_address()
                        + (1 << (self.control_register.get_increment_mode() as u16 * 5)))
                        & 0b0011_1111_1111_1111,
                );
                result
            }
            other => {
                eprintln!("Read from PPU at illegal address {:04x}", other);
                0
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8, ppu_bus: &mut PpuBus) {
        match 0x2000 + (address & 0x0007) {
            0x2000 => {
                self.control_register = ControlRegister(value);
                self.t.set_nametable_select(value & 0b11);
                // TODO implement early NMI trigger bug
            }
            0x2001 => self.mask_register = MaskRegister(value),
            0x2003 => {
                self.oam_address = value;
            }
            0x2004 => {
                self.oam[self.oam_address as usize] = value;
                self.oam_address = self.oam_address.wrapping_add(1);
            }
            0x2005 => {
                if !self.w {
                    self.t.set_coarse_x_scroll(value >> 3);
                    self.x = value & 0b111;
                } else {
                    self.t.set_coarse_y_scroll(value >> 3);
                    self.t.set_fine_y_scroll(value);
                }
                self.w = !self.w;
            }
            0x2006 => {
                if !self.w {
                    self.t.set_ppu_address(
                        (self.t.get_ppu_address() & 0x00FF)
                            | ((value as u16) << 8) & 0b0011_1111_1111_1111,
                    );
                } else {
                    self.t.set_ppu_address(
                        (self.t.get_ppu_address() & 0xFF00)
                            | (value as u16) & 0b0011_1111_1111_1111,
                    );
                    self.v.0 = self.t.0;
                }
                self.w = !self.w;
            }
            0x2007 => {
                ppu_bus.write(self.v.get_ppu_address() & 0b0011_1111_1111_1111, value);
                self.v.set_ppu_address(
                    (self.v.get_ppu_address()
                        + (1 << (self.control_register.get_increment_mode() as u8 * 5)))
                        & 0b0011_1111_1111_1111,
                );
            }
            other => eprintln!("Write to PPU at illegal address {:04x}", other),
        }
    }

    fn is_rendering_enabled(&self) -> bool {
        self.mask_register.get_show_sprites() || self.mask_register.get_show_background()
    }

    fn increment_v_horizontal(&mut self) {
        if self.v.get_coarse_x_scroll() == 31 {
            self.v.set_coarse_x_scroll(0);
            self.v
                .set_nametable_select(self.v.get_nametable_select() ^ 0b01);
        } else {
            self.v.set_coarse_x_scroll(self.v.get_coarse_x_scroll() + 1);
        }
    }

    fn increment_v_vertical(&mut self) {
        if self.v.get_fine_y_scroll() < 7 {
            self.v.set_fine_y_scroll(self.v.get_fine_y_scroll() + 1);
        } else {
            self.v.set_fine_y_scroll(0);
            match self.v.get_coarse_y_scroll() {
                29 => {
                    self.v.set_coarse_y_scroll(0);
                    self.v
                        .set_nametable_select(self.v.get_nametable_select() ^ 0b10);
                }
                31 => {
                    self.v.set_coarse_y_scroll(0);
                }
                other => self.v.set_coarse_y_scroll(other + 1),
            }
        }
    }

    fn evaluate_sprites(&mut self) {
        let mut n = 0;
        let mut n_on_overflow = 0;
        let mut sprites_found = 0;
        let sprite_height = 8 << (self.control_register.get_sprite_height() as u8);

        self.sprite_0_next_scanline = false;

        loop {
            let y = self.oam[n * 4 + 0];

            // Copying the first byte of a sprite to secondary oam before
            // checking sprite's y position is the behavior documented at
            // https://www.nesdev.org/wiki/PPU_OAM
            // The theoretical consequence of this logic is showing a broken
            // sprite on the last pixel of every line. I don't know if original
            // NES does that, but both mesen and fceux don't. Polones ignores
            // these broken sprites by setting self.sprites_next_line to the
            // number of "correct" sprites.
            self.sprite_secondary_oam[sprites_found * 4 + 0] = y;

            if between_exc(y, y.saturating_add(sprite_height), self.scanline as u8) {
                self.sprite_secondary_oam[sprites_found * 4 + 1] = self.oam[n * 4 + 1];
                self.sprite_secondary_oam[sprites_found * 4 + 2] = self.oam[n * 4 + 2];
                self.sprite_secondary_oam[sprites_found * 4 + 3] = self.oam[n * 4 + 3];
                sprites_found += 1;

                if n == 0 {
                    self.sprite_0_next_scanline = true;
                }
            }
            n += 1;
            if n == 64 {
                break;
            }
            if sprites_found == 8 {
                n_on_overflow = n;
            }
            if sprites_found == self.sprite_limit {
                break;
            }
        }
        if sprites_found == 8 {
            let mut m = 0;
            n = n_on_overflow;
            while n < 64 {
                let y = self.oam[n * 4 + m];
                if between_exc(y, y.saturating_add(sprite_height), self.scanline as u8) {
                    self.status_register.set_sprite_overflow_flag(true);
                    break;
                } else {
                    n += 1;
                    // hardware bug
                    m = (n + 1) & 0b11;

                    if n == 64 {
                        break;
                    }
                }
            }
        }

        self.sprites_next_line = sprites_found;
    }

    fn clear_secondary_oam(&mut self) {
        for byte in &mut self.sprite_secondary_oam[..] {
            *byte = 0xFF;
        }
    }

    fn set_oam_pattern(&mut self, ppu_bus: &mut PpuBus, sprite_number: usize, pattern_high: bool) {
        let y = self.sprite_secondary_oam[sprite_number * 4 + 0] as u16;
        let index = self.sprite_secondary_oam[sprite_number * 4 + 1];
        let attributes = self.sprite_secondary_oam[sprite_number * 4 + 2];
        let flip_horizontally = attributes & 0b01000000 > 0;
        let flip_vertically = attributes & 0b10000000 > 0;

        let sprite_height = 8 << (self.control_register.get_sprite_height() as u8);

        let target = if pattern_high {
            &mut self.sprite_patterns_high
        } else {
            &mut self.sprite_patterns_low
        };

        // broken sprites can be further than 7 or 15 pixes from the current scanline
        let scanline_offset = self.scanline.overflowing_sub(y).0 & (sprite_height - 1);

        let character_table;
        let tile_offset;
        let tile_row_number;

        if sprite_height == 8 {
            character_table = self.control_register.get_sprite_tile_select() as u8;
            tile_offset = index;
            tile_row_number = if flip_vertically {
                7 - scanline_offset
            } else {
                scanline_offset
            }
        } else {
            character_table = index & 1;
            if flip_vertically {
                tile_offset = (index & 0b11111110) | (((scanline_offset >> 3) as u8) ^ 1);
                tile_row_number = 7 - (scanline_offset & 0b111);
            } else {
                tile_offset = (index & 0b11111110) | (scanline_offset >> 3) as u8;
                tile_row_number = scanline_offset & 0b111;
            }
        };

        // 0HRRRR CCCCPTTT
        // |||||| |||||+++- T: Fine Y offset, the row number within a tile
        // |||||| ||||+---- P: Bit plane (0: "lower"; 1: "upper")
        // |||||| ++++----- C: Tile column
        // ||++++---------- R: Tile row
        // |+-------------- H: Half of sprite table (0: "left"; 1: "right")
        // +--------------- 0: Pattern table is at $0000-$1FFF
        let tile_row = ppu_bus.read(
            (character_table as u16) << 12
                | (tile_offset as u16) << 4
                | (pattern_high as u16) << 3
                | tile_row_number,
        );

        target[sprite_number] = if flip_horizontally {
            tile_row.reverse_bits()
        } else {
            tile_row
        };
    }
}

#[inline(always)]
fn between_exc<T: PartialOrd>(start: T, end: T, value: T) -> bool {
    value >= start && value < end
}
