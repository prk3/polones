use nes_lib::*;

use nes_lib::game_file::GameFile;
use nes_lib::nes::{DebugDisplay, Display, Frame, Input, Nes};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use std::borrow::Borrow;
use std::rc::Rc;
use std::time::Duration;

const FONT: &[u8; 16 * 32] = include_bytes!("../resources/comodore-64-font.bin");

const BLACK: (u8, u8, u8) = (0, 0, 0);
const RED: (u8, u8, u8) = (255, 0, 0);
const GREEN: (u8, u8, u8) = (0, 255, 0);
const BLUE: (u8, u8, u8) = (0, 0, 255);
const YELLOW: (u8, u8, u8) = (255, 255, 0);
const CYAN: (u8, u8, u8) = (0, 255, 255);
const MAGENTA: (u8, u8, u8) = (255, 0, 255);
const WHITE: (u8, u8, u8) = (255, 255, 255);

struct SdlDisplay {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
}

impl SdlDisplay {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;

    fn new(canvas: sdl2::render::WindowCanvas) -> Self {
        let mut canvas = canvas;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let texture_creator = Rc::new(canvas.texture_creator());
        let mut data = [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4];
        let surface = Surface::from_data(
            &mut data[..],
            Self::WIDTH,
            Self::HEIGHT,
            Self::WIDTH * 4,
            PixelFormatEnum::RGB24,
        )
        .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        canvas.clear();
        Self {
            canvas,
            texture: unsafe { std::mem::transmute(texture) },
            _texture_creator: texture_creator,
        }
    }
}

impl Display for SdlDisplay {
    fn display(&mut self, frame: Box<Frame>) {
        self.canvas.clear();

        let mut data = [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4];
        for y in 0..Self::HEIGHT as usize {
            for x in 0..Self::WIDTH as usize {
                data[4 * (y * Self::WIDTH as usize + x) + 0] = frame[y][x].2;
                data[4 * (y * Self::WIDTH as usize + x) + 1] = frame[y][x].1;
                data[4 * (y * Self::WIDTH as usize + x) + 2] = frame[y][x].0;
            }
        }

        self.texture
            .update(
                Rect::new(0, 0, Self::WIDTH, Self::HEIGHT),
                &data,
                Self::WIDTH as usize * 4,
            )
            .unwrap();

        let display_size = self.canvas.window().size();

        let frame_ratio = Self::WIDTH as f32 / Self::HEIGHT as f32;
        let display_ratio = display_size.0 as f32 / display_size.1 as f32;

        let frame_rect = Rect::new(0, 0, Self::WIDTH, Self::HEIGHT);
        let scaled_frame_rect;

        if frame_ratio > display_ratio {
            let scale = display_size.0 as f32 / Self::WIDTH as f32;
            let scaled_frame_size = (display_size.0, (Self::HEIGHT as f32 * scale) as u32);
            let scaled_frame_pos = (0, (display_size.1 - scaled_frame_size.1) as i32 / 2);
            scaled_frame_rect = Rect::new(
                scaled_frame_pos.0,
                scaled_frame_pos.1,
                scaled_frame_size.0,
                scaled_frame_size.1,
            );
        } else {
            let scale = display_size.1 as f32 / Self::HEIGHT as f32;
            let scaled_frame_size = ((Self::WIDTH as f32 * scale) as u32, display_size.1);
            let scaled_frame_pos = ((display_size.0 - scaled_frame_size.0) as i32 / 2, 0);
            scaled_frame_rect = Rect::new(
                scaled_frame_pos.0,
                scaled_frame_pos.1,
                scaled_frame_size.0,
                scaled_frame_size.1,
            );
        }

        self.canvas
            .copy(&self.texture, frame_rect, scaled_frame_rect)
            .unwrap();
        self.canvas.present();
    }
}

#[derive(Clone, Copy, Debug)]
enum InstructionTableValue {
    Opcode(Operation, AddressingMode),
    Value(u8),
    Unknown,
}

impl InstructionTableValue {
    fn unwrap_value(self) -> u8 {
        match self {
            Self::Opcode(_, _) => panic!("Tried unwrapping opcode"),
            Self::Value(value) => value,
            Self::Unknown => panic!("Tried unwrapping unknown"),
        }
    }

    fn unwrap_opcode(self) -> (Operation, AddressingMode) {
        match self {
            Self::Opcode(operation, mode) => (operation, mode),
            Self::Value(_) => panic!("Tried unwrapping value"),
            Self::Unknown => panic!("Tried unwrapping unknown"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum AddressingMode {
    Accumulator,
    Absolute,
    AbsoluteXIndexed,
    AbsoluteYIndexed,
    Immediate,
    Implied,
    Indirect,
    XIndexedIndirect,
    IndirectYIndexed,
    Relative,
    Zeropage,
    ZeropageXIndexed,
    ZeropageYIndexed,
}

impl AddressingMode {
    fn instruction_bytes(self) -> usize {
        use AddressingMode::*;
        match self {
            Accumulator | Implied => 1,
            Immediate | XIndexedIndirect | IndirectYIndexed | Relative | Zeropage
            | ZeropageXIndexed | ZeropageYIndexed => 2,
            Absolute | AbsoluteXIndexed | AbsoluteYIndexed | Indirect => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Operation {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    XXX,
}

impl Operation {
    fn color(self) -> (u8, u8, u8) {
        use Operation::*;
        match self {
            // load and store
            LDA | LDX | LDY | STA | STX | STY => GREEN,
            // transfer
            TAX | TAY | TSX | TXA | TXS | TYA => YELLOW,
            // stack
            PHA | PHP | PLA | PLP => BLUE,
            // shift
            ASL | LSR | ROL | ROR => CYAN,
            // logic
            AND | BIT | EOR | ORA => CYAN,
            // arithmetic
            ADC | CMP | CPX | CPY | SBC => WHITE,
            // increment and decrement
            DEC | DEX | DEY | INC | INX | INY => WHITE,
            // control
            BRK | JMP | JSR | RTI | RTS => RED,
            // branch
            BCC | BCS | BEQ | BMI | BNE | BPL | BVC | BVS => RED,
            // flags
            CLC | CLD | CLI | CLV | SEC | SED | SEI => MAGENTA,
            // nop
            NOP | XXX => RED,
        }
    }
}

struct SdlDebugDisplay {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    draw_buffer: [u8; Self::WIDTH as usize * Self::HEIGHT as usize * 4],
    disassembly: [InstructionTableValue; 1 << 16],
}

impl SdlDebugDisplay {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;

    #[rustfmt::skip]
    const DISASSEMBLY_TABLE: [(Operation, AddressingMode); 256] = {
        use Operation::*;
        use AddressingMode::*;
        [
            /* 0 */ (BRK, Implied),   (ORA, XIndexedIndirect), (XXX, Implied),   (XXX, Implied), (XXX, Implied),          (ORA, Zeropage),         (ASL, Zeropage),         (XXX, Implied), (PHP, Implied), (ORA, Immediate),        (ASL, Accumulator), (XXX, Implied), (XXX, Implied),          (ORA, Absolute),         (ASL, Absolute),         (XXX, Implied),
            /* 1 */ (BPL, Relative),  (ORA, IndirectYIndexed), (XXX, Implied),   (XXX, Implied), (XXX, Implied),          (ORA, ZeropageXIndexed), (ASL, ZeropageXIndexed), (XXX, Implied), (CLC, Implied), (ORA, AbsoluteYIndexed), (XXX, Implied),     (XXX, Implied), (XXX, Implied),          (ORA, AbsoluteXIndexed), (ASL, AbsoluteXIndexed), (XXX, Implied),
            /* 2 */ (JSR, Absolute),  (AND, XIndexedIndirect), (XXX, Implied),   (XXX, Implied), (BIT, Zeropage),         (AND, Zeropage),         (ROL, Zeropage),         (XXX, Implied), (PLP, Implied), (AND, Immediate),        (ROL, Accumulator), (XXX, Implied), (BIT, Absolute),         (AND, Absolute),         (ROL, Absolute),         (XXX, Implied),
            /* 3 */ (BMI, Relative),  (AND, IndirectYIndexed), (XXX, Implied),   (XXX, Implied), (XXX, Implied),          (AND, ZeropageXIndexed), (ROL, ZeropageXIndexed), (XXX, Implied), (SEC, Implied), (AND, AbsoluteYIndexed), (XXX, Implied),     (XXX, Implied), (XXX, Implied),          (AND, AbsoluteXIndexed), (ROL, AbsoluteXIndexed), (XXX, Implied),
            /* 4 */ (RTI, Implied),   (EOR, XIndexedIndirect), (XXX, Implied),   (XXX, Implied), (XXX, Implied),          (EOR, Zeropage),         (LSR, Zeropage),         (XXX, Implied), (PHA, Implied), (EOR, Immediate),        (LSR, Accumulator), (XXX, Implied), (JMP, Absolute),         (EOR, Absolute),         (LSR, Absolute),         (XXX, Implied),
            /* 5 */ (BVC, Relative),  (EOR, IndirectYIndexed), (XXX, Implied),   (XXX, Implied), (XXX, Implied),          (EOR, ZeropageXIndexed), (LSR, ZeropageXIndexed), (XXX, Implied), (CLI, Implied), (EOR, AbsoluteYIndexed), (XXX, Implied),     (XXX, Implied), (XXX, Implied),          (EOR, AbsoluteXIndexed), (LSR, AbsoluteXIndexed), (XXX, Implied),
            /* 6 */ (RTS, Implied),   (ADC, XIndexedIndirect), (XXX, Implied),   (XXX, Implied), (XXX, Implied),          (ADC, Zeropage),         (ROR, Zeropage),         (XXX, Implied), (PLA, Implied), (ADC, Immediate),        (ROR, Accumulator), (XXX, Implied), (JMP, Indirect),         (ADC, Absolute),         (ROR, Absolute),         (XXX, Implied),
            /* 7 */ (BVS, Relative),  (ADC, IndirectYIndexed), (XXX, Implied),   (XXX, Implied), (XXX, Implied),          (ADC, ZeropageXIndexed), (ROR, ZeropageXIndexed), (XXX, Implied), (SEI, Implied), (ADC, AbsoluteYIndexed), (XXX, Implied),     (XXX, Implied), (XXX, Implied),          (ADC, AbsoluteXIndexed), (ROR, AbsoluteXIndexed), (XXX, Implied),
            /* 8 */ (XXX, Implied),   (STA, XIndexedIndirect), (XXX, Implied),   (XXX, Implied), (STY, Zeropage),         (STA, Zeropage),         (STX, Zeropage),         (XXX, Implied), (DEY, Implied), (XXX, Implied),          (TXA, Implied),     (XXX, Implied), (STY, Absolute),         (STA, Absolute),         (STX, Absolute),         (XXX, Implied),
            /* 9 */ (BCC, Relative),  (STA, IndirectYIndexed), (XXX, Implied),   (XXX, Implied), (STY, ZeropageXIndexed), (STA, ZeropageXIndexed), (STX, ZeropageYIndexed), (XXX, Implied), (TYA, Implied), (STA, AbsoluteYIndexed), (TXS, Implied),     (XXX, Implied), (XXX, Implied),          (STA, AbsoluteXIndexed), (XXX, Implied),          (XXX, Implied),
            /* A */ (LDY, Immediate), (LDA, XIndexedIndirect), (LDX, Immediate), (XXX, Implied), (LDY, Zeropage),         (LDA, Zeropage),         (LDX, Zeropage),         (XXX, Implied), (TAY, Implied), (LDA, Immediate),        (TAX, Implied),     (XXX, Implied), (LDY, Absolute),         (LDA, Absolute),         (LDX, Absolute),         (XXX, Implied),
            /* B */ (BCS, Relative),  (LDA, IndirectYIndexed), (XXX, Implied),   (XXX, Implied), (LDY, ZeropageXIndexed), (LDA, ZeropageXIndexed), (LDX, ZeropageYIndexed), (XXX, Implied), (CLV, Implied), (LDA, AbsoluteYIndexed), (TSX, Implied),     (XXX, Implied), (LDY, AbsoluteXIndexed), (LDA, AbsoluteXIndexed), (LDX, AbsoluteYIndexed), (XXX, Implied),
            /* C */ (CPY, Immediate), (CMP, XIndexedIndirect), (XXX, Implied),   (XXX, Implied), (CPY, Zeropage),         (CMP, Zeropage),         (DEC, Zeropage),         (XXX, Implied), (INY, Implied), (CMP, Immediate),        (DEX, Implied),     (XXX, Implied), (CPY, Absolute),         (CMP, Absolute),         (DEC, Absolute),         (XXX, Implied),
            /* D */ (BNE, Relative),  (CMP, IndirectYIndexed), (XXX, Implied),   (XXX, Implied), (XXX, Implied),          (CMP, ZeropageXIndexed), (DEC, ZeropageXIndexed), (XXX, Implied), (CLD, Implied), (CMP, AbsoluteYIndexed), (XXX, Implied),     (XXX, Implied), (XXX, Implied),          (CMP, AbsoluteXIndexed), (DEC, AbsoluteXIndexed), (XXX, Implied),
            /* E */ (CPX, Immediate), (SBC, XIndexedIndirect), (XXX, Implied),   (XXX, Implied), (CPX, Zeropage),         (SBC, Zeropage),         (INC, Zeropage),         (XXX, Implied), (INX, Implied), (SBC, Immediate),        (NOP, Implied),     (XXX, Implied), (CPX, Absolute),         (SBC, Absolute),         (INC, Absolute),         (XXX, Implied),
            /* F */ (BEQ, Relative),  (SBC, IndirectYIndexed), (XXX, Implied),   (XXX, Implied), (XXX, Implied),          (SBC, ZeropageXIndexed), (INC, ZeropageXIndexed), (XXX, Implied), (SED, Implied), (SBC, AbsoluteYIndexed), (XXX, Implied),     (XXX, Implied), (XXX, Implied),          (SBC, AbsoluteXIndexed), (INC, AbsoluteXIndexed), (XXX, Implied),
        ]
    };

    fn new(canvas: sdl2::render::WindowCanvas) -> Self {
        let mut canvas = canvas;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let texture_creator = Rc::new(canvas.texture_creator());
        let mut data = [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4];
        let surface = Surface::from_data(
            &mut data[..],
            Self::WIDTH,
            Self::HEIGHT,
            Self::WIDTH * 4,
            PixelFormatEnum::RGB24,
        )
        .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        canvas.clear();
        canvas.present();
        Self {
            canvas,
            texture: unsafe { std::mem::transmute(texture) },
            _texture_creator: texture_creator,
            draw_buffer: [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4],
            disassembly: [InstructionTableValue::Unknown; 1 << 16],
        }
    }

    fn write_char_with_color(&mut self, value: char, line: u8, col: u8, color: (u8, u8, u8)) {
        if (line as u32) < Self::HEIGHT / 8 && (col as u32) < Self::WIDTH / 8 {
            let (char_row, char_col) = match value {
                _c @ 'A'..='O' => (0, 1 + (value as u8 - 'A' as u8)),
                _c @ 'a'..='o' => (0, 1 + (value as u8 - 'a' as u8)),
                _c @ 'P'..='Z' => (1, value as u8 - 'P' as u8),
                _c @ 'p'..='z' => (1, value as u8 - 'p' as u8),
                _c @ '0'..='9' => (3, value as u8 - '0' as u8),
                '@' => (0, 0),
                '[' => (1, 11),
                '£' => (1, 12),
                ']' => (1, 13),
                '↑' => (1, 14),
                '←' => (1, 15),
                ' ' => (2, 0),
                '!' => (2, 1),
                '"' => (2, 2),
                '#' => (2, 3),
                '$' => (2, 4),
                '%' => (2, 5),
                '&' => (2, 6),
                '\'' => (2, 7),
                '(' => (2, 8),
                ')' => (2, 9),
                '*' => (2, 10),
                '+' => (2, 11),
                ',' => (2, 12),
                '-' => (2, 13),
                '.' => (2, 14),
                '/' => (2, 15),
                ':' => (3, 10),
                ';' => (3, 11),
                '<' => (3, 12),
                '=' => (3, 13),
                '>' => (3, 14),
                '?' => (3, 15),
                _ => (3, 15),
            };

            for y in 0..8 {
                let mut row = FONT[((char_row as usize * 8) + y) * 16 + char_col as usize];
                for x in 0..8 {
                    let (r, g, b) = if row & 0b10000000 > 0 {
                        color
                    } else {
                        (0, 0, 0)
                    };
                    let line = line as usize;
                    let col = col as usize;
                    let pixel_start =
                        4 * ((line * 8 + y) * Self::WIDTH as usize + (col * 8) + x) + 0;
                    self.draw_buffer[pixel_start + 0] = b;
                    self.draw_buffer[pixel_start + 1] = g;
                    self.draw_buffer[pixel_start + 2] = r;
                    row = row << 1;
                }
            }
        }
    }

    fn write_str_with_color(&mut self, value: &str, line: u8, col: u8, color: (u8, u8, u8)) {
        let mut col = col;
        for c in value.chars() {
            self.write_char_with_color(c, line, col, color);
            col = col.saturating_add(1);
        }
    }

    fn write_u8_with_color(&mut self, value: u8, line: u8, col: u8, color: (u8, u8, u8)) {
        let hex2 = value >> 4 & 0b1111;
        let hex1 = value >> 0 & 0b1111;
        let num_to_char = |c| {
            char::from_u32(if c < 10 {
                '0' as u32 + c as u32
            } else {
                'A' as u32 + c as u32 - 10
            })
            .unwrap()
        };
        self.write_char_with_color(num_to_char(hex2), line, col, color);
        self.write_char_with_color(num_to_char(hex1), line, col.saturating_add(1), color);
    }

    fn write_u16_with_color(&mut self, value: u16, line: u8, col: u8, color: (u8, u8, u8)) {
        let hex4 = value >> 12 & 0b1111;
        let hex3 = value >> 8 & 0b1111;
        let hex2 = value >> 4 & 0b1111;
        let hex1 = value >> 0 & 0b1111;
        let num_to_char = |c| {
            char::from_u32(if c < 10 {
                '0' as u32 + c as u32
            } else {
                'A' as u32 + c as u32 - 10
            })
            .unwrap()
        };
        self.write_char_with_color(num_to_char(hex4), line, col, color);
        self.write_char_with_color(num_to_char(hex3), line, col.saturating_add(1), color);
        self.write_char_with_color(num_to_char(hex2), line, col.saturating_add(2), color);
        self.write_char_with_color(num_to_char(hex1), line, col.saturating_add(3), color);
    }

    fn write_bool_with_color(&mut self, value: bool, line: u8, col: u8, color: (u8, u8, u8)) {
        self.write_char_with_color(
            char::from_u32('0' as u32 + value as u32).unwrap(),
            line,
            col,
            color,
        );
    }

    fn write_instruction_from_disassembly(
        &mut self,
        address: u16,
        line: u8,
        col: u8,
        color: (u8, u8, u8),
    ) {
        let (operation, mode) = self.disassembly[address as usize].unwrap_opcode();

        let operation_str = format!("{:?}", operation);
        self.write_u16_with_color(address, line, col, color);
        self.write_str_with_color(&operation_str, line, col + 5, operation.color());

        // todo handle the case when next byte overflows address
        match mode {
            AddressingMode::Accumulator => {
                self.write_char_with_color('A', line, col + 9, RED);
            }
            AddressingMode::Absolute => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                self.write_char_with_color('$', line, col + 9, YELLOW);
                self.write_u16_with_color((low as u16) << 8 | high as u16, line, col + 10, WHITE);
            }
            AddressingMode::AbsoluteXIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                self.write_char_with_color('$', line, col + 9, YELLOW);
                self.write_u16_with_color((low as u16) << 8 | high as u16, line, col + 10, WHITE);
                self.write_str_with_color(",X", line, col + 14, RED);
            }
            AddressingMode::AbsoluteYIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                self.write_char_with_color('$', line, col + 9, YELLOW);
                self.write_u16_with_color((low as u16) << 8 | high as u16, line, col + 10, WHITE);
                self.write_str_with_color(",Y", line, col + 14, RED);
            }
            AddressingMode::Immediate => {
                let byte = self.disassembly[(address + 1) as usize].unwrap_value();
                self.write_char_with_color('#', line, col + 9, RED);
                self.write_char_with_color('$', line, col + 10, YELLOW);
                self.write_u8_with_color(byte, line, col + 11, WHITE);
            }
            AddressingMode::Implied => {}
            AddressingMode::Indirect => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                self.write_char_with_color('(', line, col + 9, RED);
                self.write_char_with_color('$', line, col + 10, YELLOW);
                self.write_u16_with_color((low as u16) << 8 | high as u16, line, col + 11, WHITE);
                self.write_char_with_color(')', line, col + 15, RED);
            }
            AddressingMode::XIndexedIndirect => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                self.write_char_with_color('(', line, col + 9, RED);
                self.write_char_with_color('$', line, col + 10, YELLOW);
                self.write_u8_with_color(low, line, col + 11, WHITE);
                self.write_str_with_color(",X)", line, col + 13, RED);
            }
            AddressingMode::IndirectYIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                self.write_char_with_color('(', line, col + 9, RED);
                self.write_char_with_color('$', line, col + 10, YELLOW);
                self.write_u8_with_color(low, line, col + 11, WHITE);
                self.write_str_with_color("),Y", line, col + 13, RED);
            }
            AddressingMode::Relative => {
                // TODO has the same syntax as zeropage, maybe indicate which one is it?
                let byte = self.disassembly[(address + 1) as usize].unwrap_value();
                self.write_char_with_color('$', line, col + 9, YELLOW);
                self.write_u8_with_color(byte, line, col + 10, WHITE);
            }
            AddressingMode::Zeropage => {
                // TODO has the same syntax as relative, maybe indicate which one is it?
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                self.write_char_with_color('$', line, col + 9, YELLOW);
                self.write_u8_with_color(low, line, col + 10, WHITE);
            }
            AddressingMode::ZeropageXIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                self.write_char_with_color('$', line, col + 9, YELLOW);
                self.write_u8_with_color(low, line, col + 10, WHITE);
                self.write_str_with_color(",X", line, col + 12, RED);
            }
            AddressingMode::ZeropageYIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                self.write_char_with_color('$', line, col + 9, YELLOW);
                self.write_u8_with_color(low, line, col + 10, WHITE);
                self.write_str_with_color(",Y", line, col + 12, RED);
            }
        }
    }

    fn fill_instructions(&mut self, cpu: &cpu::Cpu, nes: &Nes) {
        let mut position = cpu.program_counter;
        let mut checked_count = 0;

        while let InstructionTableValue::Unknown = self.disassembly[position as usize] {
            if checked_count == 16 {
                break;
            }
            let opcode = nes.cpu_bus_read(position);
            let (operation, mode) = Self::DISASSEMBLY_TABLE[opcode as usize];

            if operation == Operation::XXX {
                break;
            }

            self.disassembly[position as usize] = InstructionTableValue::Opcode(operation, mode);
            use AddressingMode::*;
            match mode {
                Accumulator | Implied => {
                    position += 1;
                }
                Immediate | XIndexedIndirect | IndirectYIndexed | Relative | Zeropage
                | ZeropageXIndexed | ZeropageYIndexed => {
                    self.disassembly[(position + 1) as usize] =
                        InstructionTableValue::Value(nes.cpu_bus_read(position + 1));
                    position += 2;
                }
                Absolute | AbsoluteXIndexed | AbsoluteYIndexed | Indirect => {
                    self.disassembly[(position + 1) as usize] =
                        InstructionTableValue::Value(nes.cpu_bus_read(position + 1));
                    self.disassembly[(position + 2) as usize] =
                        InstructionTableValue::Value(nes.cpu_bus_read(position + 2));
                    position += 3;
                }
            }

            checked_count += 1;
        }
    }
}

impl DebugDisplay for SdlDebugDisplay {
    fn display(&mut self, nes: &Nes) {
        let cpu_ref = nes.cpu.borrow();
        let cpu = &*cpu_ref;

        self.canvas.clear();
        self.draw_buffer.iter_mut().for_each(|byte| *byte = 0);

        self.fill_instructions(cpu, nes);

        self.write_str_with_color("A", 0, 1, YELLOW);
        self.write_u8_with_color(cpu.accumulator, 0, 3, WHITE);

        self.write_str_with_color("X", 1, 1, YELLOW);
        self.write_u8_with_color(cpu.x_index, 1, 3, WHITE);

        self.write_str_with_color("Y", 2, 1, YELLOW);
        self.write_u8_with_color(cpu.y_index, 2, 3, WHITE);

        self.write_str_with_color("SP", 3, 0, YELLOW);
        self.write_u8_with_color(cpu.stack_pointer, 3, 3, WHITE);

        self.write_str_with_color("PC", 4, 0, YELLOW);
        self.write_u16_with_color(cpu.program_counter, 4, 3, WHITE);

        self.write_str_with_color("SR", 5, 0, YELLOW);

        self.write_str_with_color("N", 6, 1, YELLOW);
        self.write_bool_with_color(cpu.status_register.get_negative(), 6, 3, WHITE);

        self.write_str_with_color("V", 7, 1, YELLOW);
        self.write_bool_with_color(cpu.status_register.get_overflow(), 7, 3, WHITE);

        self.write_str_with_color("-", 8, 1, YELLOW);
        self.write_bool_with_color(cpu.status_register.get_ignored(), 8, 3, WHITE);

        self.write_str_with_color("B", 9, 1, YELLOW);
        self.write_bool_with_color(cpu.status_register.get_break(), 9, 3, WHITE);

        self.write_str_with_color("D", 10, 1, YELLOW);
        self.write_bool_with_color(cpu.status_register.get_decimal(), 10, 3, WHITE);

        self.write_str_with_color("I", 11, 1, YELLOW);
        self.write_bool_with_color(cpu.status_register.get_interrupt(), 11, 3, WHITE);

        self.write_str_with_color("Z", 12, 1, YELLOW);
        self.write_bool_with_color(cpu.status_register.get_zero(), 12, 3, WHITE);

        self.write_str_with_color("C", 13, 1, YELLOW);
        self.write_bool_with_color(cpu.status_register.get_carry(), 13, 3, WHITE);

        let mut address = cpu.program_counter;
        let mut color = RED;
        let mut line = 14;

        loop {
            if let InstructionTableValue::Opcode(_, _) = self.disassembly[address as usize] {
                let effective_color = if address == cpu.program_counter {
                    YELLOW
                } else {
                    color
                };
                self.write_instruction_from_disassembly(address, line, 13, effective_color);
                color = WHITE;
                if line == 0 {
                    break;
                }
                line -= 1;
            }
            if address == 0 {
                break;
            }
            address -= 1;
        }

        if cpu.program_counter < u16::MAX {
            let mut address = cpu.program_counter + 1;
            let mut line = 15;

            loop {
                if let InstructionTableValue::Opcode(_, _) = self.disassembly[address as usize] {
                    self.write_instruction_from_disassembly(address, line, 13, WHITE);
                    if line == 31 {
                        break;
                    }
                    line += 1;
                }
                if address == u16::MAX {
                    break;
                }
                address += 1;
            }
        }

        self.texture
            .update(
                Rect::new(0, 0, Self::WIDTH, Self::HEIGHT),
                &self.draw_buffer,
                Self::WIDTH as usize * 4,
            )
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


struct SdlPpuDebugDisplay {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    draw_buffer: [u8; Self::WIDTH as usize * Self::HEIGHT as usize * 4],
    disassembly: [InstructionTableValue; 1 << 16],
}

impl SdlPpuDebugDisplay {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
}

struct SdlInput {

}

impl SdlInput {
    pub fn new() -> Self {
        Self {}
    }
}

impl Input for SdlInput {
    fn read(&mut self) -> nes::InputData {
        todo!()
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let file_contents = std::fs::read(args.get(1).expect("file argument missing")).expect("could not read the file");
    let game_file = GameFile::read(file_contents).expect("file does not contain a nes game");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("nes display", SdlDisplay::WIDTH * 3, SdlDisplay::HEIGHT * 3)
        .position_centered()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let window2 = video_subsystem
        .window(
            "cpu inspector",
            SdlDisplay::WIDTH * 3,
            SdlDisplay::HEIGHT * 3,
        )
        .position_centered()
        .build()
        .unwrap();

    let canvas2 = window2.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let display = SdlDisplay::new(canvas);
    let debug_display = SdlDebugDisplay::new(canvas2);
    let input = SdlInput::new();
    let mut nes = Nes::new(game_file, display, debug_display, input).expect("Could not start the game");

    'main_loop: loop {
        let start_time = std::time::Instant::now();

        for event in event_pump.poll_iter() {
            event.get_window_id();
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main_loop,
                _ => {}
            }
        }

        nes.run_one_cpu_instruction();

        // 1fps
        // let nanos_to_sleep = Duration::from_nanos(1_000_000_000).saturating_sub(start_time.elapsed());

        // 60fps
        let nanos_to_sleep = Duration::from_nanos(1_000_000_000u64 / 60).saturating_sub(start_time.elapsed());

        if nanos_to_sleep != Duration::ZERO {
            std::thread::sleep(nanos_to_sleep);
        }
    }
}

#[test]
fn disassebly_table_has_unique_elements() {
    let mut set = std::collections::BTreeSet::<(Operation, AddressingMode)>::new();

    for entry in SdlDebugDisplay::DISASSEMBLY_TABLE.iter() {
        if set.contains(entry) && entry != &(Operation::XXX, AddressingMode::Implied) {
            panic!("{:?} {:?} repeats", entry.0, entry.1);
        }
        set.insert(*entry);
    }
}
