use crate::mapper::Mapper;
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

pub struct Ppu {
    pub scanline: u16,
    pub pixel: u16,

    pub control_register: ControlRegister,
    pub mask_register: MaskRegister,
    pub status_register: StatusRegister,
    pub vblank: bool,
    pub scroll_latch: bool,
    pub ppu_address: u16,
    pub ppu_address_latch: bool,
    pub oam_address: u8,
    pub horizontal_scroll: u8,
    pub vertical_scroll: u8,
    pub vertical_scroll_next_frame: u8,
    pub buffer: Box<Frame>,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            scanline: 0,
            pixel: 0,

            control_register: ControlRegister::new(),
            mask_register: MaskRegister::new(),
            status_register: StatusRegister::new(),
            vertical_scroll: 0,
            horizontal_scroll: 0,
            vblank: false,
            scroll_latch: false,
            ppu_address: 0,
            ppu_address_latch: false,
            oam_address: 0,
            vertical_scroll_next_frame: 0, // changes to vertical scroll don't affect the current frame
            buffer: Box::new([[(0, 0, 0); 256]; 240]),
        }
    }

    pub fn tick(&mut self, nes: &Nes) {
        if self.scanline == 241 && self.pixel == 1 {
            self.vblank = true;
        }
        if self.scanline == 261 && self.pixel == 1 {
            self.vblank = false;
            // TODO clear sprite 0
            // TODO clear overflow
        }

        if self.pixel == 340 {
            self.pixel = 0;
            if self.scanline == 261 {
                self.scanline = 0;
            } else {
                self.scanline += 1;
            }
        } else {
            self.pixel += 1;
        }
    }

    pub fn cpu_read(&mut self, nes: &Nes, address: u16) -> u8 {
        match address {
            0x2002 => {
                let vblank = self.vblank;
                let sprite_hit = false; // TODO
                let sprite_overflow = false; // TODO
                let result =
                    (vblank as u8) << 7 | (sprite_hit as u8) << 6 | (sprite_overflow as u8) << 5;
                self.vblank = false;
                self.scroll_latch = false;
                self.ppu_address_latch = false;
                dbg!(result)
            }
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
        match address {
            0x2000 => self.control_register = ControlRegister(value),
            0x2001 => self.mask_register = MaskRegister(value),
            0x2003 => {
                // if self.oam_address
                self.oam_address = value;
            }
            0x2005 => {
                if !self.scroll_latch {
                    self.horizontal_scroll = value;
                    self.scroll_latch = true;
                } else {
                    self.vertical_scroll_next_frame = value;
                }
            }
            0x2006 => {
                if !self.ppu_address_latch {
                    self.ppu_address = ((self.ppu_address & 0x00FF) | (value as u16) << 8)
                        & 0b0011_1111_1111_1111;
                    self.ppu_address_latch = true;
                } else {
                    self.ppu_address =
                        ((self.ppu_address & 0xFF00) | (value as u16)) & 0b0011_1111_1111_1111;
                }
            }
            0x2007 => {
                nes.ppu_bus_write(self.ppu_address, value);
                self.ppu_address = (self.ppu_address
                    + (1 << (self.control_register.get_increment_mode() as u8 * 5)))
                    & 0b0011_1111_1111_1111;
            }
            other => eprintln!("Write to PPU at illegal address {:04x}", other)
        }
    }
}
