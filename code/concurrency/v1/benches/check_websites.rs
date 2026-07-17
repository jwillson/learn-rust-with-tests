// ANCHOR: bench
use std::time::Duration;

use concurrency_v1::check_websites;
use criterion::{Criterion, criterion_group, criterion_main};

fn slow_stub_website_checker(_url: &str) -> bool {
    std::thread::sleep(Duration::from_millis(20));
    true
}

fn bench_check_websites(c: &mut Criterion) {
    let urls = ["a url"; 100];

    c.bench_function("check_websites", |b| {
        b.iter(|| check_websites(slow_stub_website_checker, &urls))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(30));
    targets = bench_check_websites
}
criterion_main!(benches);
// ANCHOR_END: bench
