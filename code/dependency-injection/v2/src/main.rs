use std::io::Write;

// ANCHOR: code
pub fn greet(writer: &mut impl Write, name: &str) -> std::io::Result<()> {
    write!(writer, "Hello, {name}")
}

fn main() -> std::io::Result<()> {
    greet(&mut std::io::stdout(), "Elodie")
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
