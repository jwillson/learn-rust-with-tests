use std::collections::HashMap;

// ANCHOR: sequential
pub fn word_counts(documents: &[String]) -> HashMap<String, usize> {
    count_chunk(documents)
}

fn count_chunk(documents: &[String]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for document in documents {
        for word in document.split_whitespace() {
            *counts.entry(word.to_string()).or_insert(0) += 1;
        }
    }
    counts
}
// ANCHOR_END: sequential

// ANCHOR: parallel
pub fn word_counts_parallel(documents: &[String], threads: usize) -> HashMap<String, usize> {
    if documents.is_empty() {
        return HashMap::new();
    }

    let chunk_size = documents.len().div_ceil(threads.max(1)).max(1);

    // Map: each thread counts its own chunk into a private map — no shared state.
    let partials: Vec<HashMap<String, usize>> = std::thread::scope(|scope| {
        let handles: Vec<_> = documents
            .chunks(chunk_size)
            .map(|chunk| scope.spawn(|| count_chunk(chunk)))
            .collect();

        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    // Reduce: merge the partial maps into one.
    let mut total = HashMap::new();
    for partial in partials {
        for (word, count) in partial {
            *total.entry(word).or_insert(0) += count;
        }
    }
    total
}
// ANCHOR_END: parallel

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus() -> Vec<String> {
        (0..500)
            .map(|d| {
                (0..40)
                    .map(|w| format!("word{}", (d * 31 + w) % 97))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    }

    // ANCHOR: test
    #[test]
    fn parallel_agrees_with_sequential() {
        let documents = corpus();

        let sequential = word_counts(&documents);
        let parallel = word_counts_parallel(&documents, 4);

        assert_eq!(sequential, parallel);
    }

    #[test]
    fn parallel_handles_more_threads_than_documents() {
        let documents = vec!["a b".to_string(), "b c".to_string()];

        let got = word_counts_parallel(&documents, 16);

        assert_eq!(got["b"], 2);
        assert_eq!(got["a"], 1);
        assert_eq!(got["c"], 1);
    }

    #[test]
    fn an_empty_corpus_has_no_counts() {
        assert!(word_counts_parallel(&[], 4).is_empty());
    }
    // ANCHOR_END: test
}
