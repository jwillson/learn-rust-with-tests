// ANCHOR: main
use std::fs::OpenOptions;
use std::sync::Arc;

use io_sorting_v2::{FileSystemPlayerStore, player_server};

const DB_FILE_NAME: &str = "game.db.json";

#[tokio::main]
async fn main() {
    let database = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(DB_FILE_NAME)
        .expect("failed to open the database file");

    let store =
        Arc::new(FileSystemPlayerStore::new(database).expect("failed to load the database"));
    let app = player_server(store);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000")
        .await
        .expect("failed to bind to port 5000");

    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
// ANCHOR_END: main
