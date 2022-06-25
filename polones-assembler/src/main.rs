mod types;
mod constants;
mod parser;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io::BufWriter;
use types::{Item, Instruction, Directive};

use crate::parser::item;

struct Context {
    prg_rom: Vec<u8>,
    prg_rom_pointer: usize,
    chr_rom: Vec<u8>,
    label_addresses: HashMap<String, usize>,
    label_addresses_local: HashMap<String, usize>,
    address_slots: Vec<(String, usize)>,
    address_slots_local: Vec<(String, usize)>,
}

impl Context {
    fn new() -> Self {
        Self {
            prg_rom: vec![0; 32 * 1024],
            prg_rom_pointer: 0,
            chr_rom: vec![0; 8 * 1024],
            label_addresses: HashMap::new(),
            label_addresses_local: HashMap::new(),
            address_slots: Vec::new(),
            address_slots_local: Vec::new(),
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

    let mut context = Context::new();

    let source = match std::fs::read_to_string(&in_path) {
        Ok(source) => source,
        Err(error) => {
            println!("Could not read in file: {error}");
            std::process::exit(1);
        }
    };
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let (_, item_opt) = item(line)
            .unwrap_or_else(|error| {
                eprintln!("Parsing failed at line {i}: {line}");
                eprintln!("{error}");
                std::process::exit(1);
            });
        let item = if let Some(item) = item_opt { item } else { continue };

        if let Err(error) = process_item(item, &mut context) {
            eprintln!("Processing failed at line {i}: {line}");
            eprintln!("{error}");
            std::process::exit(1);
        }
    }

    if let Err(error) = finalize_context(&mut context) {
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
            for (label, slot_address) in context.address_slots_local.drain(..) {
                match context.label_addresses_local.get(&label) {
                    Some(&code_address) => {

                    }
                    None => {
                        return Err(format!("Local label {label:?} not defined"));
                    }
                }
            }
            context.label_addresses_local = HashMap::new();

            // save new label
            let entry = context.label_addresses.entry(label);
            match entry {
                Entry::Occupied(entry) => {
                    Err(format!("Label {:?} already defined", entry.key()))
                }
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
            Instruction::OpcodeAndLabel(opcode, label) => {
                context.prg_rom[context.prg_rom_pointer] = opcode;
                context.address_slots.push((label, context.prg_rom_pointer + 1));
                context.prg_rom_pointer += 3;
                Ok(())
            }
            Instruction::OpcodeAndLabelLocal(opcode, label) => {
                context.prg_rom[context.prg_rom_pointer] = opcode;
                context.address_slots_local.push((label, context.prg_rom_pointer + 1));
                context.prg_rom_pointer += 2;
                Ok(())
            }
        }
        Item::Directive(directive) => match directive {
            Directive::PutAddressAtPrgAddress((lo, hi), prg_address) => {
                if prg_address < context.prg_rom.len() - 2 {
                    context.prg_rom[prg_address] = lo;
                    context.prg_rom[prg_address + 1] = hi;
                    Ok(())
                } else {
                    Err(format!("PRG address out of bounds"))
                }
            }
            Directive::PutAddressOfSubroutineAtPrgAddress(label, prg_address) => {
                if prg_address < context.prg_rom.len() - 2 {
                    context.address_slots.push((label, prg_address));
                    Ok(())
                } else {
                    Err(format!("PRG address out of bounds"))
                }
            }
            Directive::Other(words) => {
                eprintln!("Found unknown directive: {words:?}");
                Ok(())
            }
        }
    }
}

fn finalize_context(context: &mut Context) -> Result<(), String> {
    for (label, slot_address) in context.address_slots.drain(..) {
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
