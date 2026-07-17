// ANCHOR: code
const REPEAT_COUNT: usize = 5;

pub fn repeat(character: &str) -> String {
    let mut repeated = String::new();
    for _ in 0..REPEAT_COUNT {
        repeated += character;
    }
    repeated
}
// ANCHOR_END: code

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
