use std::io::Read;
use std::net::TcpStream;
use std::time::{Duration, Instant};

// ANCHOR: code
pub fn racer<'a>(a: &'a str, b: &'a str) -> &'a str {
    let a_duration = measure_response_time(a);
    let b_duration = measure_response_time(b);

    if a_duration < b_duration { a } else { b }
}

fn measure_response_time(addr: &str) -> Duration {
    let start = Instant::now();

    if let Ok(mut stream) = TcpStream::connect(addr) {
        let mut response = String::new();
        let _ = stream.read_to_string(&mut response);
    }

    start.elapsed()
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;

    // ANCHOR: make_delayed_server
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
    // ANCHOR_END: make_delayed_server

    // ANCHOR: test
    #[test]
    fn returns_the_faster_of_two_servers() {
        let slow_addr = make_delayed_server(Duration::from_millis(20));
        let fast_addr = make_delayed_server(Duration::ZERO);

        let want = fast_addr.as_str();
        let got = racer(&slow_addr, &fast_addr);

        assert_eq!(got, want);
    }
    // ANCHOR_END: test
}
