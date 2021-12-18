use nes_lib::*;

use nes_lib::game_file::GameFile;
use nes_lib::nes::{Display, Frame, Input, Nes};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Write;
use std::ops::RangeInclusive;
use std::rc::Rc;
use std::time::Duration;

const FONT: &[u8; 16 * 32] = include_bytes!("../resources/comodore-64-font.bin");

#[derive(Clone, Copy)]
#[repr(u8)]
enum Color {
    Black = 0b000,
    Red = 0b100,
    Green = 0b010,
    Blue = 0b001,
    Yellow = 0b110,
    Cyan = 0b011,
    Magenta = 0b101,
    White = 0b111,
}
use Color::*;

struct TextArea<const W: usize, const H: usize> {
    buffer: [[(u8, Color); W]; H],
}

impl<const W: usize, const H: usize> TextArea<W, H> {
    fn new() -> Self {
        Self {
            buffer: [[(0b100000, Black); W]; H],
        }
    }

    fn clear(&mut self) {
        *self = Self::new();
    }

    fn draw_to_texture(&self, bytes: &mut [u8]) {
        let mut bytes_i = 0;
        for row in self.buffer {
            for scan in 0..8 {
                for (ch, color) in row {
                    let r = ((color as u8 & 0b100) >> 2) * 255;
                    let g = ((color as u8 & 0b010) >> 1) * 255;
                    let b = ((color as u8 & 0b001) >> 0) * 255;
                    let char_row = ch >> 4;
                    let char_col = ch & 0b1111;
                    let mut slice =
                        FONT[(((char_row as usize * 8) + scan) * 16) + char_col as usize];
                    for _ in 0..8 {
                        if slice & 0b10000000 > 0 {
                            bytes[bytes_i + 0] = b;
                            bytes[bytes_i + 1] = g;
                            bytes[bytes_i + 2] = r;
                        } else {
                            bytes[bytes_i + 0] = 0;
                            bytes[bytes_i + 1] = 0;
                            bytes[bytes_i + 2] = 0;
                        }
                        slice <<= 1;
                        bytes_i += 4;
                    }
                }
            }
        }
    }

    fn write_char_with_color(&mut self, value: char, line: u8, col: u8, color: Color) {
        if (line as usize) >= H || (col as usize) >= W {
            return;
        }

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

        self.buffer[line as usize][col as usize] = (char_row << 4 | char_col, color);
    }

    fn write_str_with_color(&mut self, value: &str, line: u8, col: u8, color: Color) {
        let mut col = col;
        for c in value.chars() {
            self.write_char_with_color(c, line, col, color);
            col = col.saturating_add(1);
        }
    }

    fn write_u8_with_color(&mut self, value: u8, line: u8, col: u8, color: Color) {
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

    fn write_u16_with_color(&mut self, value: u16, line: u8, col: u8, color: Color) {
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

    fn write_bool_with_color(&mut self, value: bool, line: u8, col: u8, color: Color) {
        self.write_char_with_color(
            char::from_u32('0' as u32 + value as u32).unwrap(),
            line,
            col,
            color,
        );
    }
}

#[derive(Clone)]
struct SdlDisplay {
    inner: Rc<RefCell<SdlDisplayInner>>,
}

struct SdlDisplayInner {
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
            inner: Rc::new(RefCell::new(SdlDisplayInner {
                canvas,
                texture: unsafe { std::mem::transmute(texture) },
                _texture_creator: texture_creator,
            })),
        }
    }

    fn handle_event(&mut self, event: Event, nes: &Nes, state: &mut EmulatorState) {
        match event {
            Event::KeyDown {
                keycode: k @ Some(Keycode::Escape),
                ..
            } => {
                state.exit = true;
            }
            Event::Quit { .. } => {
                state.exit = true;
            }
            _ => {}
        }
    }

    fn actual_display(&self) {
        let mut inner = self.inner.borrow_mut();

        inner.canvas.present();
    }
}

impl Display for SdlDisplay {
    fn display(&mut self, frame: Box<Frame>) {
        let mut inner = self.inner.borrow_mut();

        let mut data = [0; Self::WIDTH as usize * Self::HEIGHT as usize * 4];
        for y in 0..Self::HEIGHT as usize {
            for x in 0..Self::WIDTH as usize {
                data[4 * (y * Self::WIDTH as usize + x) + 0] = frame[y][x].2;
                data[4 * (y * Self::WIDTH as usize + x) + 1] = frame[y][x].1;
                data[4 * (y * Self::WIDTH as usize + x) + 2] = frame[y][x].0;
            }
        }

        inner
            .texture
            .update(
                Rect::new(0, 0, Self::WIDTH, Self::HEIGHT),
                &data,
                Self::WIDTH as usize * 4,
            )
            .unwrap();

        let display_size = inner.canvas.window().size();

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

        // TODO This deconstruction trick is smart as fuck, I have to write
        // a blog post about it.
        let SdlDisplayInner {
            canvas, texture, ..
        } = &mut *inner;
        canvas
            .copy(&texture, frame_rect, scaled_frame_rect)
            .unwrap();
    }
}

#[derive(Clone, Copy, Debug)]
enum DisassemblyValue {
    Opcode(Operation, AddressingMode),
    Value(u8),
    Unknown,
}

impl DisassemblyValue {
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
    fn color(self) -> Color {
        use Operation::*;
        match self {
            // load and store
            LDA | LDX | LDY | STA | STX | STY => Green,
            // transfer
            TAX | TAY | TSX | TXA | TXS | TYA => Yellow,
            // stack
            PHA | PHP | PLA | PLP => Blue,
            // shift
            ASL | LSR | ROL | ROR => Cyan,
            // logic
            AND | BIT | EOR | ORA => Cyan,
            // arithmetic
            ADC | CMP | CPX | CPY | SBC => White,
            // increment and decrement
            DEC | DEX | DEY | INC | INX | INY => White,
            // control
            BRK | JMP | JSR | RTI | RTS => Red,
            // branch
            BCC | BCS | BEQ | BMI | BNE | BPL | BVC | BVS => Red,
            // flags
            CLC | CLD | CLI | CLV | SEC | SED | SEI => Magenta,
            // nop
            NOP | XXX => Red,
        }
    }
}

struct SdlDebugDisplay {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    text_area: TextArea<{ Self::WIDTH as usize / 8 }, { Self::HEIGHT as usize / 8 }>,
    breakpoint_mode: bool,
    breakpoint_address: u16,
    breakpoint_pos: i8,
    breakpoints: Vec<u16>,
    disassembly: [DisassemblyValue; 1 << 16],
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
        let mut texture = texture_creator
            .create_texture_streaming(canvas.default_pixel_format(), Self::WIDTH, Self::HEIGHT)
            .unwrap();
        texture
            .with_lock(None, |data, _pitch| {
                for byte in data {
                    *byte = 0;
                }
            })
            .unwrap();
        canvas.clear();
        canvas.present();

        let breakpoints = std::fs::read_to_string("./breakpoints")
            .map(|s| deserialize_breakpoints(s))
            .unwrap_or(Vec::new());

        Self {
            canvas,
            texture: unsafe { std::mem::transmute(texture) },
            _texture_creator: texture_creator,
            breakpoint_mode: false,
            breakpoint_address: 0,
            breakpoint_pos: 0,
            breakpoints,
            text_area: TextArea::new(),
            disassembly: [DisassemblyValue::Unknown; 1 << 16],
        }
    }

    fn write_instruction_from_disassembly(
        &mut self,
        address: u16,
        line: u8,
        col: u8,
        color: Color,
    ) {
        use std::io::Write;
        use std::str;

        let (operation, mode) = self.disassembly[address as usize].unwrap_opcode();
        let ta = &mut self.text_area;

        match (
            self.breakpoint_mode,
            self.breakpoint_address == address,
            self.breakpoints.contains(&address),
        ) {
            (true, true, true) => {
                ta.write_char_with_color('B', line, col, Yellow);
            }
            (true, true, false) => {
                ta.write_char_with_color('*', line, col, Yellow);
            }
            (_, _, true) => {
                ta.write_char_with_color('B', line, col, Red);
            }
            _ => {}
        }

        let mut operation_str_buffer = [0u8; 3];
        write!(&mut operation_str_buffer[..], "{:?}", operation).unwrap();

        ta.write_u16_with_color(address, line, col + 2, color);
        ta.write_str_with_color(
            str::from_utf8(&operation_str_buffer[..]).unwrap(),
            line,
            col + 7,
            operation.color(),
        );

        // todo handle the case when next byte overflows address
        match mode {
            AddressingMode::Accumulator => {
                ta.write_char_with_color('A', line, col + 11, Red);
            }
            AddressingMode::Absolute => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                ta.write_char_with_color('$', line, col + 11, Yellow);
                ta.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 12, White);
            }
            AddressingMode::AbsoluteXIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                ta.write_char_with_color('$', line, col + 11, Yellow);
                ta.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 12, White);
                ta.write_str_with_color(",X", line, col + 16, Red);
            }
            AddressingMode::AbsoluteYIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                ta.write_char_with_color('$', line, col + 11, Yellow);
                ta.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 12, White);
                ta.write_str_with_color(",Y", line, col + 16, Red);
            }
            AddressingMode::Immediate => {
                let byte = self.disassembly[(address + 1) as usize].unwrap_value();
                ta.write_char_with_color('#', line, col + 11, Red);
                ta.write_char_with_color('$', line, col + 12, Yellow);
                ta.write_u8_with_color(byte, line, col + 13, White);
            }
            AddressingMode::Implied => {}
            AddressingMode::Indirect => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                ta.write_char_with_color('(', line, col + 11, Red);
                ta.write_char_with_color('$', line, col + 12, Yellow);
                ta.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 13, White);
                ta.write_char_with_color(')', line, col + 17, Red);
            }
            AddressingMode::XIndexedIndirect => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                ta.write_char_with_color('(', line, col + 11, Red);
                ta.write_char_with_color('$', line, col + 12, Yellow);
                ta.write_u8_with_color(low, line, col + 13, White);
                ta.write_str_with_color(",X)", line, col + 15, Red);
            }
            AddressingMode::IndirectYIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                ta.write_char_with_color('(', line, col + 11, Red);
                ta.write_char_with_color('$', line, col + 12, Yellow);
                ta.write_u8_with_color(low, line, col + 13, White);
                ta.write_str_with_color("),Y", line, col + 15, Red);
            }
            AddressingMode::Relative => {
                // TODO has the same syntax as zeropage, maybe indicate which one is it?
                let byte = self.disassembly[(address + 1) as usize].unwrap_value();
                ta.write_char_with_color('$', line, col + 11, Yellow);
                ta.write_u8_with_color(byte, line, col + 12, White);
            }
            AddressingMode::Zeropage => {
                // TODO has the same syntax as relative, maybe indicate which one is it?
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                ta.write_char_with_color('$', line, col + 11, Yellow);
                ta.write_u8_with_color(low, line, col + 12, White);
            }
            AddressingMode::ZeropageXIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                ta.write_char_with_color('$', line, col + 11, Yellow);
                ta.write_u8_with_color(low, line, col + 12, White);
                ta.write_str_with_color(",X", line, col + 14, Red);
            }
            AddressingMode::ZeropageYIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                ta.write_char_with_color('$', line, col + 11, Yellow);
                ta.write_u8_with_color(low, line, col + 12, White);
                ta.write_str_with_color(",Y", line, col + 14, Red);
            }
        }
    }

    fn fill_instructions(&mut self, nes: &Nes) {
        let cpu = nes.cpu.borrow();
        let mut position = cpu.program_counter;

        for _ in 0..16 {
            match self.disassembly[position as usize] {
                DisassemblyValue::Opcode(..) => {
                    position += 1;
                    if let DisassemblyValue::Value(..) = self.disassembly[position as usize] {
                        position += 1;
                        if let DisassemblyValue::Value(..) = self.disassembly[position as usize] {
                            position += 1;
                        }
                    }
                }
                DisassemblyValue::Value(..) => {
                    // program counter points to data, bad
                    break;
                }
                DisassemblyValue::Unknown => {
                    let opcode = nes.cpu_bus_read(position);
                    let (operation, mode) = Self::DISASSEMBLY_TABLE[opcode as usize];

                    if operation == Operation::XXX {
                        break;
                    }

                    self.disassembly[position as usize] = DisassemblyValue::Opcode(operation, mode);

                    use AddressingMode::*;
                    match mode {
                        Accumulator | Implied => {
                            position += 1;
                        }
                        Immediate | XIndexedIndirect | IndirectYIndexed | Relative | Zeropage
                        | ZeropageXIndexed | ZeropageYIndexed => {
                            self.disassembly[(position + 1) as usize] =
                                DisassemblyValue::Value(nes.cpu_bus_read(position + 1));
                            position += 2;
                        }
                        Absolute | AbsoluteXIndexed | AbsoluteYIndexed | Indirect => {
                            self.disassembly[(position + 1) as usize] =
                                DisassemblyValue::Value(nes.cpu_bus_read(position + 1));
                            self.disassembly[(position + 2) as usize] =
                                DisassemblyValue::Value(nes.cpu_bus_read(position + 2));
                            position += 3;
                        }
                    }
                }
            }
        }
    }

    fn handle_event(&mut self, event: Event, nes: &Nes, state: &mut EmulatorState) {
        if self.breakpoint_mode {
            match event {
                Event::Quit { .. } => {
                    state.exit = true;
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::Escape),
                    ..
                } => {
                    self.breakpoint_mode = false;
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::B),
                    ..
                } => {
                    if self.breakpoints.contains(&self.breakpoint_address) {
                        self.breakpoints.retain(|a| *a != self.breakpoint_address);
                    } else {
                        self.breakpoints.push(self.breakpoint_address);
                    }
                    // TODO add game name to the breakpoints file name
                    let _ = std::fs::write(
                        format!("./breakpoints"),
                        serialize_breakpoints(&self.breakpoints),
                    );
                    self.breakpoint_mode = false;
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::Down),
                    ..
                } => {
                    let mut address = self.breakpoint_address;
                    if self.breakpoint_pos < 15 {
                        for _ in 0..3 {
                            address += 1;
                            match self.disassembly[address as usize] {
                                DisassemblyValue::Opcode(..) => {
                                    self.breakpoint_address = address;
                                    self.breakpoint_pos += 1;
                                    break;
                                }
                                DisassemblyValue::Value(..) => continue,
                                DisassemblyValue::Unknown => break,
                            }
                        }
                    }
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::Up),
                    ..
                } => {
                    let mut address = self.breakpoint_address;
                    if self.breakpoint_pos > -10 {
                        for _ in 0..3 {
                            address -= 1;
                            match self.disassembly[address as usize] {
                                DisassemblyValue::Opcode(..) => {
                                    self.breakpoint_address = address;
                                    self.breakpoint_pos -= 1;
                                    break;
                                }
                                DisassemblyValue::Value(..) => continue,
                                DisassemblyValue::Unknown => break,
                            }
                        }
                    }
                }
                _ => {}
            }
        } else {
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
                    keycode: _k @ Some(Keycode::B),
                    ..
                } => {
                    let pc = nes.cpu.borrow().program_counter;
                    match self.disassembly[pc as usize] {
                        DisassemblyValue::Opcode(..) => {
                            self.breakpoint_address = pc;
                            self.breakpoint_pos = 0;
                            self.breakpoint_mode = true;
                        }
                        _ => {}
                    }
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::Return),
                    ..
                } => {
                    state.one_step = true;
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::Space),
                    ..
                } => {
                    state.running = !state.running;
                }
                _ => {}
            }
        }
    }

    fn update(&mut self, nes: &Nes) {
        self.fill_instructions(nes);
    }

    fn show(&mut self, nes: &Nes) {
        self.canvas.clear();
        self.text_area.clear();
        let cpu = nes.cpu.borrow();
        let ta = &mut self.text_area;

        ta.write_str_with_color("A", 0, 1, Yellow);
        ta.write_u8_with_color(cpu.accumulator, 0, 3, White);

        ta.write_str_with_color("X", 1, 1, Yellow);
        ta.write_u8_with_color(cpu.x_index, 1, 3, White);

        ta.write_str_with_color("Y", 2, 1, Yellow);
        ta.write_u8_with_color(cpu.y_index, 2, 3, White);

        ta.write_str_with_color("SP", 3, 0, Yellow);
        ta.write_u8_with_color(cpu.stack_pointer, 3, 3, White);

        ta.write_str_with_color("PC", 4, 0, Yellow);
        ta.write_u16_with_color(cpu.program_counter, 4, 3, White);

        ta.write_str_with_color("SR", 5, 0, Yellow);

        ta.write_str_with_color("N", 6, 1, Yellow);
        ta.write_bool_with_color(cpu.status_register.get_negative(), 6, 3, White);

        ta.write_str_with_color("V", 7, 1, Yellow);
        ta.write_bool_with_color(cpu.status_register.get_overflow(), 7, 3, White);

        ta.write_str_with_color("-", 8, 1, Yellow);
        ta.write_bool_with_color(cpu.status_register.get_ignored(), 8, 3, White);

        ta.write_str_with_color("B", 9, 1, Yellow);
        ta.write_bool_with_color(cpu.status_register.get_break(), 9, 3, White);

        ta.write_str_with_color("D", 10, 1, Yellow);
        ta.write_bool_with_color(cpu.status_register.get_decimal(), 10, 3, White);

        ta.write_str_with_color("I", 11, 1, Yellow);
        ta.write_bool_with_color(cpu.status_register.get_interrupt(), 11, 3, White);

        ta.write_str_with_color("Z", 12, 1, Yellow);
        ta.write_bool_with_color(cpu.status_register.get_zero(), 12, 3, White);

        ta.write_str_with_color("C", 13, 1, Yellow);
        ta.write_bool_with_color(cpu.status_register.get_carry(), 13, 3, White);

        if self.breakpoint_mode {
            ta.write_char_with_color('B', 29, 0, Red);
        }

        let mut address = cpu.program_counter;
        let mut color = Red;
        let mut line = 14;

        loop {
            match self.disassembly[address as usize] {
                DisassemblyValue::Opcode(..) => {
                    let effective_color = if address == cpu.program_counter {
                        Yellow
                    } else {
                        color
                    };
                    self.write_instruction_from_disassembly(address, line, 11, effective_color);
                    color = White;
                    if line == 0 {
                        break;
                    }
                    line -= 1;
                }
                DisassemblyValue::Value(..) => {}
                DisassemblyValue::Unknown => {
                    break;
                }
            }
            if address == 0 {
                break;
            }
            address -= 1;
        }

        if cpu.program_counter < u16::MAX {
            let mut line = 15;

            for address in (cpu.program_counter + 1)..=u16::MAX {
                match self.disassembly[address as usize] {
                    DisassemblyValue::Opcode(..) => {
                        self.write_instruction_from_disassembly(address, line, 11, White);
                        if line == 31 {
                            break;
                        }
                        line += 1;
                    }
                    DisassemblyValue::Value(..) => {}
                    DisassemblyValue::Unknown => {
                        break;
                    }
                }
            }
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

fn serialize_breakpoints(breakpoints: &[u16]) -> String {
    let mut output = String::new();
    for b in breakpoints.iter() {
        write!(&mut output, "{:04X}\n", *b).unwrap();
    }
    output
}

fn deserialize_breakpoints(string: String) -> Vec<u16> {
    string
        .split('\n')
        .filter(|line| !line.is_empty())
        .filter_map(|line| u16::from_str_radix(line, 16).ok())
        .collect()
}

struct SdlMemoryDisplay {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    text_area: TextArea<{ Self::WIDTH as usize / 8 }, { Self::HEIGHT as usize / 8 }>,
    page: u8,
}

impl SdlMemoryDisplay {
    const WIDTH: u32 = 384;
    const HEIGHT: u32 = 360;

    fn new(canvas: sdl2::render::WindowCanvas) -> Self {
        let mut canvas = canvas;
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        let texture_creator = Rc::new(canvas.texture_creator());
        let mut texture = texture_creator
            .create_texture_streaming(canvas.default_pixel_format(), Self::WIDTH, Self::HEIGHT)
            .unwrap();
        texture
            .with_lock(None, |data, _pitch| {
                for byte in data {
                    *byte = 0;
                }
            })
            .unwrap();
        canvas.clear();
        canvas.present();

        Self {
            canvas,
            texture: unsafe { std::mem::transmute(texture) },
            _texture_creator: texture_creator,
            text_area: TextArea::new(),
            page: 0,
        }
    }

    fn update(&mut self, nes: &Nes) {}

    fn show(&mut self, nes: &Nes) {
        let start_address = 256u16 * self.page as u16;
        let ta = &mut self.text_area;

        ta.write_str_with_color("START", 0, 0, Yellow);
        ta.write_u16_with_color(start_address, 0, 6, White);
        ta.write_str_with_color("< >", 0, 11, Yellow);

        self.text_area.write_str_with_color(
            " 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F",
            2,
            0,
            Yellow,
        );

        for row in 0..16u8 {
            self.text_area.write_char_with_color(
                char::from_u32(if row < 10 {
                    '0' as u32 + row as u32
                } else {
                    'A' as u32 + row as u32 - 10
                })
                .unwrap(),
                3 + row * 2,
                0,
                Yellow,
            );
        }

        for y in 0..16u8 {
            for x in 0..16u8 {
                self.text_area.write_u8_with_color(
                    nes.cpu_bus_read(start_address + (y as u16 * 16) + x as u16),
                    3 + y * 2,
                    1 + x * 3,
                    if x % 2 == 0 { White } else { Cyan },
                );
            }
        }

        if start_address == 0x0100 {
            let sp = nes.cpu.borrow().stack_pointer;
            let y = sp >> 4;
            let x = sp & 0x0F;
            self.text_area.write_u8_with_color(
                nes.cpu_bus_read(start_address + (y as u16 * 16) + x as u16),
                3 + y * 2,
                1 + x * 3,
                Magenta,
            );
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

    fn handle_event(&mut self, event: Event, nes: &Nes, state: &mut EmulatorState) {
        let page_ranges = [0x00..=0x19, 0x80..=0xFF];
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
                keycode: _k @ Some(Keycode::Up),
                ..
            } => {
                self.page = increase_in_ranges(&page_ranges, self.page);
            }
            Event::KeyDown {
                keycode: _k @ Some(Keycode::Down),
                ..
            } => {
                self.page = decrease_in_ranges(&page_ranges, self.page);
            }
            _ => {}
        }
    }
}

fn decrease_in_ranges(ranges: &[RangeInclusive<u8>], value: u8) -> u8 {
    for i in 0..ranges.len() {
        if ranges[i].contains(&value) {
            return if value > *ranges[i].start() {
                value - 1
            } else if i > 0 {
                *ranges[i - 1].end()
            } else {
                value
            };
        }
    }
    value
}

fn increase_in_ranges(ranges: &[RangeInclusive<u8>], value: u8) -> u8 {
    for i in 0..ranges.len() {
        if ranges[i].contains(&value) {
            return if value < *ranges[i].end() {
                value + 1
            } else if i < ranges.len() - 1 {
                *ranges[i + 1].start()
            } else {
                value
            };
        }
    }
    value
}

struct SdlPpuDebugDisplay {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    text_area: TextArea<{ Self::WIDTH as usize / 8 }, { Self::HEIGHT as usize / 8 }>,
}

impl SdlPpuDebugDisplay {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
}

impl SdlPpuDebugDisplay {
    fn new(canvas: sdl2::render::WindowCanvas) -> Self {
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
        }
    }

    fn display(&mut self, nes: &Nes) {
        self.canvas.clear();
        self.text_area.clear();
        let ta = &mut self.text_area;
        let ppu = nes.ppu.borrow();

        ta.write_str_with_color("SCANLINE", 0, 0, Yellow);
        ta.write_u16_with_color(ppu.scanline, 0, 9, White);

        ta.write_str_with_color("PIXEL", 1, 3, Yellow);
        ta.write_u16_with_color(ppu.pixel, 1, 9, White);

        ta.write_str_with_color("CTRL", 3, 2, Yellow);

        ta.write_str_with_color("NMI", 4, 3, Yellow);
        ta.write_bool_with_color(ppu.control_register.get_nmi_enable(), 4, 7, White);

        ta.write_str_with_color("M/S", 5, 3, Yellow);
        ta.write_bool_with_color(ppu.control_register.get_ppu_master_slave(), 5, 7, White);

        ta.write_str_with_color("HEIGHT", 6, 0, Yellow);
        ta.write_bool_with_color(ppu.control_register.get_sprite_height(), 6, 7, White);

        ta.write_str_with_color("BACK", 7, 2, Yellow);
        ta.write_bool_with_color(
            ppu.control_register.get_background_tile_select(),
            7,
            7,
            White,
        );

        ta.write_str_with_color("SPRITE", 8, 0, Yellow);
        ta.write_bool_with_color(ppu.control_register.get_sprite_tile_select(), 8, 7, White);

        ta.write_str_with_color("INC", 9, 3, Yellow);
        ta.write_bool_with_color(ppu.control_register.get_sprite_height(), 9, 7, White);

        ta.write_str_with_color("NTADDR", 10, 0, Yellow);
        ta.write_u8_with_color(ppu.control_register.get_name_table_address(), 10, 7, White);

        ta.write_str_with_color("MASK", 3, 11, Yellow);

        ta.write_str_with_color("BLUE", 4, 11, Yellow);
        ta.write_bool_with_color(ppu.mask_register.get_emphasize_blue(), 4, 16, White);

        ta.write_str_with_color("GREEN", 5, 10, Yellow);
        ta.write_bool_with_color(ppu.mask_register.get_emphasize_green(), 5, 16, White);

        ta.write_str_with_color("RED", 6, 12, Yellow);
        ta.write_bool_with_color(ppu.mask_register.get_emphasize_red(), 6, 16, White);

        ta.write_str_with_color("SPR", 7, 12, Yellow);
        ta.write_bool_with_color(ppu.mask_register.get_show_sprites(), 7, 16, White);

        ta.write_str_with_color("BAC", 8, 12, Yellow);
        ta.write_bool_with_color(ppu.mask_register.get_show_background(), 8, 16, White);

        ta.write_str_with_color("SPRL", 9, 11, Yellow);
        ta.write_bool_with_color(
            ppu.mask_register.get_show_sprites_in_leftmost_col(),
            9,
            16,
            White,
        );

        ta.write_str_with_color("BACL", 10, 11, Yellow);
        ta.write_bool_with_color(
            ppu.mask_register.get_show_background_in_leftmost_col(),
            10,
            16,
            White,
        );

        ta.write_str_with_color("STATUS", 3, 18, Yellow);

        ta.write_str_with_color("VBLANK", 4, 18, Yellow);
        ta.write_bool_with_color(ppu.status_register.get_vblank_flag(), 4, 25, White);

        ta.write_str_with_color("S0H", 5, 21, Yellow);
        ta.write_bool_with_color(ppu.status_register.get_hit_flag(), 5, 25, White);

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

    fn handle_event(&mut self, event: Event, nes: &Nes, state: &mut EmulatorState) {
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
}

struct SdlInput {}

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

struct EmulatorState {
    running: bool,
    exit: bool,
    one_step: bool,
    stop_on_program_counter: Option<u16>,
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let file_contents = std::fs::read(args.get(1).expect("file argument missing"))
        .expect("could not read the file");
    let game_file = GameFile::read(args.get(1).unwrap().to_string(), file_contents)
        .expect("file does not contain a nes game");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let display_window = video_subsystem
        .window("nes display", SdlDisplay::WIDTH * 3, SdlDisplay::HEIGHT * 3)
        .position(0, 720)
        .build()
        .unwrap();
    let display_window_id = display_window.id();
    let display_canvas = display_window.into_canvas().build().unwrap();

    let debug_window = video_subsystem
        .window(
            "nes cpu inspector",
            SdlDebugDisplay::WIDTH * 3,
            SdlDebugDisplay::HEIGHT * 3,
        )
        .position(768, 0)
        .build()
        .unwrap();
    let debug_window_id = debug_window.id();
    let debug_canvas = debug_window.into_canvas().build().unwrap();

    let ppu_debug_window = video_subsystem
        .window(
            "nes ppu inspector",
            SdlPpuDebugDisplay::WIDTH * 3,
            SdlPpuDebugDisplay::HEIGHT * 3,
        )
        .position(768, 720)
        .build()
        .unwrap();
    let ppu_debug_window_id = ppu_debug_window.id();
    let ppu_debug_canvas = ppu_debug_window.into_canvas().build().unwrap();

    let memory_window = video_subsystem
        .window(
            "nes memory inspector",
            SdlMemoryDisplay::WIDTH * 2,
            SdlMemoryDisplay::HEIGHT * 2,
        )
        .position(0, 1)
        .build()
        .unwrap();
    let memory_window_id = memory_window.id();
    let memory_canvas = memory_window.into_canvas().build().unwrap();

    let mut display = SdlDisplay::new(display_canvas);
    let mut debug_display = SdlDebugDisplay::new(debug_canvas);
    let mut memory_display = SdlMemoryDisplay::new(memory_canvas);
    let mut ppu_debug_display = SdlPpuDebugDisplay::new(ppu_debug_canvas);
    let input = SdlInput::new();
    let mut nes = Nes::new(game_file, display.clone(), input).expect("Could not start the game");

    let mut state = EmulatorState {
        running: false,
        exit: false,
        one_step: false,
        stop_on_program_counter: None,
    };

    display.actual_display();
    debug_display.update(&nes);
    debug_display.show(&nes);
    memory_display.update(&nes);
    memory_display.show(&nes);
    ppu_debug_display.display(&nes);

    let mut i = 0;

    loop {
        let start_time = std::time::Instant::now();

        for event in event_pump.poll_iter() {
            if event.get_window_id() == Some(display_window_id) {
                display.handle_event(event, &nes, &mut state);
            } else if event.get_window_id() == Some(memory_window_id) {
                memory_display.handle_event(event, &nes, &mut state);
            } else if event.get_window_id() == Some(debug_window_id) {
                debug_display.handle_event(event, &nes, &mut state);
            } else if event.get_window_id() == Some(ppu_debug_window_id) {
                ppu_debug_display.handle_event(event, &nes, &mut state);
            }
        }

        if state.exit {
            break;
        }

        if !state.running && state.one_step {
            nes.run_one_cpu_instruction();
            debug_display.update(&nes);
            memory_display.update(&nes);
            state.one_step = false;
        } else if state.running {
            for _ in 0..357954 {
                nes.run_one_cpu_tick();
                debug_display.update(&nes);
                memory_display.update(&nes);
                if debug_display
                    .breakpoints
                    .contains(&nes.cpu.borrow().program_counter)
                {
                    while !nes.cpu.borrow().finished_instruction() {
                        nes.run_one_cpu_tick();
                        debug_display.update(&nes);
                        memory_display.update(&nes);
                    }
                    state.running = false;
                    break;
                }
            }
        }

        display.actual_display();
        debug_display.show(&nes);
        memory_display.show(&nes);
        ppu_debug_display.display(&nes);

        // 60fps
        if !state.running {
            let nanos_to_sleep =
                Duration::from_nanos(1_000_000_000u64 / 60).saturating_sub(start_time.elapsed());
            if nanos_to_sleep != Duration::ZERO {
                std::thread::sleep(nanos_to_sleep);
            }
        }

        // i += 1;
        // if i > 1 {
        //     break;
        // }
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
