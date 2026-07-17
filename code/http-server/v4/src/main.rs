// ANCHOR: main
use std::sync::Arc;

use http_server_v4::{InMemoryPlayerStore, player_server};

#[tokio::main]
async fn main() {
    let store = Arc::new(InMemoryPlayerStore::default());
    let app = player_server(store);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000")
        .await
        .expect("failed to bind to port 5000");

    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
// ANCHOR_END: main
