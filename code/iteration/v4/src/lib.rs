// ANCHOR: code
const REPEAT_COUNT: usize = 5;

pub fn repeat(character: &str) -> String {
    character.repeat(REPEAT_COUNT)
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
