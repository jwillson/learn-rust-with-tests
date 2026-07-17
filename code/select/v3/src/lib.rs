use std::fmt;
use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc;
use std::time::Duration;

// ANCHOR: error
#[derive(Debug, PartialEq)]
pub struct RacerError {
    a: String,
    b: String,
}

impl fmt::Display for RacerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "timed out waiting for {} and {}", self.a, self.b)
    }
}

impl std::error::Error for RacerError {}
// ANCHOR_END: error

// ANCHOR: code
const TEN_SECOND_TIMEOUT: Duration = Duration::from_secs(10);

pub fn racer<'a>(a: &'a str, b: &'a str) -> Result<&'a str, RacerError> {
    configurable_racer(a, b, TEN_SECOND_TIMEOUT)
}

pub fn configurable_racer<'a>(
    a: &'a str,
    b: &'a str,
    timeout: Duration,
) -> Result<&'a str, RacerError> {
    let (sender, receiver) = mpsc::channel();

    ping(a, sender.clone());
    ping(b, sender);

    match receiver.recv_timeout(timeout) {
        Ok(winner) => {
            if winner == a {
                Ok(a)
            } else {
                Ok(b)
            }
        }
        Err(_) => Err(RacerError {
            a: a.to_string(),
            b: b.to_string(),
        }),
    }
}
// ANCHOR_END: code

fn ping(addr: &str, sender: mpsc::Sender<String>) {
    let addr = addr.to_string();

    std::thread::spawn(move || {
        if let Ok(mut stream) = TcpStream::connect(&addr) {
            let mut response = String::new();
            let _ = stream.read_to_string(&mut response);
        }
        let _ = sender.send(addr);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;

    fn make_delayed_server(delay: Duration) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                std::thread::sleep(delay);
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n");
            }
        });

        addr
    }

    // ANCHOR: test
    #[test]
    fn returns_the_faster_of_two_servers() {
        let slow_addr = make_delayed_server(Duration::from_millis(20));
        let fast_addr = make_delayed_server(Duration::ZERO);

        let want = fast_addr.as_str();
        let got = racer(&slow_addr, &fast_addr).expect("did not expect an error but got one");

        assert_eq!(got, want);
    }

    #[test]
    fn returns_an_error_if_a_server_does_not_respond_in_time() {
        let addr = make_delayed_server(Duration::from_millis(25));

        let result = configurable_racer(&addr, &addr, Duration::from_millis(20));

        assert!(result.is_err(), "expected an error but didn't get one");
    }
    // ANCHOR_END: test
}
