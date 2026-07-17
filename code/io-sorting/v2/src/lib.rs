use std::cmp::Reverse;
use std::io::{Read, Seek, SeekFrom, Write};
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

// ANCHOR: trait
pub trait PlayerStore: Send + Sync {
    fn score(&self, name: &str) -> Option<i32>;
    fn record_win(&self, name: &str);
    fn league(&self) -> Vec<Player>;
}
// ANCHOR_END: trait

pub trait Truncate {
    fn truncate(&mut self, len: u64) -> std::io::Result<()>;
}

impl Truncate for std::fs::File {
    fn truncate(&mut self, len: u64) -> std::io::Result<()> {
        self.set_len(len)
    }
}

impl Truncate for std::io::Cursor<Vec<u8>> {
    fn truncate(&mut self, len: u64) -> std::io::Result<()> {
        self.get_mut().truncate(len as usize);
        Ok(())
    }
}

struct Inner<F> {
    database: F,
    league: Vec<Player>,
}

pub struct FileSystemPlayerStore<F> {
    inner: Mutex<Inner<F>>,
}

impl<F: Read + Write + Seek + Truncate> FileSystemPlayerStore<F> {
    pub fn new(mut database: F) -> std::io::Result<FileSystemPlayerStore<F>> {
        database.seek(SeekFrom::Start(0))?;
        let league = read_league(&mut database)?;

        Ok(FileSystemPlayerStore {
            inner: Mutex::new(Inner { database, league }),
        })
    }
}

// ANCHOR: impl
impl<F: Read + Write + Seek + Truncate + Send> PlayerStore for FileSystemPlayerStore<F> {
    fn score(&self, name: &str) -> Option<i32> {
        self.inner
            .lock()
            .unwrap()
            .league
            .iter()
            .find(|player| player.name == name)
            .map(|player| player.wins)
    }

    fn record_win(&self, name: &str) {
        let mut inner = self.inner.lock().unwrap();

        match inner.league.iter_mut().find(|player| player.name == name) {
            Some(player) => player.wins += 1,
            None => inner.league.push(Player {
                name: name.to_string(),
                wins: 1,
            }),
        }

        let bytes = serde_json::to_vec(&inner.league).expect("a league always serializes");
        inner.database.seek(SeekFrom::Start(0)).unwrap();
        inner.database.write_all(&bytes).unwrap();
        inner.database.truncate(bytes.len() as u64).unwrap();
    }

    fn league(&self) -> Vec<Player> {
        let mut league = self.inner.lock().unwrap().league.clone();
        league.sort_by_key(|player| Reverse(player.wins));
        league
    }
}
// ANCHOR_END: impl

fn read_league(database: &mut impl Read) -> std::io::Result<Vec<Player>> {
    let mut contents = String::new();
    database.read_to_string(&mut contents)?;

    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&contents)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

// ANCHOR: server
pub fn player_server(store: Arc<dyn PlayerStore>) -> Router {
    Router::new()
        .route("/league", get(get_league))
        .route("/players/{name}", get(get_score).post(record_win))
        .with_state(store)
}

async fn get_league(State(store): State<Arc<dyn PlayerStore>>) -> Response {
    axum::Json(store.league()).into_response()
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
// ANCHOR_END: server

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::io::Cursor;
    use tower::ServiceExt;

    fn file_store(json: &str) -> Arc<dyn PlayerStore> {
        let database = Cursor::new(json.as_bytes().to_vec());
        Arc::new(FileSystemPlayerStore::new(database).unwrap())
    }

    // ANCHOR: integration_test
    #[tokio::test]
    async fn recording_wins_and_retrieving_them_via_http() {
        let store = file_store("[]");
        let server = player_server(store);

        // record three wins for Pepper
        for _ in 0..3 {
            let request = Request::post("/players/Pepper")
                .body(Body::empty())
                .unwrap();
            let status = server.clone().oneshot(request).await.unwrap().status();
            assert_eq!(status, StatusCode::ACCEPTED);
        }

        // read the score back
        let request = Request::get("/players/Pepper").body(Body::empty()).unwrap();
        let response = server.oneshot(request).await.unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();

        assert_eq!(String::from_utf8(bytes.to_vec()).unwrap(), "3");
    }
    // ANCHOR_END: integration_test
}
