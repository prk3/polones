use clap::{Parser, Subcommand};
use polones_core::game_file::GameFile;
use polones_core::nes::{GamepadState, Nes, PortState};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureAccess};
use std::collections::BTreeMap;
use std::path::Component;

#[derive(Debug, Parser)]
#[command(name = "polones-test")]
#[command(about = "Tool for testing polones emulator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Replay {
        rom: String,

        #[arg(short, long)]
        preview: bool,

        #[arg(short, long)]
        flamegraph: bool,
    },
    Stats {
        dir: String,
    },
}

impl Cli {
    fn collect(self) -> Vec<String> {
        match self.command {
            Commands::Replay {
                rom,
                preview,
                flamegraph,
            } => {
                let mut args = vec!["replay".into()];
                if preview {
                    args.push("--preview".into());
                }
                if flamegraph {
                    args.push("--flamegraph".into());
                }
                args.push(rom);
                args
            }
            Commands::Stats { dir } => {
                let mut args = vec!["stats".into()];
                args.push(dir);
                args
            }
        }
    }
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Replay {
            rom,
            preview,
            flamegraph,
        } => {
            replay(rom, preview, flamegraph);
        }
        Commands::Stats { dir } => {
            stats(dir);
        }
    }
}

fn replay(rom: String, preview: bool, flamegraph: bool) {
    let rom_filename = {
        let rom_path = std::path::Path::new(&rom);
        match rom_path.components().next_back() {
            Some(Component::Normal(normal)) => normal.to_string_lossy().into_owned(),
            Some(_) => {
                eprintln!("Path does not end with normal");
                std::process::exit(1);
            }
            None => {
                eprintln!("Path is empty");
                std::process::exit(1);
            }
        }
    };

    if flamegraph {
        let args = Cli {
            command: Commands::Replay {
                flamegraph: false,
                rom,
                preview,
            },
        };

        let mut flamegraph_command = std::process::Command::new("cargo");
        flamegraph_command.env("CARGO_PROFILE_RELEASE_DEBUG", "true");

        let mut flamegraph_args: Vec<String> = vec![
            "flamegraph".into(),
            "--output".into(),
            format!("../flamegraphs/{rom_filename}.svg"),
            "--freq".into(),
            "20000".into(),
            "--".into(),
        ];
        flamegraph_args.extend(args.collect());
        flamegraph_command.args(flamegraph_args);

        let mut child = match flamegraph_command.spawn() {
            Ok(child) => child,
            Err(error) => {
                eprintln!("Could not spawn flamegraph command: {error}");
                std::process::exit(1);
            }
        };
        let status = match child.wait() {
            Ok(status) => status,
            Err(error) => {
                eprintln!("Could not wait for test result: {error}");
                std::process::exit(1);
            }
        };
        std::process::exit(status.code().unwrap_or(1))
    }

    let file_contents = match std::fs::read(&rom) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("Could not read ROM file: {error}");
            std::process::exit(1)
        }
    };
    let game_file = match GameFile::read(rom.clone(), file_contents) {
        Ok(game_file) => game_file,
        Err(_error) => {
            eprintln!("Could not parse ROM file");
            std::process::exit(1);
        }
    };

    let inputs_path = format!("../inputs/{}.bin", rom_filename);
    let inputs = match std::fs::read(inputs_path) {
        Ok(inputs) => inputs,
        Err(error) => {
            eprintln!("Could not read inputs file: {error}");
            std::process::exit(1);
        }
    };

    let mut preview = if preview {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let game_window = video_subsystem
            .window("nes display", 256, 240)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = game_window.into_canvas().build().unwrap();
        canvas.present();

        let texture_creator = Box::new(canvas.texture_creator());
        let texture = texture_creator
            .create_texture::<PixelFormatEnum>(
                PixelFormatEnum::RGB24,
                TextureAccess::Static,
                256,
                240,
            )
            .unwrap();

        let texture = unsafe { std::mem::transmute::<Texture<'_>, Texture<'static>>(texture) };

        Some((canvas, texture_creator, texture))
    } else {
        None
    };

    let mut nes = Nes::new(game_file).expect("Could not start the game");
    nes.input.port_1 =
        PortState::Gamepad(GamepadState::from_byte(inputs.get(0).cloned().unwrap_or(0)));
    nes.input.port_2 =
        PortState::Gamepad(GamepadState::from_byte(inputs.get(1).cloned().unwrap_or(0)));

    let start_time = std::time::Instant::now();

    let mut display_version = nes.display.version;
    let mut input_version = nes.input.read_version;

    while (input_version as usize + 1) * 2 < inputs.len() {
        nes.run_one_cpu_tick();

        if nes.display.version > display_version {
            if let Some((canvas, _, texture)) = &mut preview {
                texture
                    .update(
                        None,
                        unsafe {
                            std::slice::from_raw_parts(&nes.display.frame[0][0].0, 256 * 240)
                        },
                        3 * 256,
                    )
                    .unwrap();
                canvas.copy(texture, None, None).unwrap();
                canvas.present();
            }

            display_version = nes.display.version;
        }

        if nes.input.read_version > input_version {
            let i = (input_version as usize + 1) * 2;
            nes.input.port_1 = PortState::Gamepad(GamepadState::from_byte(inputs[i]));
            nes.input.port_2 = PortState::Gamepad(GamepadState::from_byte(inputs[i + 1]));

            input_version = nes.input.read_version;
        }
    }

    let seconds = start_time.elapsed().as_secs_f64();

    println!("Emulation took {seconds}s");

    // make sure nes is dropped before texture creator
    drop(nes);
}

fn stats(dir: String) {
    enum Outcome {
        Success { rom: String },
        Failure { rom: String, r#type: FailureType },
    }

    enum FailureType {
        Parse,
        Start { mapper: u16 },
        Panic,
    }

    let mut outcomes: Vec<Outcome> = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("Could not read files in dir: {error}");
            std::process::exit(1);
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                eprintln!("Could not read dir entry: {error}");
                continue;
            }
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }

        let rom: String = entry.path().to_string_lossy().into_owned();

        let rom_filename = {
            match entry.path().components().next_back() {
                Some(Component::Normal(normal)) => normal.to_string_lossy().into_owned(),
                Some(_) => {
                    eprintln!("Path does not end with normal");
                    continue;
                }
                None => {
                    eprintln!("Path is empty");
                    continue;
                }
            }
        };

        if !rom_filename.ends_with(".nes") {
            continue;
        }

        let file_contents = match std::fs::read(entry.path()) {
            Ok(contents) => contents,
            Err(error) => {
                eprintln!("Could not read ROM file: {error}");
                continue;
            }
        };

        let rom_filename_clone = rom_filename.clone();

        let run_result = std::panic::catch_unwind(|| {
            let game_file = match GameFile::read(rom.clone(), file_contents) {
                Ok(game_file) => game_file,
                Err(_error) => {
                    return Outcome::Failure {
                        rom: rom_filename,
                        r#type: FailureType::Parse,
                    };
                }
            };
            let mapper = game_file.mapper;

            let mut nes = match Nes::new(game_file) {
                Ok(nes) => nes,
                Err(_error) => {
                    return Outcome::Failure {
                        rom: rom_filename,
                        r#type: FailureType::Start { mapper },
                    };
                }
            };

            for _ in 0..10_000 {
                nes.run_one_cpu_tick();
            }

            Outcome::Success { rom: rom_filename }
        });

        match run_result {
            Ok(outcome) => {
                outcomes.push(outcome);
            }
            Err(_error) => {
                outcomes.push(Outcome::Failure {
                    rom: rom_filename_clone,
                    r#type: FailureType::Panic,
                });
            }
        }
    }

    let mut total = 0;
    let mut successes = 0;
    let mut fails_parse = 0;
    let mut fails_start = 0;
    let mut fails_start_per_mapper = BTreeMap::<u16, i32>::new();
    let mut panics = 0;

    for outcome in &outcomes {
        total += 1;
        match outcome {
            Outcome::Success { .. } => successes += 1,
            Outcome::Failure {
                r#type: FailureType::Parse,
                ..
            } => fails_parse += 1,
            Outcome::Failure {
                r#type: FailureType::Start { mapper },
                ..
            } => {
                fails_start += 1;
                *fails_start_per_mapper.entry(*mapper).or_default() += 1;
            }
            Outcome::Failure {
                r#type: FailureType::Panic,
                ..
            } => panics += 1,
        }
    }

    println!("Outcome");
    println!("     Roms tested: {total:>5}");

    if total == 0 {
        return;
    }

    println!(
        "         Working: {successes:>5} ({:>6.2}%)",
        successes as f32 / total as f32 * 100.0
    );
    println!(
        "Failing at parse: {fails_parse:>5} ({:>6.2}%)",
        fails_parse as f32 / total as f32 * 100.0
    );
    println!(
        "Failing at start: {fails_start:>5} ({:>6.2}%)",
        fails_start as f32 / total as f32 * 100.0
    );
    println!(
        "       Panicking: {panics:>5} ({:>6.2}%)",
        panics as f32 / total as f32 * 100.0
    );

    if !fails_start_per_mapper.is_empty() {
        println!();
        println!("Failures at start per mapper");
        for (mapper, count) in fails_start_per_mapper.iter() {
            println!(
                "{mapper:>3}: {count:>5} ({:>6.2}%)",
                *count as f32 / fails_start as f32 * 100.0
            );
        }
    }

    if fails_parse > 0 {
        println!();
        println!("ROMs that fail at parse");
        for outcome in &outcomes {
            if let Outcome::Failure {
                r#type: FailureType::Parse,
                rom,
            } = outcome
            {
                println!("{rom}");
            }
        }
    }

    if fails_start > 0 {
        println!();
        println!("ROMs that fail at start");
        for outcome in &outcomes {
            if let Outcome::Failure {
                r#type: FailureType::Start { mapper },
                rom,
            } = outcome
            {
                println!("{rom:<85} (mapper {mapper})");
            }
        }
    }

    if panics > 0 {
        println!();
        println!("ROMs that panic");
        for outcome in &outcomes {
            if let Outcome::Failure {
                r#type: FailureType::Panic,
                rom,
            } = outcome
            {
                println!("{rom}");
            }
        }
    }
}
