# How to use this book

This book is a **workbook**, not a textbook. It's written to be *done*, not just
read. Every chapter drives out a piece of working code test-first, and you'll
learn far more by writing that code yourself than by reading the finished
version.

## The learning loop

For each chapter, work in this loop:

1. **Read** the chapter up to the first failing test.
2. **Write that test yourself**, in your own scratch crate, and run it. Watch it
   fail — and read the failure. A big part of what the book teaches is how to
   *listen to the compiler and the test output*, and you only get that by seeing
   the red for real.
3. **Try to make it pass** before reading on. Struggle a little; it's where the
   learning happens.
4. **Compare** your version against the chapter's crate — the "answer key" — and
   read the rest of the chapter for the reasoning and the refactor.

Then repeat for the next test. This is the red-green-refactor cycle, and it *is*
the thing the book is teaching. The Rust is the vehicle.

## Read the chapters in order

The chapters build on one another. The **Rust Fundamentals** section is the
spine — ownership, `Result` and `Option`, traits, the borrow checker — and
everything after it assumes those ideas. Later sections lean on earlier ones
constantly, so resist the urge to skip ahead, especially early on. You can use
it as a reference later, but the first read should be front to back.

The four movements:

- **Rust Fundamentals** — the language, one small test at a time.
- **Build an Application** — a real poker app: async, an HTTP/WebSocket server,
  persistence, a CLI.
- **Rust Projects** — a small programming language, an LRU cache, and parallel
  processing: the chapters that feel most distinctly Rust.
- **Testing Fundamentals, Q&A, and Meta** — consolidating the craft of testing
  itself.

## Reading the book

The book is an [mdBook](https://rust-lang.github.io/mdBook/). To read it locally
with search and navigation:

```sh
cargo install mdbook --locked   # once, if you don't have it
mdbook serve --open             # opens http://localhost:3000, live-reloads
```

Or read the Markdown under `src/` directly — `src/SUMMARY.md` is the table of
contents.

## Running the code

All the code lives in one Cargo workspace, with **one crate per step of each
chapter's TDD cycle**. That's the answer key: `code/hello-world/v1` through
`v8` are the eight successive states the Hello, World chapter walks you through,
each a real program that compiles. Diffing one version against the next shows
exactly what a chapter changed.

```sh
cargo test --workspace          # compile and run every chapter's code
cargo test -p hello-world-v3    # run a single snapshot
cargo bench -p iteration-v3     # the criterion benchmarks
cargo run  -p maths-v3 > clock.svg   # chapters with a binary actually run
```

Because every snippet in the prose is pulled from these crates with
`{{#include}}`, the code you read is the code the tests run — it can't drift out
of sync with the text.

## Hold your own code to the book's standard

The book models *idiomatic* Rust, and the same checks that keep it honest are
worth running on your own attempts:

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Continuous integration runs exactly these, plus `mdbook build`, so a broken
snippet or a failing test fails the build. The book cannot lie about code that
doesn't compile.

## The one habit that matters

If you take nothing else from this: **for every chapter, write the test and
watch it fail before you look at the answer.** That single discipline is what
turns reading about TDD into being able to do it.
