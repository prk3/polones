use nes_lib::game_file::GameFile;

#[test]
fn reads_all_game_files() {
    use std::fs;

    let dir = fs::read_dir("./tests/games").unwrap();

    for entry in dir {
        let path = entry.unwrap().path();
        println!("reading {}", path.display());
        let contents = fs::read(&path).unwrap();
        let game_file = GameFile::read(contents).unwrap();
        println!("read    {}, format {:?}, mapper {}", path.display(), game_file.format, game_file.mapper);
    }
}
