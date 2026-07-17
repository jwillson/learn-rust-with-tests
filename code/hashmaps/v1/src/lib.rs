use std::collections::HashMap;

// ANCHOR: code
#[derive(Default)]
pub struct Dictionary(HashMap<String, String>);

impl Dictionary {
    pub fn search(&self, word: &str) -> Option<&str> {
        self.0.get(word).map(|definition| definition.as_str())
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    fn dictionary_with(word: &str, definition: &str) -> Dictionary {
        Dictionary(HashMap::from([(word.to_string(), definition.to_string())]))
    }

    #[test]
    fn search_finds_a_known_word() {
        let dictionary = dictionary_with("test", "this is just a test");

        let got = dictionary.search("test");
        let want = Some("this is just a test");

        assert_eq!(got, want);
    }

    #[test]
    fn search_returns_nothing_for_an_unknown_word() {
        let dictionary = dictionary_with("test", "this is just a test");

        let got = dictionary.search("unknown");

        assert_eq!(got, None);
    }
    // ANCHOR_END: test
}
