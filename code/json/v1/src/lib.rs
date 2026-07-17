use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::{Deserialize, Serialize};

// ANCHOR: player
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub wins: i32,
}
// ANCHOR_END: player

pub trait PlayerStore: Send + Sync {
    fn score(&self, name: &str) -> Option<i32>;
    fn record_win(&self, name: &str);
}

// ANCHOR: code
pub fn player_server(store: Arc<dyn PlayerStore>) -> Router {
    Router::new()
        .route("/league", get(get_league))
        .route("/players/{name}", get(get_score).post(record_win))
        .with_state(store)
}

async fn get_league() -> Response {
    let league = vec![Player {
        name: "Chris".to_string(),
        wins: 20,
    }];

    axum::Json(league).into_response()
}
// ANCHOR_END: code

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::collections::HashMap;
    use tower::ServiceExt;

    #[derive(Default)]
    struct StubPlayerStore {
        scores: HashMap<String, i32>,
    }

    impl PlayerStore for StubPlayerStore {
        fn score(&self, name: &str) -> Option<i32> {
            self.scores.get(name).copied()
        }

        fn record_win(&self, _name: &str) {}
    }

    // ANCHOR: test
    #[tokio::test]
    async fn returns_the_league_table_as_json() {
        let server = player_server(Arc::new(StubPlayerStore::default()));

        let request = Request::get("/league").body(Body::empty()).unwrap();
        let response = server.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let got: Vec<Player> = serde_json::from_slice(&bytes)
            .expect("response body should parse as a list of players");

        assert_eq!(
            got,
            vec![Player {
                name: "Chris".to_string(),
                wins: 20
            }]
        );
    }
    // ANCHOR_END: test
}
