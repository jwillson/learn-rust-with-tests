# Learn Rust with Tests

## Why

- Explore the Rust language by writing tests.
- **Get a grounding with TDD**. Rust is a good language for learning TDD because
  it is a highly performant, memory-safe language with testing built in — no
  frameworks to choose, no libraries to install — and its famously strict
  compiler makes the feedback loop at the heart of TDD faster and richer.
- Be confident that you'll be able to start writing robust, well-tested systems
  in Rust.

## Where this came from

This book is a port of [Learn Go with Tests][lgwt] by Chris James. The method, the
chapter arc, and the conviction that you learn a language fastest by writing tests
in it are all his. If you write Go, go read the original.

Porting it turned out not to be a translation exercise. Go and Rust disagree about
what is hard, and the book had to move where the difficulty is. Go's chapter on
pointers and errors becomes two chapters here, on ownership and on `Result`,
because that's the wall Rust puts in front of you and one chapter isn't enough to
climb it. Go's chapter on reflection is gone, because Rust has no reflection. And
three chapters here — ownership, `Option` and pattern matching, lifetimes — have no
counterpart in the original at all, because Go has nothing like them.

[lgwt]: https://github.com/quii/learn-go-with-tests

## Test-driven development

The cycle, which this book repeats until it's boring and then repeats more:

1. **Write a test.**
2. **Make the compiler pass.** Not the test — the compiler. In Rust this step is
   bigger than it is in most languages, and it's doing more work for you.
3. **Run the test, see that it fails**, and check the error message is meaningful.
4. **Write enough code to make the test pass.**
5. **Refactor.**

Step 3 is the one everyone skips, and it's the one that matters. A test you have
never seen fail is not a test. It's a decoration that will keep passing after the
code beneath it rots.

## A note on the compiler

The original book tells you to listen to the compiler. In Rust that advice is
worth more, because the compiler has more to say. Ownership errors, lifetime
errors, non-exhaustive `match` arms — these arrive as compile errors, and each one
is a bug that would have been a runtime failure somewhere else.

So the compiler errors printed in this book are real. They were produced by
actually writing the broken code and running `cargo test` on it, not by
paraphrasing from memory. When the book says you'll see an error, you'll see that
error.

## Who this is for

People who are interested in picking up Rust, and people who already know some
Rust but want to explore testing with TDD. You don't need to know Go.

## What you'll need

- A computer.
- [Installed Rust](install-rust.md).
- A text editor.
- A terminal.

## Feedback

This is a port, and ports have seams. If a chapter feels like Go wearing a Rust
costume, that's a bug — please open an issue.

Licensed [MIT](https://github.com/jwillson/learn-rust-with-tests/blob/main/LICENSE),
as is the original.
