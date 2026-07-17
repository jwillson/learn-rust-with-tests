use std::io::Write;

// ANCHOR: code
const FINAL_WORD: &str = "Go!";
const COUNTDOWN_START: u32 = 3;

fn countdown(out: &mut impl Write) -> std::io::Result<()> {
    for i in (1..=COUNTDOWN_START).rev() {
        writeln!(out, "{i}")?;
    }

    write!(out, "{FINAL_WORD}")
}

fn main() -> std::io::Result<()> {
    countdown(&mut std::io::stdout())
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn counts_down_from_three_to_go() {
        let mut buffer = Vec::new();

        countdown(&mut buffer).unwrap();

        let got = String::from_utf8(buffer).unwrap();
        let want = "3\n2\n1\nGo!";

        assert_eq!(got, want);
    }
    // ANCHOR_END: test
}
