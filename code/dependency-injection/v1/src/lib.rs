use std::io::Write;

// ANCHOR: code
pub fn greet(writer: &mut Vec<u8>, name: &str) -> std::io::Result<()> {
    write!(writer, "Hello, {name}")
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn greet_writes_the_greeting() {
        let mut buffer = Vec::new();

        greet(&mut buffer, "Chris").unwrap();

        let got = String::from_utf8(buffer).unwrap();
        let want = "Hello, Chris";

        assert_eq!(got, want);
    }
    // ANCHOR_END: test
}
