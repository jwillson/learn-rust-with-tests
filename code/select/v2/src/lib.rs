use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc;

// ANCHOR: code
pub fn racer<'a>(a: &'a str, b: &'a str) -> &'a str {
    let (sender, receiver) = mpsc::channel();

    ping(a, sender.clone());
    ping(b, sender);

    let winner = receiver.recv().unwrap();
    if winner == a { a } else { b }
}

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
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;
    use std::time::Duration;

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

    #[test]
    fn returns_the_faster_of_two_servers() {
        let slow_addr = make_delayed_server(Duration::from_millis(20));
        let fast_addr = make_delayed_server(Duration::ZERO);

        let want = fast_addr.as_str();
        let got = racer(&slow_addr, &fast_addr);

        assert_eq!(got, want);
    }
}
