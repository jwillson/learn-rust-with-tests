// ANCHOR: code
pub fn repeat(character: &str) -> String {
    let mut repeated = String::new();
    for _ in 0..5 {
        repeated.push_str(character);
    }
    repeated
}
// ANCHOR_END: code

// ANCHOR: test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeats_the_character() {
        let repeated = repeat("a");
        let expected = "aaaaa";

        assert_eq!(repeated, expected);
    }
}
// ANCHOR_END: test
