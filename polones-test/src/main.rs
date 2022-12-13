use clap::Parser;
use polones_core::game_file::GameFile;
use polones_core::nes::{GamepadState, Nes, PortState};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureAccess};
use std::path::Component;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    rom: String,

    #[arg(short, long)]
    preview: bool,

    #[arg(short, long)]
    flamegraph: bool,
}

impl Args {
    fn collect(self) -> Vec<String> {
        let Self {
            rom,
            preview,
            flamegraph,
        } = self;

        let mut args = Vec::new();
        if preview {
            args.push("--preview".into());
        }
        if flamegraph {
            args.push("--flamegraph".into());
        }
        args.push(rom);
        args
    }
}

fn main() {
    let args = Args::parse();

    let rom_filename = {
        let rom_path = std::path::Path::new(&args.rom);
        match rom_path.components().last() {
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

    if args.flamegraph {
        let args = Args {
            flamegraph: false,
            ..args
        };

        let mut flamegraph_command = std::process::Command::new("cargo");
        flamegraph_command.env("CARGO_PROFILE_RELEASE_DEBUG", "true");

        let mut flamegraph_args: Vec<String> = vec![
            "flamegraph".into(),
            "--output".into(),
            format!("./flamegraphs/{rom_filename}.svg"),
            "--freq".into(),
            "20000".into(),
            "--".into(),
        ];
        flamegraph_args.extend(args.collect().into_iter());
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

    let file_contents = match std::fs::read(&args.rom) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("Could not read ROM file: {error}");
            std::process::exit(1)
        }
    };
    let game_file = match GameFile::read(args.rom.clone(), file_contents) {
        Ok(game_file) => game_file,
        Err(_error) => {
            eprintln!("Could not parse ROM file");
            std::process::exit(1);
        }
    };

    let inputs_path = format!("./inputs/{}.bin", rom_filename);
    let inputs = match std::fs::read(inputs_path) {
        Ok(inputs) => inputs,
        Err(error) => {
            eprintln!("Could not read inputs file: {error}");
            std::process::exit(1);
        }
    };

    let mut preview = if args.preview {
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

    let start_time = std::time::Instant::now();

    let mut display_version = nes.display.version;
    let mut input_version = nes.input.read_version;

    loop {
        for _ in 0..100 {
            nes.run_one_cpu_tick();
        }

        if nes.display.version > display_version {
            display_version = nes.display.version;

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
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
            }
        }

        if nes.input.read_version > input_version {
            input_version = nes.input.read_version;

            if input_version as usize * 2 >= inputs.len() {
                break;
            }

            let i = 2 * input_version as usize;
            nes.input.port_1 = PortState::Gamepad(GamepadState::from_byte(inputs[i]));
            nes.input.port_2 = PortState::Gamepad(GamepadState::from_byte(inputs[i + 1]));
        }
    }

    let time = start_time.elapsed().as_secs_f64();

    println!("Emulation took {time}s");

    // make sure nes is dropped before texture creator
    drop(nes);
}
