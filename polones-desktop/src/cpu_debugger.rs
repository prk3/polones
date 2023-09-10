use crate::text_area::{Color, Color::*, TextArea};
use crate::EmulatorState;
use polones_core::nes::{CpuBus, Nes};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::WindowContext;
use std::rc::Rc;

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
    /// Whether debugger is in breakpoint mode (editing breakpoints).
    breakpoint_mode: bool,
    /// Where in CPU bus addressing space the breakpoint cursor is atm.
    breakpoint_address: u16,
    /// Offset of the breakpoint cursor from the program counter (in lines).
    breakpoint_pos: i8,
    pub breakpoints: Vec<u16>,
    disassembly: [DisassemblyValue; 1 << 16],
}

impl SdlCpuDebugger {
    pub const WIDTH: u32 = 256;
    pub const HEIGHT: u32 = 240;

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
            .map(deserialize_breakpoints)
            .unwrap_or_default();

        Self {
            canvas,
            texture: unsafe { std::mem::transmute(texture) },
            _texture_creator: texture_creator,
            text_area: TextArea::new(),
            breakpoint_mode: false,
            breakpoint_address: 0,
            breakpoint_pos: 0,
            breakpoints,
            disassembly: [DisassemblyValue::Unknown; 1 << 16],
        }
    }

    pub fn handle_event(&mut self, nes: &mut Nes, event: Event, state: &mut EmulatorState) {
        let (cpu, mut cpu_bus) = nes.split_into_cpu_and_bus();
        if state.running {
            match event {
                Event::Window { win_event: WindowEvent::Close, .. } => {
                    state.exit = true;
                }
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
                    keycode: _k @ Some(Keycode::Space),
                    ..
                } => {
                    state.running = false;
                }
                _ => {}
            }
        } else if !self.breakpoint_mode {
            match event {
                Event::Window { win_event: WindowEvent::Close, .. } => {
                    state.exit = true;
                }
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
                    keycode: _k @ Some(Keycode::Space),
                    ..
                } => {
                    state.running = true;
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::N),
                    ..
                } => {
                    let nmi = nmi_address(&mut cpu_bus);

                    if self.breakpoints.contains(&nmi) {
                        self.breakpoints.retain(|b| *b != nmi);
                    } else {
                        self.breakpoints.push(nmi);
                    }
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::I),
                    ..
                } => {
                    let irq = irq_address(&mut cpu_bus);

                    if self.breakpoints.contains(&irq) {
                        self.breakpoints.retain(|b| *b != irq);
                    } else {
                        self.breakpoints.push(irq);
                    }
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::B),
                    ..
                } => match self.disassembly[cpu.program_counter as usize] {
                    DisassemblyValue::Opcode(..) => {
                        self.breakpoint_address = cpu.program_counter;
                        self.breakpoint_pos = 0;
                        self.breakpoint_mode = true;
                    }
                    _ => {
                        eprintln!(
                            "Could not set a breakpoint: PC is not pointing at an instruction"
                        )
                    }
                },
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::Return),
                    ..
                } => {
                    state.one_step = true;
                }
                _ => {}
            }
        } else {
            match event {
                Event::Window { win_event: WindowEvent::Close, .. } => {
                    state.exit = true;
                }
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
                    keycode: _k @ Some(Keycode::N),
                    ..
                } => {
                    let nmi = nmi_address(&mut cpu_bus);

                    if self.breakpoints.contains(&nmi) {
                        self.breakpoints.retain(|b| *b != nmi);
                    } else {
                        self.breakpoints.push(nmi);
                    }
                }
                Event::KeyDown {
                    keycode: _k @ Some(Keycode::B),
                    ..
                } => {
                    if self.breakpoints.contains(&self.breakpoint_address) {
                        self.breakpoints.retain(|b| *b != self.breakpoint_address);
                    } else {
                        self.breakpoints.push(self.breakpoint_address);
                    }
                    self.breakpoint_mode = false;
                    // TODO add game name to the breakpoints file name
                    let _ = std::fs::write(
                        "./breakpoints".to_string(),
                        serialize_breakpoints(&self.breakpoints),
                    );
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
        }
    }

    pub fn draw(&mut self, nes: &mut Nes) {
        self.canvas.clear();
        self.text_area.clear();
        let ta = &mut self.text_area;
        let (cpu, mut cpu_bus) = nes.split_into_cpu_and_bus();

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
            ta.write_char_with_color('B', 27, 0, Red);
        }
        if self.breakpoints.contains(&nmi_address(&mut cpu_bus)) {
            ta.write_str_with_color("NMI", 28, 0, Red);
        }
        if self.breakpoints.contains(&irq_address(&mut cpu_bus)) {
            ta.write_str_with_color("IRQ", 29, 0, Red);
        }

        let mut color = Red;
        let mut line = 14;

        for address in (0..=cpu.program_counter).rev() {
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
                DisassemblyValue::Unknown => break,
            }
        }

        if cpu.program_counter < u16::MAX {
            let mut line = 15;

            for address in (cpu.program_counter + 1)..=u16::MAX {
                match self.disassembly[address as usize] {
                    DisassemblyValue::Opcode(..) => {
                        self.write_instruction_from_disassembly(address, line, 11, White);
                        if line == 30 {
                            break;
                        }
                        line += 1;
                    }
                    DisassemblyValue::Value(..) => {}
                    DisassemblyValue::Unknown => break,
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

    pub fn update_disassembly(&mut self, nes: &mut Nes) {
        let (cpu, mut cpu_bus) = nes.split_into_cpu_and_bus();
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
                    let opcode = cpu_bus.read(position);
                    let (operation, mode) = DISASSEMBLY_TABLE[opcode as usize];

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
                                DisassemblyValue::Value(cpu_bus.read(position + 1));
                            position += 2;
                        }
                        Absolute | AbsoluteXIndexed | AbsoluteYIndexed | Indirect => {
                            self.disassembly[(position + 1) as usize] =
                                DisassemblyValue::Value(cpu_bus.read(position + 1));
                            self.disassembly[(position + 2) as usize] =
                                DisassemblyValue::Value(cpu_bus.read(position + 2));
                            position += 3;
                        }
                    }
                }
            }
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

        let ta = &mut self.text_area;
        let (operation, mode) = self.disassembly[address as usize].unwrap_opcode();

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

        match mode {
            AddressingMode::Accumulator => {
                ta.write_char_with_color('A', line, col + 11, Red);
            }
            AddressingMode::Absolute => {
                if address <= u16::MAX - 2 {
                    let low = self.disassembly[(address + 1) as usize].unwrap_value();
                    let high = self.disassembly[(address + 2) as usize].unwrap_value();
                    ta.write_char_with_color('$', line, col + 11, Yellow);
                    ta.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 12, White);
                }
            }
            AddressingMode::AbsoluteXIndexed => {
                if address <= u16::MAX - 2 {
                    let low = self.disassembly[(address + 1) as usize].unwrap_value();
                    let high = self.disassembly[(address + 2) as usize].unwrap_value();
                    ta.write_char_with_color('$', line, col + 11, Yellow);
                    ta.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 12, White);
                    ta.write_str_with_color(",X", line, col + 16, Red);
                }
            }
            AddressingMode::AbsoluteYIndexed => {
                if address <= u16::MAX - 2 {
                    let low = self.disassembly[(address + 1) as usize].unwrap_value();
                    let high = self.disassembly[(address + 2) as usize].unwrap_value();
                    ta.write_char_with_color('$', line, col + 11, Yellow);
                    ta.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 12, White);
                    ta.write_str_with_color(",Y", line, col + 16, Red);
                }
            }
            AddressingMode::Immediate => {
                if address <= u16::MAX - 1 {
                    let byte = self.disassembly[(address + 1) as usize].unwrap_value();
                    ta.write_char_with_color('#', line, col + 11, Red);
                    ta.write_char_with_color('$', line, col + 12, Yellow);
                    ta.write_u8_with_color(byte, line, col + 13, White);
                }
            }
            AddressingMode::Implied => {}
            AddressingMode::Indirect => {
                if address <= u16::MAX - 2 {
                    let low = self.disassembly[(address + 1) as usize].unwrap_value();
                    let high = self.disassembly[(address + 2) as usize].unwrap_value();
                    ta.write_char_with_color('(', line, col + 11, Red);
                    ta.write_char_with_color('$', line, col + 12, Yellow);
                    ta.write_u16_with_color((high as u16) << 8 | low as u16, line, col + 13, White);
                    ta.write_char_with_color(')', line, col + 17, Red);
                }
            }
            AddressingMode::XIndexedIndirect => {
                if address <= u16::MAX - 1 {
                    let low = self.disassembly[(address + 1) as usize].unwrap_value();
                    ta.write_char_with_color('(', line, col + 11, Red);
                    ta.write_char_with_color('$', line, col + 12, Yellow);
                    ta.write_u8_with_color(low, line, col + 13, White);
                    ta.write_str_with_color(",X)", line, col + 15, Red);
                }
            }
            AddressingMode::IndirectYIndexed => {
                if address <= u16::MAX - 1 {
                    let low = self.disassembly[(address + 1) as usize].unwrap_value();
                    ta.write_char_with_color('(', line, col + 11, Red);
                    ta.write_char_with_color('$', line, col + 12, Yellow);
                    ta.write_u8_with_color(low, line, col + 13, White);
                    ta.write_str_with_color("),Y", line, col + 15, Red);
                }
            }
            AddressingMode::Relative => {
                // TODO has the same syntax as zeropage, maybe indicate which one is it?
                if address <= u16::MAX - 1 {
                    let byte = self.disassembly[(address + 1) as usize].unwrap_value();
                    ta.write_char_with_color('$', line, col + 11, Yellow);
                    ta.write_u8_with_color(byte, line, col + 12, White);
                }
            }
            AddressingMode::Zeropage => {
                // TODO has the same syntax as relative, maybe indicate which one is it?
                if address <= u16::MAX - 1 {
                    let low = self.disassembly[(address + 1) as usize].unwrap_value();
                    ta.write_char_with_color('$', line, col + 11, Yellow);
                    ta.write_u8_with_color(low, line, col + 12, White);
                }
            }
            AddressingMode::ZeropageXIndexed => {
                if address <= u16::MAX - 1 {
                    let low = self.disassembly[(address + 1) as usize].unwrap_value();
                    ta.write_char_with_color('$', line, col + 11, Yellow);
                    ta.write_u8_with_color(low, line, col + 12, White);
                    ta.write_str_with_color(",X", line, col + 14, Red);
                }
            }
            AddressingMode::ZeropageYIndexed => {
                if address <= u16::MAX - 1 {
                    let low = self.disassembly[(address + 1) as usize].unwrap_value();
                    ta.write_char_with_color('$', line, col + 11, Yellow);
                    ta.write_u8_with_color(low, line, col + 12, White);
                    ta.write_str_with_color(",Y", line, col + 14, Red);
                }
            }
        }
    }
}

fn serialize_breakpoints(breakpoints: &[u16]) -> String {
    use std::fmt::Write;

    let mut output = String::new();
    for b in breakpoints.iter() {
        writeln!(&mut output, "{:04X}", *b).unwrap();
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

fn nmi_address(cpu_bus: &mut CpuBus<'_>) -> u16 {
    let low = cpu_bus.read(0xFFFA);
    let high = cpu_bus.read(0xFFFB);
    let nmi = ((high as u16) << 8) | low as u16;
    nmi
}

fn irq_address(cpu_bus: &mut CpuBus<'_>) -> u16 {
    let low = cpu_bus.read(0xFFFE);
    let high = cpu_bus.read(0xFFFF);
    let irq = ((high as u16) << 8) | low as u16;
    irq
}

#[test]
fn disassembly_table_has_unique_elements() {
    let mut set = std::collections::BTreeSet::<(Operation, AddressingMode)>::new();

    for entry in DISASSEMBLY_TABLE
        .iter()
        .filter(|entry| **entry != (Operation::XXX, AddressingMode::Implied))
    {
        assert!(!set.contains(entry), "{:?} {:?} repeats", entry.0, entry.1);
        set.insert(*entry);
    }
}
