use std::collections::HashMap;

// ANCHOR: code
/// Count how often each whitespace-separated word appears across all documents.
pub fn word_counts(documents: &[String]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for document in documents {
        for word in document.split_whitespace() {
            *counts.entry(word.to_string()).or_insert(0) += 1;
        }
    }

    counts
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn counts_words_across_documents() {
        let documents = vec![
            "the cat sat".to_string(),
            "the dog sat".to_string(),
            "the the".to_string(),
        ];

        let got = word_counts(&documents);

        assert_eq!(got["the"], 4);
        assert_eq!(got["sat"], 2);
        assert_eq!(got["cat"], 1);
        assert_eq!(got["dog"], 1);
    }

    #[test]
    fn an_empty_corpus_has_no_counts() {
        assert!(word_counts(&[]).is_empty());
    }
    // ANCHOR_END: test
}
