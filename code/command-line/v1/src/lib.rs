use std::cmp::Reverse;
use std::io::{BufRead, Read, Seek, SeekFrom, Write};
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

pub trait PlayerStore: Send + Sync {
    fn score(&self, name: &str) -> Option<i32>;
    fn record_win(&self, name: &str);
    fn league(&self) -> Vec<Player>;
}

// ANCHOR: cli
pub struct Cli<R> {
    store: Arc<dyn PlayerStore>,
    input: R,
}

impl<R: BufRead> Cli<R> {
    pub fn new(store: Arc<dyn PlayerStore>, input: R) -> Cli<R> {
        Cli { store, input }
    }

    pub fn play_poker(&mut self) {
        let mut line = String::new();
        if self.input.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }

        if let Some(name) = line.trim().strip_suffix(" wins") {
            self.store.record_win(name);
        }
    }
}
// ANCHOR_END: cli

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

fn read_league(database: &mut impl Read) -> std::io::Result<Vec<Player>> {
    let mut contents = String::new();
    database.read_to_string(&mut contents)?;

    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&contents)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: spy
    #[derive(Default)]
    struct StubPlayerStore {
        win_calls: Mutex<Vec<String>>,
    }

    impl PlayerStore for StubPlayerStore {
        fn score(&self, _name: &str) -> Option<i32> {
            None
        }

        fn record_win(&self, name: &str) {
            self.win_calls.lock().unwrap().push(name.to_string());
        }

        fn league(&self) -> Vec<Player> {
            Vec::new()
        }
    }

    #[track_caller]
    fn assert_player_win(store: &StubPlayerStore, winner: &str) {
        let calls = store.win_calls.lock().unwrap();
        assert_eq!(calls.len(), 1, "expected exactly one win call");
        assert_eq!(calls[0], winner);
    }
    // ANCHOR_END: spy

    // ANCHOR: test
    #[test]
    fn records_chris_win_from_user_input() {
        let store = Arc::new(StubPlayerStore::default());
        let input = "Chris wins\n".as_bytes();

        let mut cli = Cli::new(store.clone(), input);
        cli.play_poker();

        assert_player_win(&store, "Chris");
    }

    #[test]
    fn records_cleo_win_from_user_input() {
        let store = Arc::new(StubPlayerStore::default());
        let input = "Cleo wins\n".as_bytes();

        let mut cli = Cli::new(store.clone(), input);
        cli.play_poker();

        assert_player_win(&store, "Cleo");
    }
    // ANCHOR_END: test
}
