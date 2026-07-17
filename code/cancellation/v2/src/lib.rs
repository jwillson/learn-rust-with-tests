use std::fmt;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// ANCHOR: token
#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> CancellationToken {
        CancellationToken::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}
// ANCHOR_END: token

// ANCHOR: error
#[derive(Debug, PartialEq)]
pub struct Cancelled;

impl fmt::Display for Cancelled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the work was cancelled")
    }
}

impl std::error::Error for Cancelled {}
// ANCHOR_END: error

// ANCHOR: code
pub trait Store {
    fn fetch(&self, cancel: &CancellationToken) -> Result<String, Cancelled>;
}

pub fn respond(
    writer: &mut impl Write,
    store: &impl Store,
    cancel: &CancellationToken,
) -> std::io::Result<()> {
    match store.fetch(cancel) {
        Ok(data) => write!(writer, "{data}"),
        Err(Cancelled) => Ok(()),
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // ANCHOR: token_tests
    #[test]
    fn a_new_token_is_not_cancelled() {
        let token = CancellationToken::new();

        assert!(!token.is_cancelled());
    }

    #[test]
    fn cancelling_flips_the_token() {
        let token = CancellationToken::new();

        token.cancel();

        assert!(token.is_cancelled());
    }

    #[test]
    fn clones_share_the_same_signal() {
        let token = CancellationToken::new();
        let clone = token.clone();

        clone.cancel();

        assert!(token.is_cancelled());
    }

    #[test]
    fn the_signal_crosses_threads() {
        let token = CancellationToken::new();
        let clone = token.clone();

        thread::spawn(move || clone.cancel()).join().unwrap();

        assert!(token.is_cancelled());
    }
    // ANCHOR_END: token_tests

    // ANCHOR: spy
    struct SpyStore {
        response: String,
    }

    impl Store for SpyStore {
        fn fetch(&self, cancel: &CancellationToken) -> Result<String, Cancelled> {
            let mut result = String::new();
            for c in self.response.chars() {
                if cancel.is_cancelled() {
                    return Err(Cancelled);
                }
                thread::sleep(Duration::from_millis(10));
                result.push(c);
            }
            Ok(result)
        }
    }
    // ANCHOR_END: spy

    // ANCHOR: test
    #[test]
    fn returns_data_from_the_store() {
        let store = SpyStore {
            response: "hello, world".to_string(),
        };
        let cancel = CancellationToken::new();
        let mut response = Vec::new();

        respond(&mut response, &store, &cancel).unwrap();

        assert_eq!(String::from_utf8(response).unwrap(), "hello, world");
    }

    #[test]
    fn writes_no_response_when_the_request_is_cancelled() {
        let store = SpyStore {
            response: "hello, world".to_string(),
        };
        let cancel = CancellationToken::new();
        let mut response = Vec::new();

        let canceller = cancel.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(5));
            canceller.cancel();
        });

        respond(&mut response, &store, &cancel).unwrap();

        assert!(
            response.is_empty(),
            "a response should not have been written"
        );
    }
    // ANCHOR_END: test
}
