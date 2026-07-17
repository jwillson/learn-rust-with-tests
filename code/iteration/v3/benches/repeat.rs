// ANCHOR: bench
use criterion::{Criterion, criterion_group, criterion_main};
use iteration_v3::repeat;

fn bench_repeat(c: &mut Criterion) {
    c.bench_function("repeat", |b| b.iter(|| repeat("a")));
}

criterion_group!(benches, bench_repeat);
criterion_main!(benches);
// ANCHOR_END: bench
