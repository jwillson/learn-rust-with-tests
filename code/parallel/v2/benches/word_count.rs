use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use parallel_v2::{word_counts, word_counts_parallel};

fn corpus(documents: usize, words_per_doc: usize, vocabulary: usize) -> Vec<String> {
    (0..documents)
        .map(|d| {
            (0..words_per_doc)
                .map(|w| format!("word{}", (d * 31 + w) % vocabulary))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

fn bench_word_counts(c: &mut Criterion) {
    let documents = corpus(20_000, 100, 1_000);
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    c.bench_function("sequential", |b| b.iter(|| word_counts(&documents)));
    c.bench_function("parallel", |b| {
        b.iter(|| word_counts_parallel(&documents, threads))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20).measurement_time(Duration::from_secs(10));
    targets = bench_word_counts
}
criterion_main!(benches);
