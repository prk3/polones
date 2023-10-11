mod constants;
mod parser;
mod types;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use types::{Directive, Instruction, Item};

use crate::parser::item;

struct Context {
    in_path: String,
    prg_rom: Vec<u8>,
    prg_rom_pointer: usize,
    chr_rom: Vec<u8>,
    label_addresses: HashMap<String, usize>,
    label_addresses_local: HashMap<String, usize>,
    address_slots_abs: Vec<(String, usize)>,
    address_slots_rel: Vec<(String, usize)>,
    address_slots_local_abs: Vec<(String, usize)>,
    address_slots_local_rel: Vec<(String, usize)>,
}

impl Context {
    fn new(in_path: String) -> Self {
        Self {
            in_path,
            prg_rom: vec![0; 32 * 1024],
            prg_rom_pointer: 0,
            chr_rom: vec![0; 8 * 1024],
            label_addresses: HashMap::new(),
            label_addresses_local: HashMap::new(),
            address_slots_abs: Vec::new(),
            address_slots_rel: Vec::new(),
            address_slots_local_abs: Vec::new(),
            address_slots_local_rel: Vec::new(),
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (in_path, out_path) = match (args.next(), args.next(), args.next()) {
        (Some(in_path), Some(out_path), None) => (in_path, out_path),
        _ => {
            println!("Bad args. Please provide in path and out path.");
            std::process::exit(1);
        }
    };

    let mut context = Context::new(in_path.clone());

    let source = match std::fs::read_to_string(&in_path) {
        Ok(source) => source,
        Err(error) => {
            println!("Could not read in file: {error}");
            std::process::exit(1);
        }
    };
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let (_, item_opt) = item(line).unwrap_or_else(|error| {
            eprintln!("Parsing failed at line {}: {line}", i + 1);
            eprintln!("{error}");
            std::process::exit(1);
        });
        let item = if let Some(item) = item_opt {
            item
        } else {
            continue;
        };

        if let Err(error) = process_item(item, &mut context) {
            eprintln!("Processing failed at line {i}: {line}");
            eprintln!("{error}");
            std::process::exit(1);
        }
    }

    if let Err(error) = finalize_file_scope(&mut context) {
        eprintln!("Finalizing failed");
        eprintln!("{error}");
        std::process::exit(1);
    }

    let out_file = std::fs::File::create(&out_path).unwrap_or_else(|error| {
        eprintln!("Could not create out file: {error}");
        std::process::exit(1);
    });

    let mut writer = BufWriter::with_capacity(16 * 1024, out_file);

    if let Err(error) = write_context(&context, &mut writer) {
        eprintln!("Could not write context to out file: {error}");
        std::process::exit(1);
    }
}

fn process_item(item: Item, context: &mut Context) -> Result<(), String> {
    match item {
        Item::LabelLocal(label) => {
            let entry = context.label_addresses_local.entry(label);
            match entry {
                Entry::Occupied(entry) => {
                    Err(format!("Local label {:?} already defined", entry.key()))
                }
                Entry::Vacant(entry) => {
                    entry.insert(context.prg_rom_pointer);
                    Ok(())
                }
            }
        }
        Item::Label(label) => {
            // take care of local labels in the block that just ended
            finalize_label_scope(context)?;
            context.label_addresses_local = HashMap::new();

            // save new label
            let entry = context.label_addresses.entry(label);
            match entry {
                Entry::Occupied(entry) => Err(format!("Label {:?} already defined", entry.key())),
                Entry::Vacant(entry) => {
                    entry.insert(context.prg_rom_pointer);
                    Ok(())
                }
            }
        }
        // TODO handle out of bounds writes
        Item::Instruction(instruction) => match instruction {
            Instruction::Opcode(opcode) => {
                context.prg_rom[context.prg_rom_pointer] = opcode;
                context.prg_rom_pointer += 1;
                Ok(())
            }
            Instruction::OpcodeAndByte(opcode, byte) => {
                context.prg_rom[context.prg_rom_pointer] = opcode;
                context.prg_rom[context.prg_rom_pointer + 1] = byte;
                context.prg_rom_pointer += 2;
                Ok(())
            }
            Instruction::OpcodeAndTwoBytes(opcode, lo, hi) => {
                context.prg_rom[context.prg_rom_pointer] = opcode;
                context.prg_rom[context.prg_rom_pointer + 1] = lo;
                context.prg_rom[context.prg_rom_pointer + 2] = hi;
                context.prg_rom_pointer += 3;
                Ok(())
            }
            Instruction::OpcodeAbsAndLabel(opcode, label) => {
                context.prg_rom[context.prg_rom_pointer] = opcode;
                context
                    .address_slots_abs
                    .push((label, context.prg_rom_pointer + 1));
                context.prg_rom_pointer += 3;
                Ok(())
            }
            Instruction::OpcodeRelAndLabel(opcode, label) => {
                context.prg_rom[context.prg_rom_pointer] = opcode;
                context
                    .address_slots_rel
                    .push((label, context.prg_rom_pointer + 1));
                context.prg_rom_pointer += 2;
                Ok(())
            }
            Instruction::OpcodeAbsAndLocalLabel(opcode, label) => {
                context.prg_rom[context.prg_rom_pointer] = opcode;
                context
                    .address_slots_local_abs
                    .push((label, context.prg_rom_pointer + 1));
                context.prg_rom_pointer += 3;
                Ok(())
            }
            Instruction::OpcodeRelAndLocalLabel(opcode, label) => {
                context.prg_rom[context.prg_rom_pointer] = opcode;
                context
                    .address_slots_local_rel
                    .push((label, context.prg_rom_pointer + 1));
                context.prg_rom_pointer += 2;
                Ok(())
            }
        },
        Item::Directive(directive) => match directive {
            Directive::PutAddressAtPrgAddress(lo, hi, prg_address) => {
                if (0..=0x7FFE).contains(&prg_address) {
                    context.prg_rom[prg_address] = lo;
                    context.prg_rom[prg_address + 1] = hi;
                    Ok(())
                } else {
                    Err(format!("PRG address out of bounds"))
                }
            }
            Directive::PutAddressOfSubroutineAtPrgAddress(label, prg_address) => {
                if (0..=0x7FFE).contains(&prg_address) {
                    context.address_slots_abs.push((label, prg_address));
                    Ok(())
                } else {
                    Err(format!("PRG address out of bounds"))
                }
            }
            Directive::PutImageAtChrAddress(path, chr_address) => 'arm: {
                let full_path = PathBuf::from(&context.in_path).parent().unwrap().join(path);
                let pages = match load_chr_image(full_path.as_path()) {
                    Ok(pages) => pages,
                    Err(error) => break 'arm Err(format!("Failed to load chr image: {error}")),
                };

                if chr_address % (4 * 1024) != 0 {
                    break 'arm Err(format!("chr_address is not aligned to chr page boundary"));
                }

                if chr_address + (pages.len() * 4 * 1024) > (2 * 4 * 1024) {
                    break 'arm Err(format!("chr data would overflow chr section"));
                }

                let mut i = 0;
                for page in pages {
                    for byte in page {
                        context.chr_rom[chr_address + i] = byte;
                        i += 1;
                    }
                }

                Ok(())
            }
            Directive::Other(words) => {
                eprintln!("Found unknown directive: {words:?}");
                Ok(())
            }
        },
    }
}

fn finalize_label_scope(context: &mut Context) -> Result<(), String> {
    for (label, slot_address) in context.address_slots_local_abs.drain(..) {
        match context.label_addresses_local.get(&label) {
            Some(&code_address) => {
                // TODO these conversions will not ok for prg size > 32k
                let code_address_on_cpu_bus = 0x8000 + code_address;
                let lo = code_address_on_cpu_bus as u8;
                let hi = (code_address_on_cpu_bus >> 8) as u8;

                context.prg_rom[slot_address] = lo;
                context.prg_rom[slot_address + 1] = hi;
            }
            None => {
                return Err(format!("Local label {label:?} not defined"));
            }
        }
    }
    for (label, slot_address) in context.address_slots_local_rel.drain(..) {
        match context.label_addresses_local.get(&label) {
            Some(&code_address) => {
                let offset = code_address as i64 - (slot_address as i64 + 1);
                if let Ok(offset) = i8::try_from(offset) {
                    context.prg_rom[slot_address] = offset.to_be_bytes()[0];
                } else {
                    return Err(format!("Code with {label:?} is too far"));
                }
            }
            None => {
                return Err(format!("Local label {label:?} not defined"));
            }
        }
    }
    Ok(())
}

fn finalize_file_scope(context: &mut Context) -> Result<(), String> {
    for (label, slot_address) in context.address_slots_abs.drain(..) {
        match context.label_addresses.get(&label) {
            Some(&code_address) => {
                // TODO these conversions will not ok for prg size > 32k
                let code_address_on_cpu_bus = 0x8000 + code_address;
                let lo = code_address_on_cpu_bus as u8;
                let hi = (code_address_on_cpu_bus >> 8) as u8;

                context.prg_rom[slot_address] = lo;
                context.prg_rom[slot_address + 1] = hi;
            }
            None => {
                return Err(format!("Label {label:?} not defined"));
            }
        }
    }
    for (label, slot_address) in context.address_slots_rel.drain(..) {
        match context.label_addresses.get(&label) {
            Some(&code_address) => {
                let offset = code_address as i64 - (slot_address as i64 + 1);
                if let Ok(offset) = u8::try_from(offset) {
                    context.prg_rom[slot_address] = offset;
                } else {
                    return Err(format!("Code with {label:?} is too far"));
                }
            }
            None => {
                return Err(format!("Label {label:?} not defined"));
            }
        }
    }
    Ok(())
}

fn write_context<W: std::io::Write>(context: &Context, writer: &mut W) -> std::io::Result<()> {
    // header
    writer.write(b"NES\x1A")?;
    writer.write(&[0x02])?;
    writer.write(&[0x01])?;
    writer.write(&[0b00000000])?;
    writer.write(&[0b00001000])?;
    writer.write(&[0x00])?;
    writer.write(&[0x00])?;
    writer.write(&[0x00])?;
    writer.write(&[0, 0, 0, 0, 0])?;

    // prg rom
    writer.write(&context.prg_rom[..])?;

    // chr rom
    writer.write(&context.chr_rom[..])?;

    Ok(())
}

fn load_chr_image(path: &Path) -> Result<Vec<[u8; 4096]>, String> {
    use nom::bytes::complete::*;
    use nom::character::*;
    use nom::multi::*;
    use nom::sequence::*;

    let bytes = std::fs::read(path).map_err(|e| format!("Could not open image file: {e}"))?;

    let parsed: nom::IResult<&[u8], _, nom::error::VerboseError<_>> = tuple((
        tag("P6\n"),
        fold_many0(
            tuple((tag("#"), take_until("\n"), tag("\n"))),
            || (),
            |_, _| (),
        ),
        take_while1(is_digit),
        tag(" "),
        take_while1(is_digit),
        tag("\n"),
        nom::multi::fold_many0(
            tuple((tag("#"), take_until("\n"), tag("\n"))),
            || (),
            |_, _| (),
        ),
        take_while1(is_digit),
        tag("\n"),
    ))(bytes.as_ref());

    let (rest, (_, _, width, _, height, _, _, max, _)) = match parsed {
        Ok(parsed) => parsed,
        Err(error) => {
            let mut error = error.to_string();
            if error.len() > 10_000 {
                let mut index = 10_000;
                while !error.is_char_boundary(index) {
                    index -= 1;
                }
                error.truncate(index);
                error.push_str("...");
            }

            return Err(format!("Could not parse image file: {error}"));
        }
    };

    let width: usize = std::str::from_utf8(width)
        .unwrap()
        .parse()
        .map_err(|e| format!("Could not parse width: {e}"))?;
    let height: usize = std::str::from_utf8(height)
        .unwrap()
        .parse()
        .map_err(|e| format!("Could not parse height: {e}"))?;
    let max: usize = std::str::from_utf8(max)
        .unwrap()
        .parse()
        .map_err(|e| format!("Could not parse max: {e}"))?;

    if max != 255 {
        return Err("Max other than 255 is not supported".into());
    }

    if width % 128 != 0 || width < 128 {
        return Err("Width must be a multiple of 128".into());
    }

    if height % 128 != 0 || height < 128 {
        return Err("Height must be a multiple of 128".into());
    }

    if rest.len() < width * height * 3 {
        return Err("Image data too short".into());
    }

    let mut output = Vec::with_capacity(width as usize / 128);

    fn patterns(r: u8, g: u8, b: u8) -> u16 {
        let sum = r as u32 + g as u32 + b as u32;
        match sum {
            0..=191 => u16::from_be_bytes([0, 0]),
            192..=383 => u16::from_be_bytes([0, 1]),
            384..=575 => u16::from_be_bytes([1, 0]),
            _ => u16::from_be_bytes([1, 1]),
        }
    }
    for p in 0..(width / 128) {
        let mut page = [0u8; { 4 * 1024 }];
        for cy in 0..16 {
            for cx in 0..16 {
                for y in 0..8 {
                    let start = (3 * width * (cy * 8 + y)) + (p * 3 * 128) + (cx * 3 * 8);
                    let byte = patterns(rest[start + 0], rest[start + 1], rest[start + 2]) << 7
                        | patterns(rest[start + 3], rest[start + 4], rest[start + 5]) << 6
                        | patterns(rest[start + 6], rest[start + 7], rest[start + 8]) << 5
                        | patterns(rest[start + 9], rest[start + 10], rest[start + 11]) << 4
                        | patterns(rest[start + 12], rest[start + 13], rest[start + 14]) << 3
                        | patterns(rest[start + 15], rest[start + 16], rest[start + 17]) << 2
                        | patterns(rest[start + 18], rest[start + 19], rest[start + 20]) << 1
                        | patterns(rest[start + 21], rest[start + 22], rest[start + 23]);
                    let [high, low] = byte.to_be_bytes();
                    page[cy << 8 | cx << 4 | y] = low;
                    page[cy << 8 | cx << 4 | 1 << 3 | y] = high;
                }
            }
        }
        output.push(page);
    }

    Ok(output)
}
