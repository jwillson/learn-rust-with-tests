// ANCHOR: first_sentence
pub fn first_sentence(text: &str) -> &str {
    match text.find('.') {
        Some(position) => &text[..position],
        None => text,
    }
}
// ANCHOR_END: first_sentence

// ANCHOR: longer
pub fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
    if b.len() > a.len() { b } else { a }
}
// ANCHOR_END: longer

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: first_sentence_tests
    #[test]
    fn returns_everything_up_to_the_first_full_stop() {
        let article = "Ownership is the star of the show. Everything else orbits it.";

        let got = first_sentence(article);

        assert_eq!(got, "Ownership is the star of the show");
    }

    #[test]
    fn returns_the_whole_text_when_there_is_no_full_stop() {
        let got = first_sentence("no full stop here");

        assert_eq!(got, "no full stop here");
    }
    // ANCHOR_END: first_sentence_tests

    // ANCHOR: longer_test
    #[test]
    fn returns_the_longer_of_two_excerpts() {
        let got = longer("short", "a good deal longer");

        assert_eq!(got, "a good deal longer");
    }
    // ANCHOR_END: longer_test
}
