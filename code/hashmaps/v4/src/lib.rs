use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::fmt;

// ANCHOR: error
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryError {
    WordExists,
    WordDoesNotExist,
}

impl fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DictionaryError::WordExists => {
                write!(f, "cannot add word because it already exists")
            }
            DictionaryError::WordDoesNotExist => {
                write!(
                    f,
                    "cannot perform operation on word because it does not exist"
                )
            }
        }
    }
}

impl Error for DictionaryError {}
// ANCHOR_END: error

#[derive(Default)]
pub struct Dictionary(HashMap<String, String>);

impl Dictionary {
    pub fn search(&self, word: &str) -> Option<&str> {
        self.0.get(word).map(|definition| definition.as_str())
    }

    pub fn add(&mut self, word: &str, definition: &str) -> Result<(), DictionaryError> {
        match self.0.entry(word.to_string()) {
            Entry::Occupied(_) => Err(DictionaryError::WordExists),
            Entry::Vacant(entry) => {
                entry.insert(definition.to_string());
                Ok(())
            }
        }
    }

    pub fn update(&mut self, word: &str, definition: &str) -> Result<(), DictionaryError> {
        match self.0.get_mut(word) {
            Some(existing) => {
                *existing = definition.to_string();
                Ok(())
            }
            None => Err(DictionaryError::WordDoesNotExist),
        }
    }

    // ANCHOR: delete
    pub fn delete(&mut self, word: &str) -> Result<(), DictionaryError> {
        match self.0.remove(word) {
            Some(_) => Ok(()),
            None => Err(DictionaryError::WordDoesNotExist),
        }
    }
    // ANCHOR_END: delete
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

    // ANCHOR: delete_test
    #[test]
    fn deletes_a_word() {
        let mut dictionary = dictionary_with("test", "test definition");

        let result = dictionary.delete("test");

        assert_eq!(result, Ok(()));
        assert_eq!(dictionary.search("test"), None);
    }

    #[test]
    fn refuses_to_delete_an_unknown_word() {
        let mut dictionary = Dictionary::default();

        let result = dictionary.delete("test");

        assert_eq!(result, Err(DictionaryError::WordDoesNotExist));
    }
    // ANCHOR_END: delete_test
}
