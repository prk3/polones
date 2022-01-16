use crate::text_area::{Color, Color::*, TextArea};
use crate::EmulatorState;
use polones_core::nes::Nes;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;
use std::borrow::Borrow;
use std::rc::Rc;

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

pub struct SdlCpuDebugger {
    canvas: sdl2::render::WindowCanvas,
    _texture_creator: Rc<sdl2::render::TextureCreator<WindowContext>>,
    texture: sdl2::render::Texture<'static>,
    text_area: TextArea<{ Self::WIDTH as usize / 8 }, { Self::HEIGHT as usize / 8 }>,
    breakpoint_mode: bool,
    breakpoint_address: u16,
    breakpoint_pos: i8,
    pub breakpoints: Vec<u16>,
    nmi_breakpoint: u16,
    disassembly: [DisassemblyValue; 1 << 16],
}

impl SdlCpuDebugger {
    pub const WIDTH: u32 = 256;
    pub const HEIGHT: u32 = 240;

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

    pub fn new(canvas: sdl2::render::WindowCanvas) -> Self {
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
            nmi_breakpoint: 0,
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

    pub fn handle_event(&mut self, event: Event, nes: &Nes, state: &mut EmulatorState) {
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
                    if self.breakpoint_pos > -14 {
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
                    keycode: _k @ Some(Keycode::N),
                    ..
                } => {
                    if self.breakpoints.contains(&self.nmi_breakpoint) {
                        self.breakpoints.retain(|b| *b != self.nmi_breakpoint);
                    } else {
                        self.breakpoints.push(self.nmi_breakpoint);
                    }
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

    pub fn update(&mut self, nes: &Nes) {
        self.fill_instructions(nes);
    }

    pub fn show(&mut self, nes: &Nes) {
        self.canvas.clear();
        self.text_area.clear();
        let cpu = nes.cpu.borrow();
        let ta = &mut self.text_area;

        {
            let low = nes.borrow().cpu_bus_read(0xFFFA);
            let high = nes.borrow().cpu_bus_read(0xFFFB);
            self.nmi_breakpoint = ((high as u16) << 8) | low as u16;
        }

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
            ta.write_char_with_color('B', 28, 0, Red);
        }

        if self.breakpoints.contains(&self.nmi_breakpoint) {
            ta.write_str_with_color("NMI", 29, 0, Red);
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
    use std::fmt::Write;

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

#[test]
fn disassebly_table_has_unique_elements() {
    let mut set = std::collections::BTreeSet::<(Operation, AddressingMode)>::new();

    for entry in SdlCpuDebugger::DISASSEMBLY_TABLE.iter() {
        if set.contains(entry) && entry != &(Operation::XXX, AddressingMode::Implied) {
            panic!("{:?} {:?} repeats", entry.0, entry.1);
        }
        set.insert(*entry);
    }
}
