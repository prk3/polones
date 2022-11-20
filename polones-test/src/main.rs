use clap::Parser;
use polones_core::game_file::GameFile;
use polones_core::nes::{Audio, Display, Frame, Input, Nes, PortState};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureAccess, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::cell::RefCell;
use std::path::Component;
use std::rc::Rc;

struct SdlDisplay {
    texture: Option<Texture<'static>>,
    _texture_creator: Option<Box<TextureCreator<WindowContext>>>,
    canvas: Option<Canvas<Window>>,
}

impl Display for SdlDisplay {
    fn draw(&mut self, frame: Box<Frame>) {
        if let (Some(canvas), Some(texture)) = (&mut self.canvas, &mut self.texture) {
            texture
                .update(
                    None,
                    unsafe { std::slice::from_raw_parts(&frame[0][0].0, 256 * 240) },
                    3 * 256,
                )
                .unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }
    }
}

struct SdlInput {
    inputs: Vec<u8>,
    inputs_used: usize,
}

impl Input for SdlInput {
    fn read_port_1(&mut self) -> PortState {
        if self.inputs_used < self.inputs.len() {
            let input = self.inputs[self.inputs_used];
            self.inputs_used += 1;
            PortState::Gamepad {
                a: input & 0b10000000 > 0,
                b: input & 0b01000000 > 0,
                select: input & 0b00100000 > 0,
                start: input & 0b00010000 > 0,
                up: input & 0b00001000 > 0,
                down: input & 0b00000100 > 0,
                left: input & 0b00000010 > 0,
                right: input & 0b00000001 > 0,
            }
        } else {
            PortState::Unplugged
        }
    }

    fn read_port_2(&mut self) -> PortState {
        if self.inputs_used < self.inputs.len() {
            let input = self.inputs[self.inputs_used];
            self.inputs_used += 1;
            PortState::Gamepad {
                a: input & 0b10000000 > 0,
                b: input & 0b01000000 > 0,
                select: input & 0b00100000 > 0,
                start: input & 0b00010000 > 0,
                up: input & 0b00001000 > 0,
                down: input & 0b00000100 > 0,
                left: input & 0b00000010 > 0,
                right: input & 0b00000001 > 0,
            }
        } else {
            PortState::Unplugged
        }
    }
}

#[derive(Clone)]
struct SdlInputShared(Rc<RefCell<SdlInput>>);
impl Input for SdlInputShared {
    fn read_port_1(&mut self) -> PortState {
        self.0.borrow_mut().read_port_1()
    }
    fn read_port_2(&mut self) -> PortState {
        self.0.borrow_mut().read_port_1()
    }
}

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

    let (canvas, texture_creator, texture) = if args.preview {
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

        (Some(canvas), Some(texture_creator), Some(texture))
    } else {
        (None, None, None)
    };

    let display = SdlDisplay {
        canvas,
        _texture_creator: texture_creator,
        texture,
    };

    let input = SdlInputShared(Rc::new(RefCell::new(SdlInput {
        inputs_used: 0,
        inputs,
    })));

    let mut nes =
        Nes::new(game_file, display, input.clone(), Discard).expect("Could not start the game");

    let start_time = std::time::Instant::now();

    loop {
        for _ in 0..(1_789_773 / 60) {
            nes.run_one_cpu_tick();
        }

        let input = input.0.borrow();

        if input.inputs_used >= input.inputs.len() {
            break;
        }
    }

    let time = start_time.elapsed().as_secs_f64();

    println!("Emulation took {time}s");

    // make sure nes is dropped before texture creator
    drop(nes);
}
