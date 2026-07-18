# Learn Rust with Tests

Learn Rust by writing tests first.

This is a port of Chris James' excellent [Learn Go with Tests][lgwt] to Rust. It
keeps that book's spine — the same chapter arc, the same red/green/refactor
discipline, the same "listen to the compiler" voice — while rewriting every
example in idiomatic Rust.

It is not a mechanical translation. Some chapters port almost directly (Iteration,
Structs, Dependency Injection). Others had to change, because the two languages
disagree about what is hard:

- **Pointers & errors** becomes two chapters, **Ownership & borrowing** and
  **Errors & Result**. Go's chapter teaches pointer receivers and error returns;
  that's the natural place for Rust's core model, and one chapter isn't enough.
- **Reflection** is gone. Rust has no reflection, so that slot teaches
  **Traits & generics** instead.
- **Context** becomes **Cancellation**, since Rust has no `context.Context`.
- **Revisiting arrays and slices with generics** becomes **Iterators & closures** —
  the Go chapter builds `Reduce` by hand; Rust already ships `Iterator`.

And some chapters exist here that can't exist there at all: Ownership & borrowing,
Option & pattern matching, Lifetimes.

[lgwt]: https://github.com/quii/learn-go-with-tests

## How to use it

This is a **workbook**. For each chapter, write the failing test yourself and
watch it fail *before* reading the solution — that red-green-refactor loop is
the thing being taught. Read the chapters in order; the fundamentals are the
spine everything else builds on. The first page of the book, **How to use this
book**, spells out the learning loop in full.

## Reading it

```sh
cargo install mdbook --locked   # once
mdbook serve --open             # http://localhost:3000, live-reloads
```

Or read the Markdown under `src/` directly; `src/SUMMARY.md` is the table of
contents.

## Running the code

```sh
cargo test --workspace          # compile and run every chapter's code
cargo test -p hello-world-v3    # a single snapshot
cargo bench -p iteration-v3     # the criterion benchmarks
cargo run  -p maths-v3 > clock.svg   # chapters with a binary actually run
```

Every chapter keeps one crate per step of its TDD cycle — `code/hello-world/v1`
through `v8` — so each intermediate state the book walks you through is a real
program that really compiles, and diffing one version against the next shows
exactly what a chapter changed. The book's snippets are `{{#include}}`d straight
out of those crates, which means the code on the page is the code the test suite
runs. It can't drift.

The book models idiomatic Rust, so the same checks CI enforces are worth running
on your own attempts:

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

## Layout

```
src/            the book's prose (mdBook)
  SUMMARY.md    table of contents
code/           one directory per chapter, one crate per TDD step
  hello-world/
    v1/ ... v8/
```

## Credit

All credit for the method, structure, and the idea of teaching a language this way
goes to [Chris James][quii] and the Learn Go with Tests contributors. If you write
Go, read the original — it's better than this, and it's where this came from.

This is a community port, not affiliated with or endorsed by Chris James. Any
mistakes in the Rust are mine, not his.

Licensed [MIT](LICENSE), as is the original.

[quii]: https://github.com/quii
