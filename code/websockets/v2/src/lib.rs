use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::any;

// ANCHOR: game
pub trait Game: Send + Sync {
    fn start(&self, number_of_players: u32);
    fn finish(&self, winner: &str);
}
// ANCHOR_END: game

// ANCHOR: server
pub fn game_server(game: Arc<dyn Game>) -> Router {
    Router::new().route("/ws", any(ws_handler)).with_state(game)
}

async fn ws_handler(State(game): State<Arc<dyn Game>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| play_game(socket, game))
}

async fn play_game(mut socket: WebSocket, game: Arc<dyn Game>) {
    if let Some(number_of_players) = read_text(&mut socket).await
        && let Ok(count) = number_of_players.trim().parse::<u32>()
    {
        game.start(count);
    }

    if let Some(winner) = read_text(&mut socket).await {
        game.finish(winner.trim());
    }
}

async fn read_text(socket: &mut WebSocket) -> Option<String> {
    match socket.recv().await {
        Some(Ok(Message::Text(text))) => Some(text.to_string()),
        _ => None,
    }
}
// ANCHOR_END: server

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::SinkExt;
    use std::sync::Mutex;
    use tokio_tungstenite::tungstenite::Message as ClientMessage;

    // ANCHOR: spy
    #[derive(Default)]
    struct GameSpy {
        started_with: Mutex<Option<u32>>,
        finished_with: Mutex<Option<String>>,
    }

    impl Game for GameSpy {
        fn start(&self, number_of_players: u32) {
            *self.started_with.lock().unwrap() = Some(number_of_players);
        }

        fn finish(&self, winner: &str) {
            *self.finished_with.lock().unwrap() = Some(winner.to_string());
        }
    }
    // ANCHOR_END: spy

    // ANCHOR: test
    #[tokio::test]
    async fn plays_a_game_over_a_websocket() {
        let game = Arc::new(GameSpy::default());
        let app = game_server(game.clone());

        // Bind a real server on a free port and serve in the background.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Connect a real WebSocket client and play a game.
        let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/ws"))
            .await
            .unwrap();

        socket.send(ClientMessage::text("3")).await.unwrap();
        socket.send(ClientMessage::text("Ruth")).await.unwrap();
        socket.close(None).await.unwrap();

        // Give the server a moment to process both messages.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert_eq!(*game.started_with.lock().unwrap(), Some(3));
        assert_eq!(
            *game.finished_with.lock().unwrap(),
            Some("Ruth".to_string())
        );
    }
    // ANCHOR_END: test
}
