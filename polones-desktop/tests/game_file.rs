use polones_core::game_file::GameFile;

#[test]
fn reads_all_game_files() {
    use std::fs;

    let dir = fs::read_dir("./tests/roms").unwrap();

    for entry in dir {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |extension| extension == "nes") {
            println!("reading {}", path.display());
            let contents = fs::read(&path).unwrap();
            let game_file = GameFile::read(path.display().to_string(), contents).unwrap();
            println!("read {}, format {:?}, mapper {}", path.display(), game_file.format, game_file.mapper);
        }
    }
}
