use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub wins: i32,
}

// ANCHOR: store
pub trait PlayerStore: Send + Sync {
    fn score(&self, name: &str) -> Option<i32>;
    fn record_win(&self, name: &str);
    fn league(&self) -> Vec<Player>;
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

    fn league(&self) -> Vec<Player> {
        let mut league: Vec<Player> = self
            .scores
            .lock()
            .unwrap()
            .iter()
            .map(|(name, &wins)| Player {
                name: name.clone(),
                wins,
            })
            .collect();
        league.sort_by_key(|player| std::cmp::Reverse(player.wins));
        league
    }
}
// ANCHOR_END: in_memory

// ANCHOR: code
pub fn player_server(store: Arc<dyn PlayerStore>) -> Router {
    Router::new()
        .route("/league", get(get_league))
        .route("/players/{name}", get(get_score).post(record_win))
        .with_state(store)
}

async fn get_league(State(store): State<Arc<dyn PlayerStore>>) -> Response {
    axum::Json(store.league()).into_response()
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
    use tower::ServiceExt;

    #[derive(Default)]
    struct StubPlayerStore {
        scores: HashMap<String, i32>,
        league: Vec<Player>,
    }

    impl PlayerStore for StubPlayerStore {
        fn score(&self, name: &str) -> Option<i32> {
            self.scores.get(name).copied()
        }

        fn record_win(&self, _name: &str) {}

        fn league(&self) -> Vec<Player> {
            self.league.clone()
        }
    }

    async fn get_league(server: Router) -> (StatusCode, Vec<Player>) {
        let request = Request::get("/league").body(Body::empty()).unwrap();
        let response = server.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let league = serde_json::from_slice(&bytes)
            .expect("response body should parse as a list of players");
        (status, league)
    }

    // ANCHOR: test
    #[tokio::test]
    async fn returns_the_league_table_from_the_store() {
        let wanted_league = vec![
            Player {
                name: "Cleo".to_string(),
                wins: 32,
            },
            Player {
                name: "Chris".to_string(),
                wins: 20,
            },
            Player {
                name: "Tiest".to_string(),
                wins: 14,
            },
        ];
        let store = StubPlayerStore {
            scores: HashMap::new(),
            league: wanted_league.clone(),
        };

        let (status, got) = get_league(player_server(Arc::new(store))).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(got, wanted_league);
    }
    // ANCHOR_END: test

    // ANCHOR: in_memory_test
    #[test]
    fn in_memory_store_builds_a_sorted_league() {
        let store = InMemoryPlayerStore::default();
        store.record_win("Chris");
        store.record_win("Cleo");
        store.record_win("Cleo");

        let league = store.league();

        assert_eq!(
            league,
            vec![
                Player {
                    name: "Cleo".to_string(),
                    wins: 2
                },
                Player {
                    name: "Chris".to_string(),
                    wins: 1
                },
            ]
        );
    }
    // ANCHOR_END: in_memory_test
}
