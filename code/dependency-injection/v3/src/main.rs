use std::io::Write;
use std::net::TcpListener;

// ANCHOR: code
pub fn greet(writer: &mut impl Write, name: &str) -> std::io::Result<()> {
    write!(writer, "Hello, {name}")
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5001")?;

    for stream in listener.incoming() {
        let mut stream = stream?;
        greet(&mut stream, "world")?;
    }

    Ok(())
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_writes_the_greeting() {
        let mut buffer = Vec::new();

        greet(&mut buffer, "Chris").unwrap();

        let got = String::from_utf8(buffer).unwrap();
        let want = "Hello, Chris";

        assert_eq!(got, want);
    }
}
