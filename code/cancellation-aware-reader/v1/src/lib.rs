use std::io::Read;
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

// ANCHOR: reader
pub struct CancellableReader<R> {
    reader: R,
    cancel: CancellationToken,
}

impl<R: Read> CancellableReader<R> {
    pub fn new(reader: R, cancel: CancellationToken) -> CancellableReader<R> {
        CancellableReader { reader, cancel }
    }
}

impl<R: Read> Read for CancellableReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.cancel.is_cancelled() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "read cancelled",
            ));
        }

        self.reader.read(buf)
    }
}
// ANCHOR_END: reader

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: normal_test
    #[test]
    fn behaves_like_a_normal_reader_when_not_cancelled() {
        let cancel = CancellationToken::new();
        let mut reader = CancellableReader::new("123456".as_bytes(), cancel);

        let mut got = String::new();
        reader.read_to_string(&mut got).unwrap();

        assert_eq!(got, "123456");
    }
    // ANCHOR_END: normal_test

    // ANCHOR: cancel_test
    #[test]
    fn stops_reading_once_cancelled() {
        let cancel = CancellationToken::new();
        let mut reader = CancellableReader::new("123456".as_bytes(), cancel.clone());

        // Read a few bytes normally.
        let mut buffer = [0u8; 3];
        let n = reader.read(&mut buffer).unwrap();
        assert_eq!(&buffer[..n], b"123");

        // Now cancel, and the next read fails instead of returning more data.
        cancel.cancel();
        let result = reader.read(&mut buffer);

        assert!(result.is_err(), "expected a cancelled read");
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Interrupted);
    }
    // ANCHOR_END: cancel_test
}
