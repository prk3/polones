const FONT: &[u8; 16 * 32] = include_bytes!("../resources/comodore-64-font.bin");

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black = 0b000,
    Red = 0b100,
    Green = 0b010,
    Blue = 0b001,
    Yellow = 0b110,
    Cyan = 0b011,
    Magenta = 0b101,
    White = 0b111,
}

pub struct TextArea<const W: usize, const H: usize> {
    buffer: [[(u8, Color); W]; H],
}

impl<const W: usize, const H: usize> TextArea<W, H> {
    pub fn new() -> Self {
        Self {
            buffer: [[(0b100000, Color::Black); W]; H],
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn draw_to_texture(&self, bytes: &mut [u8]) {
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

    pub fn write_char_with_color(&mut self, value: char, line: u8, col: u8, color: Color) {
        if (line as usize) >= H || (col as usize) >= W {
            return;
        }

        let (char_row, char_col) = match value {
            _c @ 'A'..='O' => (0, 1 + (value as u8 - b'A')),
            _c @ 'a'..='o' => (0, 1 + (value as u8 - b'a')),
            _c @ 'P'..='Z' => (1, value as u8 - b'P'),
            _c @ 'p'..='z' => (1, value as u8 - b'p'),
            _c @ '0'..='9' => (3, value as u8 - b'0'),
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

    pub fn write_str_with_color(&mut self, value: &str, line: u8, col: u8, color: Color) {
        let mut col = col;
        for c in value.chars() {
            self.write_char_with_color(c, line, col, color);
            col = col.saturating_add(1);
        }
    }

    pub fn write_u8_with_color(&mut self, value: u8, line: u8, col: u8, color: Color) {
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

    pub fn write_u16_with_color(&mut self, value: u16, line: u8, col: u8, color: Color) {
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

    pub fn write_bool_with_color(&mut self, value: bool, line: u8, col: u8, color: Color) {
        self.write_char_with_color(
            char::from_u32('0' as u32 + value as u32).unwrap(),
            line,
            col,
            color,
        );
    }
}
