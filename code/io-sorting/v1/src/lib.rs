use std::cmp::Reverse;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub wins: i32,
}

// ANCHOR: truncate
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
// ANCHOR_END: truncate

// ANCHOR: store
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

    pub fn league(&self) -> Vec<Player> {
        let mut league = self.inner.lock().unwrap().league.clone();
        league.sort_by_key(|player| Reverse(player.wins));
        league
    }

    pub fn score(&self, name: &str) -> Option<i32> {
        self.inner
            .lock()
            .unwrap()
            .league
            .iter()
            .find(|player| player.name == name)
            .map(|player| player.wins)
    }

    pub fn record_win(&self, name: &str) {
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
// ANCHOR_END: store

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // ANCHOR: helper
    fn store_from(json: &str) -> FileSystemPlayerStore<Cursor<Vec<u8>>> {
        let database = Cursor::new(json.as_bytes().to_vec());
        FileSystemPlayerStore::new(database).unwrap()
    }
    // ANCHOR_END: helper

    // ANCHOR: test
    #[test]
    fn reads_a_league_sorted_by_wins() {
        let store = store_from(r#"[{"name": "Cleo", "wins": 10}, {"name": "Chris", "wins": 33}]"#);

        let got = store.league();

        assert_eq!(
            got,
            vec![
                Player {
                    name: "Chris".to_string(),
                    wins: 33
                },
                Player {
                    name: "Cleo".to_string(),
                    wins: 10
                },
            ]
        );
    }

    #[test]
    fn reads_a_players_score() {
        let store = store_from(r#"[{"name": "Cleo", "wins": 10}, {"name": "Chris", "wins": 33}]"#);

        assert_eq!(store.score("Chris"), Some(33));
        assert_eq!(store.score("Nobody"), None);
    }

    #[test]
    fn records_a_win_for_an_existing_player() {
        let store = store_from(r#"[{"name": "Cleo", "wins": 10}, {"name": "Chris", "wins": 33}]"#);

        store.record_win("Chris");

        assert_eq!(store.score("Chris"), Some(34));
    }

    #[test]
    fn records_a_win_for_a_new_player() {
        let store = store_from(r#"[{"name": "Cleo", "wins": 10}]"#);

        store.record_win("Pepper");

        assert_eq!(store.score("Pepper"), Some(1));
    }

    #[test]
    fn starts_from_an_empty_file() {
        let store = FileSystemPlayerStore::new(Cursor::new(Vec::new())).unwrap();

        store.record_win("Pepper");

        assert_eq!(store.score("Pepper"), Some(1));
    }
    // ANCHOR_END: test
}
