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
use std::cell::RefCell;
use std::fmt::Write;
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

struct TextBuffer<const W: usize, const H: usize> {
    buffer: [[(u8, u8, u8, u8); W]; H],
}

impl<const W: usize, const H: usize> TextBuffer<W, H> {
    fn new() -> Self {
        Self {
            buffer: [[(0, 0, 0, 0); W]; H],
        }
    }

    fn clear(&mut self) {
        for y in 0..H {
            for x in 0..W {
                self.buffer[y][x] = (0, 0, 0, 0);
            }
        }
    }

    fn as_texture_data(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(&self.buffer[0][0].0, W * H * 4) }
    }

    fn write_char_with_color(&mut self, value: char, line: u8, col: u8, color: (u8, u8, u8)) {
        if (line as usize) < H / 8 && (col as usize) < W / 8 {
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
                    self.buffer[line * 8 + y][col * 8 + x] = (b, g, r, 0);
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
    text_buffer: TextBuffer<{ Self::WIDTH as usize }, { Self::HEIGHT as usize }>,
    breakpoint_mode: bool,
    breakpoint_address: u16,
    breakpoint_pos: i8,
    breakpoints: Vec<u16>,
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
            text_buffer: TextBuffer::new(),
            disassembly: [InstructionTableValue::Unknown; 1 << 16],
        }
    }

    fn write_instruction_from_disassembly(
        &mut self,
        address: u16,
        line: u8,
        col: u8,
        color: (u8, u8, u8),
    ) {
        let (operation, mode) = self.disassembly[address as usize].unwrap_opcode();
        let tb = &mut self.text_buffer;

        match (
            self.breakpoint_mode,
            self.breakpoint_address == address,
            self.breakpoints.contains(&address),
        ) {
            (true, true, true) => {
                tb.write_char_with_color('B', line, col, YELLOW);
            }
            (true, true, false) => {
                tb.write_char_with_color('*', line, col, YELLOW);
            }
            (_, _, true) => {
                tb.write_char_with_color('B', line, col, RED);
            }
            _ => {}
        }

        let operation_str = format!("{:?}", operation);
        tb.write_u16_with_color(address, line, col + 2, color);
        tb.write_str_with_color(&operation_str, line, col + 7, operation.color());

        // todo handle the case when next byte overflows address
        match mode {
            AddressingMode::Accumulator => {
                tb.write_char_with_color('A', line, col + 11, RED);
            }
            AddressingMode::Absolute => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                tb.write_char_with_color('$', line, col + 11, YELLOW);
                tb.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 12, WHITE);
            }
            AddressingMode::AbsoluteXIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                tb.write_char_with_color('$', line, col + 11, YELLOW);
                tb.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 12, WHITE);
                tb.write_str_with_color(",X", line, col + 16, RED);
            }
            AddressingMode::AbsoluteYIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                tb.write_char_with_color('$', line, col + 11, YELLOW);
                tb.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 12, WHITE);
                tb.write_str_with_color(",Y", line, col + 16, RED);
            }
            AddressingMode::Immediate => {
                let byte = self.disassembly[(address + 1) as usize].unwrap_value();
                tb.write_char_with_color('#', line, col + 11, RED);
                tb.write_char_with_color('$', line, col + 12, YELLOW);
                tb.write_u8_with_color(byte, line, col + 13, WHITE);
            }
            AddressingMode::Implied => {}
            AddressingMode::Indirect => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                let high = self.disassembly[(address + 2) as usize].unwrap_value();
                tb.write_char_with_color('(', line, col + 11, RED);
                tb.write_char_with_color('$', line, col + 12, YELLOW);
                tb.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 13, WHITE);
                tb.write_char_with_color(')', line, col + 17, RED);
            }
            AddressingMode::XIndexedIndirect => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                tb.write_char_with_color('(', line, col + 11, RED);
                tb.write_char_with_color('$', line, col + 12, YELLOW);
                tb.write_u8_with_color(low, line, col + 13, WHITE);
                tb.write_str_with_color(",X)", line, col + 15, RED);
            }
            AddressingMode::IndirectYIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                tb.write_char_with_color('(', line, col + 11, RED);
                tb.write_char_with_color('$', line, col + 12, YELLOW);
                tb.write_u8_with_color(low, line, col + 13, WHITE);
                tb.write_str_with_color("),Y", line, col + 15, RED);
            }
            AddressingMode::Relative => {
                // TODO has the same syntax as zeropage, maybe indicate which one is it?
                let byte = self.disassembly[(address + 1) as usize].unwrap_value();
                tb.write_char_with_color('$', line, col + 11, YELLOW);
                tb.write_u8_with_color(byte, line, col + 12, WHITE);
            }
            AddressingMode::Zeropage => {
                // TODO has the same syntax as relative, maybe indicate which one is it?
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                tb.write_char_with_color('$', line, col + 11, YELLOW);
                tb.write_u8_with_color(low, line, col + 12, WHITE);
            }
            AddressingMode::ZeropageXIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                tb.write_char_with_color('$', line, col + 11, YELLOW);
                tb.write_u8_with_color(low, line, col + 12, WHITE);
                tb.write_str_with_color(",X", line, col + 14, RED);
            }
            AddressingMode::ZeropageYIndexed => {
                let low = self.disassembly[(address + 1) as usize].unwrap_value();
                tb.write_char_with_color('$', line, col + 11, YELLOW);
                tb.write_u8_with_color(low, line, col + 12, WHITE);
                tb.write_str_with_color(",Y", line, col + 14, RED);
            }
        }
    }

    fn fill_instructions(&mut self, nes: &Nes) {
        let cpu = nes.cpu.borrow();
        let mut position = cpu.program_counter;
        let mut checked_count = 0;

        loop {
            // while let InstructionTableValue::Unknown = self.disassembly[position as usize] {
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
                                InstructionTableValue::Opcode(..) => {
                                    self.breakpoint_address = address;
                                    self.breakpoint_pos += 1;
                                    break;
                                }
                                InstructionTableValue::Value(..) => continue,
                                InstructionTableValue::Unknown => break,
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
                                InstructionTableValue::Opcode(..) => {
                                    self.breakpoint_address = address;
                                    self.breakpoint_pos -= 1;
                                    break;
                                }
                                InstructionTableValue::Value(..) => continue,
                                InstructionTableValue::Unknown => break,
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
                        InstructionTableValue::Opcode(..) => {
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
}

fn serialize_breakpoints(breakpoints: &[u16]) -> String {
    let mut output = String::new();
    for b in breakpoints.iter() {
        write!(&mut output, "{}\n", *b);
    }
    output
}

fn deserialize_breakpoints(string: String) -> Vec<u16> {
    string
        .split('\n')
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.parse::<u16>().ok())
        .collect()
}

impl DebugDisplay for SdlDebugDisplay {
    fn display(&mut self, nes: &Nes) {
        self.fill_instructions(nes);

        self.canvas.clear();
        self.text_buffer.clear();
        let cpu = nes.cpu.borrow();
        let tb = &mut self.text_buffer;

        tb.write_str_with_color("A", 0, 1, YELLOW);
        tb.write_u8_with_color(cpu.accumulator, 0, 3, WHITE);

        tb.write_str_with_color("X", 1, 1, YELLOW);
        tb.write_u8_with_color(cpu.x_index, 1, 3, WHITE);

        tb.write_str_with_color("Y", 2, 1, YELLOW);
        tb.write_u8_with_color(cpu.y_index, 2, 3, WHITE);

        tb.write_str_with_color("SP", 3, 0, YELLOW);
        tb.write_u8_with_color(cpu.stack_pointer, 3, 3, WHITE);

        tb.write_str_with_color("PC", 4, 0, YELLOW);
        tb.write_u16_with_color(cpu.program_counter, 4, 3, WHITE);

        tb.write_str_with_color("SR", 5, 0, YELLOW);

        tb.write_str_with_color("N", 6, 1, YELLOW);
        tb.write_bool_with_color(cpu.status_register.get_negative(), 6, 3, WHITE);

        tb.write_str_with_color("V", 7, 1, YELLOW);
        tb.write_bool_with_color(cpu.status_register.get_overflow(), 7, 3, WHITE);

        tb.write_str_with_color("-", 8, 1, YELLOW);
        tb.write_bool_with_color(cpu.status_register.get_ignored(), 8, 3, WHITE);

        tb.write_str_with_color("B", 9, 1, YELLOW);
        tb.write_bool_with_color(cpu.status_register.get_break(), 9, 3, WHITE);

        tb.write_str_with_color("D", 10, 1, YELLOW);
        tb.write_bool_with_color(cpu.status_register.get_decimal(), 10, 3, WHITE);

        tb.write_str_with_color("I", 11, 1, YELLOW);
        tb.write_bool_with_color(cpu.status_register.get_interrupt(), 11, 3, WHITE);

        tb.write_str_with_color("Z", 12, 1, YELLOW);
        tb.write_bool_with_color(cpu.status_register.get_zero(), 12, 3, WHITE);

        tb.write_str_with_color("C", 13, 1, YELLOW);
        tb.write_bool_with_color(cpu.status_register.get_carry(), 13, 3, WHITE);

        if self.breakpoint_mode {
            tb.write_char_with_color('B', 29, 0, RED);
        }

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
                self.write_instruction_from_disassembly(address, line, 11, effective_color);
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
                    self.write_instruction_from_disassembly(address, line, 11, WHITE);
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
                &self.text_buffer.as_texture_data(),
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
    text_buffer: TextBuffer<{ Self::WIDTH as usize }, { Self::HEIGHT as usize }>,
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
            text_buffer: TextBuffer::new(),
        }
    }

    fn display(&mut self, nes: &Nes) {
        self.canvas.clear();
        self.text_buffer.clear();
        let tb = &mut self.text_buffer;
        let ppu = nes.ppu.borrow();

        tb.write_str_with_color("SCANLINE", 0, 0, YELLOW);
        tb.write_u16_with_color(ppu.scanline, 0, 9, WHITE);

        tb.write_str_with_color("PIXEL", 1, 3, YELLOW);
        tb.write_u16_with_color(ppu.pixel, 1, 9, WHITE);

        tb.write_str_with_color("CTRL", 2, 2, YELLOW);

        tb.write_str_with_color("NMI", 3, 3, YELLOW);
        tb.write_bool_with_color(ppu.control_register.get_nmi_enable(), 3, 7, WHITE);

        tb.write_str_with_color("M/S", 4, 3, YELLOW);
        tb.write_bool_with_color(ppu.control_register.get_ppu_master_slave(), 4, 7, WHITE);

        tb.write_str_with_color("HEIGHT", 5, 0, YELLOW);
        tb.write_bool_with_color(ppu.control_register.get_sprite_height(), 5, 7, WHITE);

        tb.write_str_with_color("BACK", 6, 2, YELLOW);
        tb.write_bool_with_color(
            ppu.control_register.get_background_tile_select(),
            6,
            7,
            WHITE,
        );

        tb.write_str_with_color("SPRITE", 7, 0, YELLOW);
        tb.write_bool_with_color(ppu.control_register.get_sprite_tile_select(), 7, 7, WHITE);

        tb.write_str_with_color("INC", 8, 3, YELLOW);
        tb.write_bool_with_color(ppu.control_register.get_sprite_height(), 8, 7, WHITE);

        tb.write_str_with_color("NTADDR", 9, 0, YELLOW);
        tb.write_u8_with_color(ppu.control_register.get_name_table_address(), 9, 7, WHITE);

        self.texture
            .update(
                Rect::new(0, 0, Self::WIDTH, Self::HEIGHT),
                &self.text_buffer.as_texture_data(),
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
        .position_centered()
        .build()
        .unwrap();
    let display_window_id = display_window.id();
    let display_canvas = display_window.into_canvas().build().unwrap();

    let debug_window = video_subsystem
        .window(
            "nes cpu inspector",
            SdlDisplay::WIDTH * 3,
            SdlDisplay::HEIGHT * 3,
        )
        .position_centered()
        .build()
        .unwrap();
    let debug_window_id = debug_window.id();
    let debug_canvas = debug_window.into_canvas().build().unwrap();

    let ppu_debug_window = video_subsystem
        .window(
            "nes ppu inspector",
            SdlDisplay::WIDTH * 3,
            SdlDisplay::HEIGHT * 3,
        )
        .position_centered()
        .build()
        .unwrap();
    let ppu_debug_window_id = ppu_debug_window.id();
    let ppu_debug_canvas = ppu_debug_window.into_canvas().build().unwrap();

    let mut display = SdlDisplay::new(display_canvas);
    let mut debug_display = SdlDebugDisplay::new(debug_canvas);
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
    debug_display.display(&nes);
    ppu_debug_display.display(&nes);

    loop {
        let start_time = std::time::Instant::now();

        for event in event_pump.poll_iter() {
            if event.get_window_id() == Some(display_window_id) {
                display.handle_event(event, &nes, &mut state);
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
            state.one_step = false;
        } else if state.running {
            nes.run_one_cpu_instruction();
            if debug_display.breakpoints.contains(&nes.cpu.borrow().program_counter) {
                state.running = false;
            }
        }

        display.actual_display();
        debug_display.display(&nes);
        ppu_debug_display.display(&nes);

        // 60fps
        if !state.running {
            let nanos_to_sleep =
                Duration::from_nanos(1_000_000_000u64 / 60).saturating_sub(start_time.elapsed());
            if nanos_to_sleep != Duration::ZERO {
                std::thread::sleep(nanos_to_sleep);
            }
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
