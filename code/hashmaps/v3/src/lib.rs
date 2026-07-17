use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryError {
    WordExists,
    WordDoesNotExist,
}

#[derive(Default)]
pub struct Dictionary(HashMap<String, String>);

impl Dictionary {
    pub fn search(&self, word: &str) -> Option<&str> {
        self.0.get(word).map(|definition| definition.as_str())
    }

    // ANCHOR: add
    pub fn add(&mut self, word: &str, definition: &str) -> Result<(), DictionaryError> {
        match self.0.entry(word.to_string()) {
            Entry::Occupied(_) => Err(DictionaryError::WordExists),
            Entry::Vacant(entry) => {
                entry.insert(definition.to_string());
                Ok(())
            }
        }
    }
    // ANCHOR_END: add

    // ANCHOR: update
    pub fn update(&mut self, word: &str, definition: &str) -> Result<(), DictionaryError> {
        match self.0.get_mut(word) {
            Some(existing) => {
                *existing = definition.to_string();
                Ok(())
            }
            None => Err(DictionaryError::WordDoesNotExist),
        }
    }
    // ANCHOR_END: update
}

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

    // ANCHOR: update_test
    #[test]
    fn updates_the_definition_of_an_existing_word() {
        let mut dictionary = dictionary_with("test", "this is just a test");

        let result = dictionary.update("test", "new definition");

        assert_eq!(result, Ok(()));
        assert_definition(&dictionary, "test", "new definition");
    }

    #[test]
    fn refuses_to_update_an_unknown_word() {
        let mut dictionary = Dictionary::default();

        let result = dictionary.update("test", "this is just a test");

        assert_eq!(result, Err(DictionaryError::WordDoesNotExist));
    }
    // ANCHOR_END: update_test
}
