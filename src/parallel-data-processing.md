# Parallel data processing

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/parallel)**

The last project cashes in Rust's headline promise: **fearless concurrency**.
The [Concurrency](concurrency.md) and [Sync](sync.md) chapters taught the
primitives; the application section used them for *I/O* concurrency (a server
juggling connections). This chapter is about *compute* parallelism — taking a
CPU-bound job and splitting it across cores to make it genuinely faster — which
is where "fearless" earns its keep. The whole thing is a `make it work, make it
right, make it fast` story, and we'll finish with real benchmark numbers.

Our task: count how often each word appears across a big pile of documents.

## Make it work: sequential

```rust,ignore
{{#include ../code/parallel/v1/src/lib.rs:code}}
```

Nothing surprising — iterate the documents, split each into words, tally them in
a `HashMap` with the `entry` API from the [HashMaps chapter](hashmaps.md). Tests
lock in the behaviour:

```rust,ignore
{{#include ../code/parallel/v1/src/lib.rs:test}}
```

## Measure before optimising

The [Refactoring checklist](refactoring-checklist.md) chapter's discipline — and
Knuth's warning about premature optimisation — say: don't parallelise on a hunch,
*measure*. A criterion benchmark over a big generated corpus (20,000 documents,
100 words each) gives us the baseline:

```text
sequential              time:   [75.598 ms 76.128 ms 76.534 ms]
```

About 76 milliseconds. Now we have a number to beat.

## Make it fast: map-reduce across threads

The plan is the tea-making principle applied to computation: instead of one
thread counting all 20,000 documents, split them into chunks and count each
chunk on its own thread, then combine the results. This is **map-reduce**:

- **Map** — each thread counts its chunk into a *private* `HashMap`.
- **Reduce** — merge the partial maps into one total.

The critical design choice is that the threads share *nothing mutable*. Each
builds its own map, so there's no lock, no contention, no shared state to race
over. Only at the end, on one thread, do we merge.

```rust,ignore
{{#include ../code/parallel/v2/src/lib.rs:parallel}}
```

Two things make this clean and safe:

- **`std::thread::scope`** (from the Concurrency chapter) lets the spawned
  threads *borrow* `documents` — the chunks are slices into it — because the
  scope guarantees every thread finishes before it returns. No `Arc`, no
  cloning the corpus.
- **No `Mutex` anywhere.** Because each thread owns its partial map and we merge
  sequentially afterward, there's no shared mutable state. This is the Sync
  chapter's advice in action — "use channels/ownership to pass data, mutexes
  only when you must share" — and here we simply don't share. The result is that
  the parallel version is almost as easy to read as the sequential one.

And here's the fearless part: **it is impossible to write a data race here that
compiles.** If we'd tried to have all threads write into one shared `HashMap`
without synchronisation, the borrow checker would have rejected it (`E0499`, the
same error the whole book keeps meeting), exactly as it did for the naive
counter in the Sync chapter. The compiler doesn't let us ship the bug. In most
languages, parallelising a computation means carefully auditing for races; in
Rust, the code that has a race doesn't build.

## Prove it's still correct

Parallelism is a refactor — the *result* must not change. So the key test runs
both implementations on the same corpus and asserts they produce identical maps:

```rust,ignore
{{#include ../code/parallel/v2/src/lib.rs:test}}
```

This is the [specification-across-two-backends](bytecode-virtual-machine.md)
pattern one final time: the sequential version *is* the specification, and the
parallel version must match it exactly, plus edge cases (more threads than
documents, an empty corpus). With that green, we've made it fast without making
it wrong.

## The payoff

Benchmarking both on the same corpus:

```text
sequential              time:   [75.598 ms 76.128 ms 76.534 ms]
parallel                time:   [28.429 ms 28.992 ms 29.667 ms]
```

From 76 ms to 29 ms on an 8-core machine — about **2.6× faster**. Worth being
honest about that number: it's *not* 8×, and understanding why is the real
lesson. The map phase parallelises beautifully, but the **reduce** phase — merging
the partial maps — is sequential, and allocating all those `String` keys costs
memory bandwidth that's shared across cores. This is [Amdahl's
law](https://en.wikipedia.org/wiki/Amdahl%27s_law): your speedup is capped by the
part that stays serial. A 2.6× win for a dozen lines of straightforward,
race-free code is an excellent trade — and the benchmark is what tells you it was
worth doing, rather than a guess.

## And the one-liner: rayon

We built this with the standard library to *see* the mechanics, and that's the
right way to learn it. But in real Rust you'd often reach for
[rayon](https://docs.rs/rayon), the data-parallelism crate, which turns a
sequential iterator into a parallel one by changing `iter()` to `par_iter()`:

```rust,ignore
// with rayon:
documents
    .par_iter()
    .map(|doc| count_one(doc))
    .reduce(HashMap::new, merge);
```

Same map-reduce shape, same fearless guarantees, a fraction of the code — rayon
handles the chunking, the thread pool, and the work-stealing for you. Knowing
what it does under the hood (which you now do) is what lets you use it well.

## Wrapping up

- **Compute parallelism is map-reduce**: split the work, process chunks on
  separate threads into private results, then merge. `std::thread::scope` lets
  those threads borrow the input safely.
- **Share nothing mutable and there's nothing to race over.** Private per-thread
  maps plus a sequential merge means no `Mutex`, no contention — and the borrow
  checker guarantees the absence of data races at compile time. That is
  *fearless concurrency*: not that parallel code is trivial to design, but that
  the race conditions can't compile.
- **Measure, don't guess.** A benchmark gave us the baseline, justified the
  work, and reported the real 2.6× speedup — sub-linear because of Amdahl's law,
  which is the honest shape of most parallelism.
- **The ecosystem (rayon) makes it a near one-liner**, but the standard-library
  version teaches you what's actually happening.

That's the projects section — and very nearly the whole book. You've built a
language, a data structure that taught you to think in handles, and a parallel
processor that turned Rust's safety guarantees into free performance. The
remaining chapters step back from building to reflect on the *practice* of
testing that carried you through all of it.
