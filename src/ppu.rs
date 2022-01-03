use std::borrow::Borrow;

use crate::nes::{Frame, Nes};

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

#[derive(Clone, Copy)]
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
    fn new() -> Self {
        Self(0)
    }
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

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MaskRegister(u8);

#[rustfmt::skip]
impl MaskRegister {
    fn new() -> Self {
        Self(0)
    }
    flag_methods!(get_emphasize_blue,                  set_emphasize_blue,                  7);
    flag_methods!(get_emphasize_green,                 set_emphasize_green,                 6);
    flag_methods!(get_emphasize_red,                   set_emphasize_red,                   5);
    flag_methods!(get_show_sprites,                    set_show_sprites,                    4);
    flag_methods!(get_show_background,                 set_show_background,                 3);
    flag_methods!(get_show_sprites_in_leftmost_col,    set_show_sprites_in_leftmost_col,    2);
    flag_methods!(get_show_background_in_leftmost_col, set_show_background_in_leftmost_col, 1);
    flag_methods!(get_greyscale,                       set_greyscale,                       0);
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StatusRegister(u8);

#[rustfmt::skip]
impl StatusRegister {
    fn new() -> Self {
        Self(0)
    }
    flag_methods!(get_hit_flag,    set_hit_flag,    1);
    flag_methods!(get_vblank_flag, set_vblank_flag, 0);
}

// yyy NN YYYYY XXXXX
// ||| || ||||| +++++-- coarse X scroll
// ||| || +++++-------- coarse Y scroll
// ||| ++-------------- nametable select
// +++----------------- fine Y scroll
pub struct Loopy(pub u16);

impl Loopy {
    pub fn new() -> Self {
        Self(0)
    }
    pub fn get_coarse_x_scroll(&self) -> u8 {
        self.0 as u8 & 0b11111
    }
    pub fn set_coarse_x_scroll(&mut self, new: u8) {
        self.0 = (self.0 & 0b111_11_11111_00000) | (new as u16 & 0b11111);
    }
    pub fn get_coarse_y_scroll(&self) -> u8 {
        (self.0 >> 5) as u8 & 0b11111
    }
    pub fn set_coarse_y_scroll(&mut self, new: u8) {
        self.0 = (self.0 & 0b111_11_00000_11111) | ((new as u16 & 0b11111) << 5);
    }
    pub fn get_nametable_select(&self) -> u8 {
        (self.0 >> 10) as u8 & 0b11
    }
    pub fn set_nametable_select(&mut self, new: u8) {
        self.0 = (self.0 & 0b111_00_11111_11111) | ((new as u16 & 0b11) << 10);
    }
    pub fn get_fine_y_scroll(&self) -> u8 {
        (self.0 >> 12) as u8 & 0b111
    }
    pub fn set_fine_y_scroll(&mut self, new: u8) {
        self.0 = (self.0 & 0b000_11_11111_11111) | ((new as u16 & 0b111) << 12);
    }
}

pub struct Ppu {
    pub scanline: u16,
    pub dot: u16,

    pub control_register: ControlRegister,
    pub mask_register: MaskRegister,
    pub status_register: StatusRegister,
    pub vblank: bool,
    pub scroll_latch: bool,
    pub ppu_address: u16,
    pub oam_address: u8,
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
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            scanline: 0,
            dot: 0,

            control_register: ControlRegister::new(),
            mask_register: MaskRegister::new(),
            status_register: StatusRegister::new(),
            vertical_scroll: 0,
            horizontal_scroll: 0,
            vblank: false,
            scroll_latch: false,
            ppu_address: 0,
            oam_address: 0,
            vertical_scroll_next_frame: 0, // changes to vertical scroll don't affect the current frame
            buffer: Box::new([[(0, 0, 0); 256]; 240]),
            buffer_index: 0,
            oam: [0; 256],

            odd: false,
            v: Loopy::new(),
            t: Loopy::new(),
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
        }
    }

    pub fn tick(&mut self, nes: &Nes) {
        // update flags
        if self.scanline == 241 && self.dot == 1 {
            self.vblank = true;
            self.status_register.set_vblank_flag(true);
            if self.control_register.get_nmi_enable() {
                nes.cpu.borrow_mut().nmi();
            }
        }
        if self.scanline == 261 && self.dot == 1 {
            self.vblank = false;
            self.status_register.set_vblank_flag(false);
            // TODO clear sprite 0
            // TODO clear overflow
        }

        // deal with render and pre-render scanlines
        if self.scanline <= 239 || self.scanline == 261 {
            if (2..=257).contains(&self.dot) || (322..=337).contains(&self.dot) {
                self.pattern_low_shift_register <<= 1;
                self.pattern_high_shift_register <<= 1;
                self.attribute_low_shift_register <<= 1;
                self.attribute_high_shift_register <<= 1;
            }
            if (9..=257).contains(&self.dot) || (329..=337).contains(&self.dot) {
                if (self.dot - 1) % 8 == 0 {
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
            }
            if self.scanline <= 239 && (1..=256).contains(&self.dot) {
                if self.mask_register.get_show_background() {
                    let mask: u16 = 0b10000000_00000000 >> self.x;
                    let pattern_low = ((self.pattern_low_shift_register & mask) > 0) as u8;
                    let pattern_high = ((self.pattern_high_shift_register & mask) > 0) as u8;
                    let color = (pattern_high << 1) | pattern_low;
                    let palette_low = ((self.attribute_low_shift_register & mask) > 0) as u8;
                    let palette_high = ((self.attribute_high_shift_register & mask) > 0) as u8;
                    let palette = (palette_high << 1) | palette_low;

                    let rgb = PALLETTE[if color == 0b00 {
                        (nes.ppu_bus_read(0x3F00) & 0b00111111) as usize
                    } else {
                        (nes.ppu_bus_read(0x3F00 | (palette << 2) as u16 | color as u16)
                            & 0b00111111) as usize
                    }];
                    self.buffer[self.buffer_index / 256][self.buffer_index % 256] = rgb;
                }
                self.buffer_index += 1;
            }
            if (1..=256).contains(&self.dot) || (321..=336).contains(&self.dot) {
                match (self.dot - 1) % 8 {
                    0 => {
                        self.nametable_byte = nes.ppu_bus_read(0x2000 + (self.v.0 & 0x0FFF));
                    }
                    2 => {
                        // NN 1111 YYY XXX
                        // || |||| ||| +++-- high 3 bits of coarse X (x/4)
                        // || |||| +++------ high 3 bits of coarse Y (y/4)
                        // || ++++---------- attribute offset (960 bytes)
                        // ++--------------- nametable select
                        let attribute_byte = nes.ppu_bus_read(
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
                        self.bg_tile_byte_low = nes.ppu_bus_read(
                            self.control_register.get_background_tile_select() as u16 >> 0 << 12
                                | self.nametable_byte as u16 >> 0 << 4
                                | self.v.get_fine_y_scroll() as u16,
                        );
                    }
                    6 => {
                        // same as above, but with plane bit set
                        self.bg_tile_byte_high = nes.ppu_bus_read(
                            self.control_register.get_background_tile_select() as u16 >> 0 << 12
                                | self.nametable_byte as u16 >> 0 << 4
                                | 0b1000
                                | self.v.get_fine_y_scroll() as u16,
                        );
                    }
                    7 if self.dot != 256 => {
                        self.increment_horizontal();
                    }
                    7 if self.dot == 256 => {
                        self.increment_horizontal();
                        self.increment_vertical();
                    }
                    _ => {}
                }
            }
            if self.dot == 257 {
                if self.mask_register.get_show_background() || self.mask_register.get_show_sprites()
                {
                    self.v.set_coarse_x_scroll(self.t.get_coarse_x_scroll());
                    self.v.set_nametable_select(
                        (self.v.get_nametable_select() & 0b10)
                            | (self.t.get_nametable_select() & 0b01),
                    );
                }
            }
            if (258..=320).contains(&self.dot) {
                // TODO fetch sprite data
            }
            if (337..=339).contains(&self.dot) {
                self.nametable_byte = nes.ppu_bus_read(0x2000 + (self.v.0 & 0x0FFF));
            }
        }

        // display screen buffer if ready
        if self.buffer_index == 256 * 240 {
            self.buffer_index = 0;
            let mut buffer = Box::new([[(0, 0, 0); 256]; 240]);
            std::mem::swap(&mut buffer, &mut self.buffer);
            nes.display.borrow_mut().display(buffer);
        }

        //
        if self.dot == 257 {}
        if self.scanline == 261 && self.dot >= 280 && self.dot <= 304 {
            self.v.set_coarse_y_scroll(self.t.get_coarse_y_scroll());
            self.v.set_fine_y_scroll(self.t.get_fine_y_scroll());
            self.v.set_nametable_select(
                (self.v.get_nametable_select() & 0b01) | (self.t.get_nametable_select() & 0b10),
            );
        }

        if self.dot == 340 {
            self.dot = 0;
            if self.scanline == 261 {
                // On odd frames scanline 261 dot 340 should be skipped.
                // However, it's easier to run it normally and then skip
                // to scanline 0 dot 1
                if self.odd {
                    self.scanline = 0;
                    self.dot = 1;
                } else {
                    self.scanline = 0;
                }
                self.buffer_index = 0;
                self.odd = !self.odd;
            } else {
                self.scanline += 1;
            }
        } else {
            self.dot += 1;
        }
    }

    pub fn cpu_read(&mut self, nes: &Nes, address: u16) -> u8 {
        match 0x2000 + (address & 0x0007) {
            0x2002 => {
                let vblank = self.status_register.get_vblank_flag();
                let sprite_hit = false; // TODO
                let sprite_overflow = false; // TODO
                let result =
                    (vblank as u8) << 7 | (sprite_hit as u8) << 6 | (sprite_overflow as u8) << 5;
                self.status_register.set_vblank_flag(false);
                self.scroll_latch = false;
                self.w = false;
                result
            }
            0x2004 => self.oam[self.oam_address as usize],
            0x2007 => {
                let result = nes.ppu_bus_read(self.ppu_address);
                self.ppu_address = (self.ppu_address
                    + (1 << (self.control_register.get_increment_mode() as u8 * 5)))
                    & 0b0011_1111_1111_1111;
                result
            }
            other => {
                eprintln!("Read from PPU at illegal address {:04x}", other);
                0
            }
        }
    }

    pub fn cpu_write(&mut self, nes: &Nes, address: u16, value: u8) {
        match 0x2000 + (address & 0x0007) {
            0x2000 => {
                let old_control_register = self.control_register;
                self.control_register = ControlRegister(value);
                self.t.set_nametable_select(value & 0b11);
                // if !old_control_register.get_nmi_enable() && self.control_register.get_nmi_enable() && self.vblank && self.status_register.get_vblank_flag() {
                //     trigger NMI early
                // }
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
                    self.t.0 = (self.t.0 & 0x00FF) | ((value as u16 & 0b00111111) << 8);
                    self.ppu_address =
                        ((self.ppu_address & 0x00FF) | (value as u16) << 8) & 0b0011_1111_1111_1111;
                } else {
                    self.t.0 = (self.t.0 & 0xFF00) | (value as u16);
                    self.v.0 = self.t.0;
                    self.ppu_address =
                        ((self.ppu_address & 0xFF00) | (value as u16)) & 0b0011_1111_1111_1111;
                }
                self.w = !self.w;
            }
            0x2007 => {
                nes.ppu_bus_write(self.ppu_address, value);
                self.ppu_address = (self.ppu_address
                    + (1 << (self.control_register.get_increment_mode() as u8 * 5)))
                    & 0b0011_1111_1111_1111;
            }
            other => eprintln!("Write to PPU at illegal address {:04x}", other),
        }
    }

    fn increment_horizontal(&mut self) {
        if self.mask_register.get_show_background() || self.mask_register.get_show_sprites() {
            if self.v.get_coarse_x_scroll() == 31 {
                self.v.set_coarse_x_scroll(0);
                self.v
                    .set_nametable_select(self.v.get_nametable_select() ^ 0b01);
            } else {
                self.v.set_coarse_x_scroll(self.v.get_coarse_x_scroll() + 1);
            }
        }
    }

    fn increment_vertical(&mut self) {
        if self.mask_register.get_show_background() || self.mask_register.get_show_sprites() {
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
    }
}
