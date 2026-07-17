// ANCHOR: cli_main
use std::fs::OpenOptions;
use std::io::BufReader;
use std::sync::Arc;

use command_line_v1::{Cli, FileSystemPlayerStore};

const DB_FILE_NAME: &str = "game.db.json";

fn main() {
    let database = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(DB_FILE_NAME)
        .expect("failed to open the database file");

    let store =
        Arc::new(FileSystemPlayerStore::new(database).expect("failed to load the database"));

    println!("Let's play poker");
    println!("Type \"{{Name}} wins\" to record a win");

    let stdin = BufReader::new(std::io::stdin());
    let mut cli = Cli::new(store, stdin);
    cli.play_poker();
}
// ANCHOR_END: cli_main
