use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;

// ANCHOR: store
pub trait PlayerStore: Send + Sync {
    fn score(&self, name: &str) -> Option<i32>;
    fn record_win(&self, name: &str);
}
// ANCHOR_END: store

// ANCHOR: in_memory
#[derive(Default)]
pub struct InMemoryPlayerStore {
    scores: Mutex<HashMap<String, i32>>,
}

impl PlayerStore for InMemoryPlayerStore {
    fn score(&self, name: &str) -> Option<i32> {
        self.scores.lock().unwrap().get(name).copied()
    }

    fn record_win(&self, name: &str) {
        *self
            .scores
            .lock()
            .unwrap()
            .entry(name.to_string())
            .or_insert(0) += 1;
    }
}
// ANCHOR_END: in_memory

// ANCHOR: code
pub fn player_server(store: Arc<dyn PlayerStore>) -> Router {
    Router::new()
        .route("/players/{name}", get(get_score).post(record_win))
        .with_state(store)
}

async fn get_score(
    State(store): State<Arc<dyn PlayerStore>>,
    Path(name): Path<String>,
) -> Response {
    match store.score(&name) {
        Some(score) => score.to_string().into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn record_win(
    State(store): State<Arc<dyn PlayerStore>>,
    Path(name): Path<String>,
) -> Response {
    store.record_win(&name);
    StatusCode::ACCEPTED.into_response()
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tower::ServiceExt;

    // ANCHOR: stub
    #[derive(Default)]
    struct StubPlayerStore {
        scores: HashMap<String, i32>,
        win_calls: Mutex<Vec<String>>,
    }

    impl PlayerStore for StubPlayerStore {
        fn score(&self, name: &str) -> Option<i32> {
            self.scores.get(name).copied()
        }

        fn record_win(&self, name: &str) {
            self.win_calls.lock().unwrap().push(name.to_string());
        }
    }
    // ANCHOR_END: stub

    async fn get_score_request(server: Router, name: &str) -> (StatusCode, String) {
        let request = Request::get(format!("/players/{name}"))
            .body(Body::empty())
            .unwrap();
        let response = server.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    async fn post_win_request(server: Router, name: &str) -> StatusCode {
        let request = Request::post(format!("/players/{name}"))
            .body(Body::empty())
            .unwrap();
        server.oneshot(request).await.unwrap().status()
    }

    // ANCHOR: get_test
    #[tokio::test]
    async fn returns_a_players_score() {
        let store = Arc::new(StubPlayerStore {
            scores: HashMap::from([("Pepper".to_string(), 20)]),
            ..Default::default()
        });

        let (status, body) = get_score_request(player_server(store), "Pepper").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "20");
    }

    #[tokio::test]
    async fn returns_404_on_missing_players() {
        let store = Arc::new(StubPlayerStore::default());

        let (status, _) = get_score_request(player_server(store), "Apollo").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }
    // ANCHOR_END: get_test

    // ANCHOR: post_test
    #[tokio::test]
    async fn records_a_win_when_a_post_is_received() {
        let store = Arc::new(StubPlayerStore::default());

        let status = post_win_request(player_server(store.clone()), "Pepper").await;

        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(*store.win_calls.lock().unwrap(), vec!["Pepper".to_string()]);
    }
    // ANCHOR_END: post_test
}
