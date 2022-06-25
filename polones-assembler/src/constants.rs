use crate::types::AddressingMode;
use std::collections::HashMap;

pub const OPCODE_TO_ADDRESSING_MODE: [AddressingMode; 256] = {
    use AddressingMode::*;
    [
        Implied,          // 00: BRK impl
        XIndexedIndirect, // 01: ORA X,ind
        Implied,          // 02: NOP (illegal instruction)
        Implied,          // 03: NOP (illegal instruction)
        Implied,          // 04: NOP (illegal instruction)
        Zeropage,         // 05: ORA zpg
        Zeropage,         // 06: ASL zpg
        Implied,          // 07: NOP (illegal instruction)
        Implied,          // 08: PHP impl
        Immediate,        // 09: ORA #
        Accumulator,      // 0A: ASL A
        Implied,          // 0B: NOP (illegal instruction)
        Implied,          // 0C: NOP (illegal instruction)
        Absolute,         // 0D: ORA abs
        Absolute,         // 0E: ASL abs
        Implied,          // 0F: NOP (illegal instruction)
        Relative,         // 10: BPL rel
        IndirectYIndexed, // 11: ORA ind,Y
        Implied,          // 12: NOP (illegal instruction)
        Implied,          // 13: NOP (illegal instruction)
        Implied,          // 14: NOP (illegal instruction)
        ZeropageXIndexed, // 15: ORA zpg,X
        ZeropageXIndexed, // 16: ASL zpg,X
        Implied,          // 17: NOP (illegal instruction)
        Implied,          // 18: CLC impl
        AbsoluteYIndexed, // 19: ORA abs,Y
        Implied,          // 1A: NOP (illegal instruction)
        Implied,          // 1B: NOP (illegal instruction)
        Implied,          // 1C: NOP (illegal instruction)
        AbsoluteXIndexed, // 1D: ORA abs,X
        AbsoluteXIndexed, // 1E: ASL abs,X
        Implied,          // 1F: NOP (illegal instruction)
        Absolute,         // 20: JSR abs
        XIndexedIndirect, // 21: AND X,ind
        Implied,          // 22: NOP (illegal instruction)
        Implied,          // 23: NOP (illegal instruction)
        Zeropage,         // 24: BIT zpg
        Zeropage,         // 25: AND zpg
        Zeropage,         // 26: ROL zpg
        Implied,          // 27: NOP (illegal instruction)
        Implied,          // 28: PLP impl
        Immediate,        // 29: AND #
        Accumulator,      // 2A: ROL A
        Implied,          // 2B: NOP (illegal instruction)
        Absolute,         // 2C: BIT abs
        Absolute,         // 2D: AND abs
        Absolute,         // 2E: ROL abs
        Implied,          // 2F: NOP (illegal instruction)
        Relative,         // 30: BMI rel
        IndirectYIndexed, // 31: AND ind,Y
        Implied,          // 32: NOP (illegal instruction)
        Implied,          // 33: NOP (illegal instruction)
        Implied,          // 34: NOP (illegal instruction)
        ZeropageXIndexed, // 35: AND zpg,X
        ZeropageXIndexed, // 36: ROL zpg,X
        Implied,          // 37: NOP (illegal instruction)
        Implied,          // 38: SEC impl
        AbsoluteYIndexed, // 39: AND abs,Y
        Implied,          // 3A: NOP (illegal instruction)
        Implied,          // 3B: NOP (illegal instruction)
        Implied,          // 3C: NOP (illegal instruction)
        AbsoluteXIndexed, // 3D: AND abs,X
        AbsoluteXIndexed, // 3E: ROL abs,X
        Implied,          // 3F: NOP (illegal instruction)
        Implied,          // 40: RTI impl
        XIndexedIndirect, // 41: EOR X,ind
        Implied,          // 42: NOP (illegal instruction)
        Implied,          // 43: NOP (illegal instruction)
        Implied,          // 44: NOP (illegal instruction)
        Zeropage,         // 45: EOR zpg
        Zeropage,         // 46: LSR zpg
        Implied,          // 47: NOP (illegal instruction)
        Implied,          // 48: PHA impl
        Immediate,        // 49: EOR #
        Accumulator,      // 4A: LSR A
        Implied,          // 4B: NOP (illegal instruction)
        Absolute,         // 4C: JMP abs
        Absolute,         // 4D: EOR abs
        Absolute,         // 4E: LSR abs
        Implied,          // 4F: NOP (illegal instruction)
        Relative,         // 50: BVC rel
        IndirectYIndexed, // 51: EOR ind,Y
        Implied,          // 52: NOP (illegal instruction)
        Implied,          // 53: NOP (illegal instruction)
        Implied,          // 54: NOP (illegal instruction)
        ZeropageXIndexed, // 55: EOR zpg,X
        ZeropageXIndexed, // 56: LSR zpg,X
        Implied,          // 57: NOP (illegal instruction)
        Implied,          // 58: CLI impl
        AbsoluteYIndexed, // 59: EOR abs,Y
        Implied,          // 5A: NOP (illegal instruction)
        Implied,          // 5B: NOP (illegal instruction)
        Implied,          // 5C: NOP (illegal instruction)
        AbsoluteXIndexed, // 5D: EOR abs,X
        AbsoluteXIndexed, // 5E: LSR abs,X
        Implied,          // 5F: NOP (illegal instruction)
        Implied,          // 60: RTS impl
        XIndexedIndirect, // 61: ADC X,ind
        Implied,          // 62: NOP (illegal instruction)
        Implied,          // 63: NOP (illegal instruction)
        Implied,          // 64: NOP (illegal instruction)
        Zeropage,         // 65: ADC zpg
        Zeropage,         // 66: ROR zpg
        Implied,          // 67: NOP (illegal instruction)
        Implied,          // 68: PLA impl
        Immediate,        // 69: ADC #
        Accumulator,      // 6A: ROR A
        Implied,          // 6B: NOP (illegal instruction)
        Indirect,         // 6C: JMP ind
        Absolute,         // 6D: ADC abs
        Absolute,         // 6E: ROR abs
        Implied,          // 6F: NOP (illegal instruction)
        Relative,         // 70: BVS rel
        IndirectYIndexed, // 71: ADC ind,Y
        Implied,          // 72: NOP (illegal instruction)
        Implied,          // 73: NOP (illegal instruction)
        Implied,          // 74: NOP (illegal instruction)
        ZeropageXIndexed, // 75: ADC zpg,X
        ZeropageXIndexed, // 76: ROR zpg,X
        Implied,          // 77: NOP (illegal instruction)
        Implied,          // 78: SEI impl
        AbsoluteYIndexed, // 79: ADC abs,Y
        Implied,          // 7A: NOP (illegal instruction)
        Implied,          // 7B: NOP (illegal instruction)
        Implied,          // 7C: NOP (illegal instruction)
        AbsoluteXIndexed, // 7D: ADC abs,X
        AbsoluteXIndexed, // 7E: ROR abs,X
        Implied,          // 7F: NOP (illegal instruction)
        Implied,          // 80: NOP (illegal instruction)
        XIndexedIndirect, // 81: STA X,ind
        Implied,          // 82: NOP (illegal instruction)
        Implied,          // 83: NOP (illegal instruction)
        Zeropage,         // 84: STY zpg
        Zeropage,         // 85: STA zpg
        Zeropage,         // 86: STX zpg
        Implied,          // 87: NOP (illegal instruction)
        Implied,          // 88: DEY impl
        Implied,          // 89: NOP (illegal instruction)
        Implied,          // 8A: TXA impl
        Implied,          // 8B: NOP (illegal instruction)
        Absolute,         // 8C: STY abs
        Absolute,         // 8D: STA abs
        Absolute,         // 8E: STX abs
        Implied,          // 8F: NOP (illegal instruction)
        Relative,         // 90: BCC rel
        IndirectYIndexed, // 91: STA ind,Y
        Implied,          // 92: NOP (illegal instruction)
        Implied,          // 93: NOP (illegal instruction)
        ZeropageXIndexed, // 94: STY zpg,X
        ZeropageXIndexed, // 95: STA zpg,X
        ZeropageYIndexed, // 96: STX zpg,Y
        Implied,          // 97: NOP (illegal instruction)
        Implied,          // 98: TYA impl
        AbsoluteYIndexed, // 99: STA abs,Y
        Implied,          // 9A: TXS impl
        Implied,          // 9B: NOP (illegal instruction)
        Implied,          // 9C: NOP (illegal instruction)
        AbsoluteXIndexed, // 9D: STA abs,X
        Implied,          // 9E: NOP (illegal instruction)
        Implied,          // 9F: NOP (illegal instruction)
        Immediate,        // A0: LDY #
        XIndexedIndirect, // A1: LDA X,ind
        Immediate,        // A2: LDX #
        Implied,          // A3: NOP (illegal instruction)
        Zeropage,         // A4: LDY zpg
        Zeropage,         // A5: LDA zpg
        Zeropage,         // A6: LDX zpg
        Implied,          // A7: NOP (illegal instruction)
        Implied,          // A8: TAY impl
        Immediate,        // A9: LDA #
        Implied,          // AA: TAX impl
        Implied,          // AB: NOP (illegal instruction)
        Absolute,         // AC: LDY abs
        Absolute,         // AD: LDA abs
        Absolute,         // AE: LDX abs
        Implied,          // AF: NOP (illegal instruction)
        Relative,         // B0: BCS rel
        IndirectYIndexed, // B1: LDA ind,Y
        Implied,          // B2: NOP (illegal instruction)
        Implied,          // B3: NOP (illegal instruction)
        ZeropageXIndexed, // B4: LDY zpg,X
        ZeropageXIndexed, // B5: LDA zpg,X
        ZeropageYIndexed, // B6: LDX zpg,Y
        Implied,          // B7: NOP (illegal instruction)
        Implied,          // B8: CLV impl
        AbsoluteYIndexed, // B9: LDA abs,Y
        Implied,          // BA: TSX impl
        Implied,          // BB: NOP (illegal instruction)
        AbsoluteXIndexed, // BC: LDY abs,X
        AbsoluteXIndexed, // BD: LDA abs,X
        AbsoluteYIndexed, // BE: LDX abs,Y
        Implied,          // BF: NOP (illegal instruction)
        Immediate,        // C0: CPY #
        XIndexedIndirect, // C1: CMP X,ind
        Implied,          // C2: NOP (illegal instruction)
        Implied,          // C3: NOP (illegal instruction)
        Zeropage,         // C4: CPY zpg
        Zeropage,         // C5: CMP zpg
        Zeropage,         // C6: DEC zpg
        Implied,          // C7: NOP (illegal instruction)
        Implied,          // C8: INY impl
        Immediate,        // C9: CMP #
        Implied,          // CA: DEX impl
        Implied,          // CB: NOP (illegal instruction)
        Absolute,         // CC: CPY abs
        Absolute,         // CD: CMP abs
        Absolute,         // CE: DEC abs
        Implied,          // CF: NOP (illegal instruction)
        Relative,         // D0: BNE rel
        IndirectYIndexed, // D1: CMP ind,Y
        Implied,          // D2: NOP (illegal instruction)
        Implied,          // D3: NOP (illegal instruction)
        Implied,          // D4: NOP (illegal instruction)
        ZeropageXIndexed, // D5: CMP zpg,X
        ZeropageXIndexed, // D6: DEC zpg,X
        Implied,          // D7: NOP (illegal instruction)
        Implied,          // D8: CLD impl
        AbsoluteYIndexed, // D9: CMP abs,Y
        Implied,          // DA: NOP (illegal instruction)
        Implied,          // DB: NOP (illegal instruction)
        Implied,          // DC: NOP (illegal instruction)
        AbsoluteXIndexed, // DD: CMP abs,X
        AbsoluteXIndexed, // DE: DEC abs,X
        Implied,          // DF: NOP (illegal instruction)
        Immediate,        // E0: CPX #
        XIndexedIndirect, // E1: SBC X,ind
        Implied,          // E2: NOP (illegal instruction)
        Implied,          // E3: NOP (illegal instruction)
        Zeropage,         // E4: CPX zpg
        Zeropage,         // E5: SBC zpg
        Zeropage,         // E6: INC zpg
        Implied,          // E7: NOP (illegal instruction)
        Implied,          // E8: INX impl
        Immediate,        // E9: SBC #
        Implied,          // EA: NOP impl
        Implied,          // EB: NOP (illegal instruction)
        Absolute,         // EC: CPX abs
        Absolute,         // ED: SBC abs
        Absolute,         // EE: INC abs
        Implied,          // EF: NOP (illegal instruction)
        Relative,         // F0: BEQ rel
        IndirectYIndexed, // F1: SBC ind,Y
        Implied,          // F2: NOP (illegal instruction)
        Implied,          // F3: NOP (illegal instruction)
        Implied,          // F4: NOP (illegal instruction)
        ZeropageXIndexed, // F5: SBC zpg,X
        ZeropageXIndexed, // F6: INC zpg,X
        Implied,          // F7: NOP (illegal instruction)
        Implied,          // F8: SED impl
        AbsoluteYIndexed, // F9: SBC abs,Y
        Implied,          // FA: NOP (illegal instruction)
        Implied,          // FB: NOP (illegal instruction)
        Implied,          // FC: NOP (illegal instruction)
        AbsoluteXIndexed, // FD: SBC abs,X
        AbsoluteXIndexed, // FE: INC abs,X
        Implied,          // FF: NOP (illegal instruction)
    ]
};

lazy_static::lazy_static! {
pub static ref MNEMONIC_TO_OPCODES: HashMap<&'static str, &'static [u8]> = HashMap::from_iter(
    [
        ("ADC", &[0x61, 0x65, 0x69, 0x6D, 0x71, 0x75, 0x79, 0x7D][..]),
        ("AND", &[0x21, 0x25, 0x29, 0x2D, 0x31, 0x35, 0x39, 0x3D]),
        ("ASL", &[0x06, 0x0A, 0x0E, 0x16, 0x1E]),
        ("BCC", &[0x90]),
        ("BCS", &[0xB0]),
        ("BEQ", &[0xF0]),
        ("BIT", &[0x24, 0x2C]),
        ("BMI", &[0x30]),
        ("BNE", &[0xD0]),
        ("BPL", &[0x10]),
        ("BRK", &[0x00]),
        ("BVC", &[0x50]),
        ("BVS", &[0x70]),
        ("CLC", &[0x18]),
        ("CLD", &[0xD8]),
        ("CLI", &[0x58]),
        ("CLV", &[0xB8]),
        ("CMP", &[0xC1, 0xC5, 0xC9, 0xCD, 0xD1, 0xD5, 0xD9, 0xDD]),
        ("CPX", &[0xE0, 0xE4, 0xEC]),
        ("CPY", &[0xC0, 0xC4, 0xCC]),
        ("DEC", &[0xC6, 0xCE, 0xD6, 0xDE]),
        ("DEX", &[0xCA]),
        ("DEY", &[0x88]),
        ("EOR", &[0x41, 0x45, 0x49, 0x4D, 0x51, 0x55, 0x59, 0x5D]),
        ("INC", &[0xE6, 0xEE, 0xF6, 0xFE]),
        ("INX", &[0xE8]),
        ("INY", &[0xC8]),
        ("JMP", &[0x4C, 0x6C]),
        ("JSR", &[0x20]),
        ("LDA", &[0xA1, 0xA5, 0xA9, 0xAD, 0xB1, 0xB5, 0xB9, 0xBD]),
        ("LDX", &[0xA2, 0xA6, 0xAE, 0xB6, 0xBE]),
        ("LDY", &[0xA0, 0xA4, 0xAC, 0xB4, 0xBC]),
        ("LSR", &[0x46, 0x4A, 0x4E, 0x56, 0x5E]),
        ("NOP", &[0xEA]),
        ("ORA", &[0x01, 0x05, 0x09, 0x0D, 0x11, 0x15, 0x19, 0x1D]),
        ("PHA", &[0x48]),
        ("PHP", &[0x08]),
        ("PLA", &[0x68]),
        ("PLP", &[0x28]),
        ("ROL", &[0x26, 0x2A, 0x2E, 0x36, 0x3E]),
        ("ROR", &[0x66, 0x6A, 0x6E, 0x76, 0x7E]),
        ("RTI", &[0x40]),
        ("RTS", &[0x60]),
        ("SBC", &[0xE1, 0xE5, 0xE9, 0xED, 0xF1, 0xF5, 0xF9, 0xFD]),
        ("SEC", &[0x38]),
        ("SED", &[0xF8]),
        ("SEI", &[0x78]),
        ("STA", &[0x81, 0x85, 0x8D, 0x91, 0x95, 0x99, 0x9D]),
        ("STX", &[0x86, 0x8E, 0x96]),
        ("STY", &[0x84, 0x8C, 0x94]),
        ("TAX", &[0xAA]),
        ("TAY", &[0xA8]),
        ("TSX", &[0xBA]),
        ("TXA", &[0x8A]),
        ("TXS", &[0x9A]),
        ("TYA", &[0x98]),
    ]
    .into_iter(),
);
}
