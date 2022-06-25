use crate::constants::*;
use crate::types::*;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::ErrorKind;
use nom::multi::separated_list1;
use nom::sequence::*;
use std::convert::Infallible;

pub fn item(input: &str) -> nom::IResult<&str, Option<Item>> {
    let (rest, _) = space0(input)?;

    if rest.starts_with(";!") {
        let rest = &rest[2..];
        let rest = strip_comment(rest);
        let (rest, _) = space0(rest)?;

        return map(
            alt((
                map(
                    tuple((
                        tag("put"),
                        space1,
                        tag("address"),
                        space1,
                        tag("of"),
                        space1,
                        label,
                        space1,
                        tag("at"),
                        space1,
                        tag("prg"),
                        space1,
                        long_address_hex,
                        space0,
                        eof,
                    )),
                    |tuple| Directive::PutAddressOfSubroutineAtPrgAddress(tuple.6, tuple.12),
                ),
                map(
                    tuple((
                        tag("put"),
                        space1,
                        tag("address"),
                        space1,
                        address_hex,
                        space1,
                        tag("at"),
                        space1,
                        tag("prg"),
                        space1,
                        long_address_hex,
                        space0,
                        eof,
                    )),
                    |tuple| Directive::PutAddressAtPrgAddress(tuple.4, tuple.10),
                ),
                map(
                    separated_list1(space1, take_while1(|c: char| !c.is_whitespace())),
                    |list: Vec<&str>| {
                        Directive::Other(list.into_iter().map(String::from).collect())
                    },
                ),
            )),
            |directive| Some(Item::Directive(directive)),
        )(rest);
    }

    let rest = strip_comment(rest);
    let (rest, _) = space0(rest)?;

    if rest.is_empty() {
        return Ok((rest, None));
    }

    alt((
        map(instruction, |i| Some(Item::Instruction(i))),
        map(label_line, |l| Some(Item::Label(l))),
    ))(rest)
}

fn strip_comment(line: &str) -> &str {
    line.splitn(2, ';').next().unwrap()
}

fn mnemonic(input: &str) -> nom::IResult<&str, &str> {
    map_res(alpha1, |mnemonic| {
        if MNEMONIC_TO_OPCODES.contains_key(mnemonic) {
            Ok(mnemonic)
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify,
            )))
        }
    })(input)
}

fn instruction(input: &str) -> nom::IResult<&str, Instruction> {
    let (rest, mnemonic) = preceded(space0, mnemonic)(input)?;

    for opcode in MNEMONIC_TO_OPCODES[mnemonic] {
        match OPCODE_TO_ADDRESSING_MODE[*opcode as usize] {
            AddressingMode::Accumulator => {
                if let Ok((rest, _)) =
                    tuple::<_, _, (&str, ErrorKind), _>((space1, char('A'), space0, eof))(rest)
                {
                    return Ok((rest, Instruction::Opcode(*opcode)));
                }
            }
            AddressingMode::Absolute => {
                if let Ok((rest, destination)) =
                    delimited(space1, either_address_hex_or_label, tuple((space0, eof)))(rest)
                {
                    return Ok((
                        rest,
                        match destination {
                            Either::Left((lo, hi)) => {
                                Instruction::OpcodeAndTwoBytes(*opcode, lo, hi)
                            }
                            Either::Right(label) => Instruction::OpcodeAndLabel(*opcode, label),
                        },
                    ));
                }
            }
            AddressingMode::AbsoluteXIndexed => {
                if let Ok((rest, (lo, hi))) =
                    delimited(space1, address_hex, tuple((tag(",X"), space0, eof)))(rest)
                {
                    return Ok((rest, Instruction::OpcodeAndTwoBytes(*opcode, lo, hi)));
                }
            }
            AddressingMode::AbsoluteYIndexed => {
                if let Ok((rest, (lo, hi))) =
                    delimited(space1, address_hex, tuple((tag(",Y"), space0, eof)))(rest)
                {
                    return Ok((rest, Instruction::OpcodeAndTwoBytes(*opcode, lo, hi)));
                }
            }
            AddressingMode::Immediate => {
                if let Ok((rest, byte)) = delimited(
                    tuple((space1, char('#'))),
                    alt((byte_dec, byte_hex, byte_bin)),
                    tuple((space0, eof)),
                )(rest)
                {
                    return Ok((rest, Instruction::OpcodeAndByte(*opcode, byte)));
                }
            }
            AddressingMode::Implied => {
                if let Ok((rest, _)) = tuple::<_, _, (&str, ErrorKind), _>((space0, eof))(rest) {
                    return Ok((rest, Instruction::Opcode(*opcode)));
                }
            }
            AddressingMode::Indirect => {
                if let Ok((rest, (lo, hi))) = delimited(
                    tuple((space1, char('('))),
                    address_hex,
                    tuple((char(')'), space0, eof)),
                )(rest)
                {
                    return Ok((rest, Instruction::OpcodeAndTwoBytes(*opcode, lo, hi)));
                }
            }
            AddressingMode::XIndexedIndirect => {
                if let Ok((rest, byte)) = delimited(
                    tuple((space1, char('('))),
                    byte_hex,
                    tuple((tag(",X)"), space0, eof)),
                )(rest)
                {
                    return Ok((rest, Instruction::OpcodeAndByte(*opcode, byte)));
                }
            }
            AddressingMode::IndirectYIndexed => {
                if let Ok((rest, byte)) = delimited(
                    tuple((space1, char('('))),
                    byte_hex,
                    tuple((tag("),Y"), space0, eof)),
                )(rest)
                {
                    return Ok((rest, Instruction::OpcodeAndByte(*opcode, byte)));
                }
            }
            AddressingMode::Relative => {
                if let Ok((rest, destination)) =
                    delimited(space1, either_byte_hex_or_label_local, tuple((space0, eof)))(rest)
                {
                    return Ok((
                        rest,
                        match destination {
                            Either::Left(byte) => Instruction::OpcodeAndByte(*opcode, byte),
                            Either::Right(label) => {
                                Instruction::OpcodeAndLabelLocal(*opcode, label)
                            }
                        },
                    ));
                }
            }
            AddressingMode::Zeropage => {
                if let Ok((rest, byte)) = delimited(space1, byte_hex, tuple((space0, eof)))(rest) {
                    return Ok((rest, Instruction::OpcodeAndByte(*opcode, byte)));
                }
            }
            AddressingMode::ZeropageXIndexed => {
                if let Ok((rest, byte)) =
                    delimited(space1, byte_hex, tuple((tag(",X"), space0, eof)))(rest)
                {
                    return Ok((rest, Instruction::OpcodeAndByte(*opcode, byte)));
                }
            }
            AddressingMode::ZeropageYIndexed => {
                if let Ok((rest, byte)) =
                    delimited(space1, byte_hex, tuple((tag(",Y"), space0, eof)))(rest)
                {
                    return Ok((rest, Instruction::OpcodeAndByte(*opcode, byte)));
                }
            }
        }
    }

    fail(rest)
}

fn byte_dec(input: &str) -> nom::IResult<&str, u8> {
    map_res(take_while1(|c: char| c.is_digit(10)), |input| {
        u8::from_str_radix(input, 10)
    })(input)
}

fn byte_hex(input: &str) -> nom::IResult<&str, u8> {
    preceded(
        char('$'),
        map_res(take_while_m_n(2, 2, |c: char| c.is_digit(16)), |input| {
            u8::from_str_radix(input, 16)
        }),
    )(input)
}

fn byte_bin(input: &str) -> nom::IResult<&str, u8> {
    preceded(
        char('%'),
        map_res(take_while1(|c: char| c.is_digit(2)), |input| {
            u8::from_str_radix(input, 2)
        }),
    )(input)
}

fn address_hex(input: &str) -> nom::IResult<&str, (u8, u8)> {
    preceded(
        char('$'),
        map_res(
            take_while_m_n(4, 4, |c: char| c.is_digit(16)),
            address_from_hex,
        ),
    )(input)
}

fn address_from_hex(input: &str) -> Result<(u8, u8), std::num::ParseIntError> {
    let value = u16::from_str_radix(input, 16)?;
    Ok(((value >> 8) as u8, value as u8))
}

fn label(input: &str) -> nom::IResult<&str, String> {
    map_res(
        tuple((
            take_while1(|c: char| c.is_ascii() && c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_ascii() && c.is_alphanumeric() || c == '_'),
        )),
        |(start, end): (&str, &str)| Result::<_, Infallible>::Ok(format!("{start}{end}")),
    )(input)
}

fn label_local(input: &str) -> nom::IResult<&str, String> {
    map_res(
        preceded(
            char('@'),
            take_while1(|c: char| c.is_ascii() && c.is_alphanumeric() || c == '_'),
        ),
        |label: &str| Result::<_, Infallible>::Ok(label.to_string()),
    )(input)
}

fn label_line(input: &str) -> nom::IResult<&str, String> {
    delimited(space0, label, tuple((space0, eof)))(input)
}

fn label_local_line(input: &str) -> nom::IResult<&str, String> {
    delimited(space0, label_local, tuple((space0, eof)))(input)
}

fn long_address_hex(input: &str) -> nom::IResult<&str, usize> {
    preceded(
        char('$'),
        map_res(hex_digit1, |hex| usize::from_str_radix(hex, 16)),
    )(input)
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

fn either_address_hex_or_label(input: &str) -> nom::IResult<&str, Either<(u8, u8), String>> {
    alt((
        map(address_hex, |address| Either::Left(address)),
        map(label, |label| Either::Right(label)),
    ))(input)
}

fn either_byte_hex_or_label_local(input: &str) -> nom::IResult<&str, Either<u8, String>> {
    alt((
        map(byte_hex, |byte| Either::Left(byte)),
        map(label_local, |label| Either::Right(label)),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Directive, Instruction, Item};

    #[test]
    fn item_parsing() {
        fn item_value(input: &str) -> Result<Option<Item>, String> {
            item(input)
                .map(|(_, output)| output)
                .map_err(|error| error.to_string())
        }

        // empty
        assert_eq!(Ok(None), item_value(""));
        assert_eq!(Ok(None), item_value("    "));
        assert_eq!(Ok(None), item_value("; comment"));

        // directives
        assert_eq!(
            Ok(Some(Item::Directive(
                Directive::PutAddressOfSubroutineAtPrgAddress("hello_world".into(), 0xcdef)
            ))),
            item_value(";! put address of hello_world at prg $CDEF")
        );
        assert_eq!(
            Ok(Some(Item::Directive(Directive::PutAddressAtPrgAddress(
                (0x01, 0x23),
                0xcdef
            )))),
            item_value(";! put address $0123 at prg $CDEF")
        );

        // labels
        assert_eq!(
            Ok(Some(Item::Label("hello_world".into()))),
            item_value("hello_world")
        );
        assert_eq!(
            Ok(Some(Item::Label("adc".into()))),
            item_value("adc ; super thing")
        );

        // address mode accumulator
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::Opcode(0x0A)))),
            item_value("ASL A")
        );
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::Opcode(0x0A)))),
            item_value("    ASL A ; hello")
        );

        // address mode absolute
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndTwoBytes(
                0x4C, 0x09, 0xAF
            )))),
            item_value("JMP $09AF")
        );
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndLabel(
                0x4C,
                "there".into()
            )))),
            item_value("JMP there")
        );

        // address mode absolute x indexed
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndTwoBytes(
                0x3D, 0x09, 0xAF
            )))),
            item_value("AND $09AF,X")
        );

        // address mode absolute y indexed
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndTwoBytes(
                0xBE, 0x09, 0xAF
            )))),
            item_value("LDX $09AF,Y")
        );

        // address mode immediate
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndByte(
                0xA0, 0x1E
            )))),
            item_value("LDY #$1E")
        );
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndByte(
                0xA0, 0x1E
            )))),
            item_value("LDY #30")
        );
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndByte(
                0xA0, 0x1E
            )))),
            item_value("LDY #%11110")
        );

        // address mode implied
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::Opcode(0x08)))),
            item_value("PHP")
        );

        // address mode indirect
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndTwoBytes(
                0x6C, 0xAB, 0xCD
            )))),
            item_value("JMP ($abcd)")
        );

        // address mode x indexed indirect
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndByte(
                0xA1, 0xAB
            )))),
            item_value("LDA ($ab,X)")
        );

        // address mode indirect y indexed
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndByte(
                0xB1, 0xAB
            )))),
            item_value("LDA ($ab),Y")
        );

        // address mode relative
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndByte(
                0x10, 0xFB
            )))),
            item_value("BPL $FB")
        );
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndLabelLocal(
                0x10,
                "back".into()
            )))),
            item_value("BPL @back")
        );

        // address mode zeropage
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndByte(
                0x24, 0xEC
            )))),
            item_value("BIT $EC")
        );

        // address mode zeropage x indexed
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndByte(
                0x75, 0x12
            )))),
            item_value("ADC $12,X")
        );

        // address mode zeropage y indexed
        assert_eq!(
            Ok(Some(Item::Instruction(Instruction::OpcodeAndByte(
                0xB6, 0x89
            )))),
            item_value("LDX $89,Y")
        );
    }

    #[test]
    fn parsing_values() {
        assert_eq!(Ok((" ", 123)), byte_dec("123 "));
        assert_eq!(Ok((" ", 0x7F)), byte_hex("$7F "));
        assert_eq!(Ok((" ", 0b110011)), byte_bin("%110011 "));
        assert_eq!(Ok((" ", (0xAB, 0x12))), address_hex("$AB12 "));
        assert_eq!(Ok((" ", 0xabcd1234)), long_address_hex("$abcd1234 "));
    }

    #[test]
    fn parsing_labels() {
        assert_eq!(Ok((" ", "hello_world".into())), label("hello_world "));
        assert_eq!(Ok((" ", "_nice".into())), label("_nice "));
        assert_eq!(Ok((":", "DO_THIS".into())), label("DO_THIS:"));
        assert_eq!(Ok(("", "pow2".into())), label("pow2"));
        assert!(matches!(label("1234"), Err(_)));
        assert!(matches!(label(" "), Err(_)));
    }

    #[test]
    fn parsing_local_labels() {
        assert_eq!(Ok((" ", "Loop".into())), label("@Loop "));
        assert_eq!(Ok((" ", "1".into())), label("@1 "));
        assert!(matches!(label("abc"), Err(_)));
        assert!(matches!(label("@:"), Err(_)));
    }
}
