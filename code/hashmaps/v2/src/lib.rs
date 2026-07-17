use std::collections::HashMap;

// ANCHOR: error
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryError {
    WordExists,
}
// ANCHOR_END: error

#[derive(Default)]
pub struct Dictionary(HashMap<String, String>);

// ANCHOR: code
impl Dictionary {
    pub fn search(&self, word: &str) -> Option<&str> {
        self.0.get(word).map(|definition| definition.as_str())
    }

    pub fn add(&mut self, word: &str, definition: &str) -> Result<(), DictionaryError> {
        if self.0.contains_key(word) {
            return Err(DictionaryError::WordExists);
        }

        self.0.insert(word.to_string(), definition.to_string());
        Ok(())
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    fn dictionary_with(word: &str, definition: &str) -> Dictionary {
        Dictionary(HashMap::from([(word.to_string(), definition.to_string())]))
    }

    #[track_caller]
    fn assert_definition(dictionary: &Dictionary, word: &str, definition: &str) {
        let got = dictionary.search(word);

        assert_eq!(got, Some(definition), "should find added word");
    }

    #[test]
    fn search_finds_a_known_word() {
        let dictionary = dictionary_with("test", "this is just a test");

        assert_definition(&dictionary, "test", "this is just a test");
    }

    #[test]
    fn search_returns_nothing_for_an_unknown_word() {
        let dictionary = dictionary_with("test", "this is just a test");

        assert_eq!(dictionary.search("unknown"), None);
    }

    // ANCHOR: test
    #[test]
    fn adds_a_new_word() {
        let mut dictionary = Dictionary::default();

        let result = dictionary.add("test", "this is just a test");

        assert_eq!(result, Ok(()));
        assert_definition(&dictionary, "test", "this is just a test");
    }

    #[test]
    fn refuses_to_overwrite_an_existing_word() {
        let mut dictionary = dictionary_with("test", "this is just a test");

        let result = dictionary.add("test", "new test");

        assert_eq!(result, Err(DictionaryError::WordExists));
        assert_definition(&dictionary, "test", "this is just a test");
    }
    // ANCHOR_END: test
}
