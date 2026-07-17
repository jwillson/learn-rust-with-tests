use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;

// ANCHOR: store
pub trait PlayerStore: Send + Sync {
    fn score(&self, name: &str) -> Option<i32>;
}
// ANCHOR_END: store

// ANCHOR: code
pub fn player_server(store: Arc<dyn PlayerStore>) -> Router {
    Router::new()
        .route("/players/{name}", get(get_score))
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
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::collections::HashMap;
    use tower::ServiceExt;

    // ANCHOR: stub
    struct StubPlayerStore {
        scores: HashMap<String, i32>,
    }

    impl PlayerStore for StubPlayerStore {
        fn score(&self, name: &str) -> Option<i32> {
            self.scores.get(name).copied()
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
        let body = String::from_utf8(bytes.to_vec()).unwrap();

        (status, body)
    }

    fn stub_server() -> Router {
        let store = StubPlayerStore {
            scores: HashMap::from([("Pepper".to_string(), 20), ("Floyd".to_string(), 10)]),
        };
        player_server(Arc::new(store))
    }

    // ANCHOR: test
    #[tokio::test]
    async fn returns_peppers_score() {
        let (status, body) = get_score_request(stub_server(), "Pepper").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "20");
    }

    #[tokio::test]
    async fn returns_floyds_score() {
        let (status, body) = get_score_request(stub_server(), "Floyd").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "10");
    }

    #[tokio::test]
    async fn returns_404_on_missing_players() {
        let (status, _body) = get_score_request(stub_server(), "Apollo").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }
    // ANCHOR_END: test
}
