# Reading files

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/reading-files)**

In this chapter we're going to read some files, get data out of them, and do
something useful. Pretend you're building blog software with a friend: authors
write posts in markdown with a little metadata at the top, and on startup the
web server reads a folder to build a collection of `Post`s.

A post file looks like this:

```markdown
Title: Hello, TDD world!
Description: First post on our wonderful blog
Tags: tdd, rust
---
Hello world!

The body of posts starts after the `---`
```

and the thing we want out of it is:

```rust,ignore
{{#include ../code/reading-files/v2/src/lib.rs:post}}
```

## Thinking about the kind of test we want to see

The Go book's central lesson here is *don't couple your parser to the file
system*. It reaches for Go's `io/fs` abstraction and an in-memory
`fstest.MapFS` so its tests never touch a disk. Rust gives us an even cleaner
seam, and it's one this book has leaned on since the
[Dependency injection](dependency-injection.md) chapter: the
[`Read`](https://doc.rust-lang.org/std/io/trait.Read.html) family of traits.

Here's the key move. Parsing a post is really *two* jobs — walking a directory
to find files, and turning one file's *bytes* into a `Post`. Only the first
job cares about the file system. So we'll build the parser against
`impl BufRead` — "anything I can read lines from" — and a file, a network
socket, or a plain `&str` in a test all satisfy it. Where Go injects a fake
file system, we inject the bytes directly and skip the file system entirely
for the interesting logic. That's the tightest possible feedback loop.

## Write the test first

Start as small as useful: pull the title out of a post. And notice the test
double is just a string's bytes — no folder, no `tempdir`, nothing to clean
up:

```rust,ignore
#[test]
fn parses_the_title_from_a_post() {
    let post = post_from_reader("Title: Hello, TDD world!".as_bytes()).unwrap();

    assert_eq!(post.title, "Hello, TDD world!");
}
```

`"...".as_bytes()` gives a `&[u8]`, and the standard library implements
`Read` (and `BufRead`) for `&[u8]` precisely so that in-memory data is
a first-class reader. This is the same trick as the Maths chapter rendering
into a `Vec<u8>` instead of a file, run in reverse.

## Write enough code to make it pass

`post_from_reader` returns `std::io::Result<Post>` — reading can fail (a
malformed post, an I/O error), and we learned in the
[Errors chapter](errors-and-result.md) to make fallibility visible in the
type. After the stub returns an empty title and fails as expected, the real
version reads the first line and strips the prefix:

```rust,ignore
{{#include ../code/reading-files/v1/src/lib.rs:code}}
```

Three things earn their place here:

- **`reader.lines()`** is a `BufRead` method returning an *iterator* of
  `Result<String>` — one per line, each fallible because reading can go wrong
  mid-stream. That's why `line?` appears: each line must be unwrapped before
  use, and `?` propagates a real I/O error while handing us the `String`.
- **`strip_prefix` returns `Option`** (our [Option chapter](option-and-pattern-matching.md)
  friend), and `ok_or_else` converts the "no such prefix" `None` into our
  error — the standard bridge from "missing" to "failed".
- **`std::io::Error::new(ErrorKind::InvalidData, ...)`** builds an error of
  the standard I/O family, so a bad post and a bad disk read are the same
  type flowing through the same `?`.

The tests for a second title and for a missing-title rejection come along
easily. Green.

## Write the test first: the whole post

Now parse everything. The test asserts on a complete `Post` value — and
because we `#[derive(PartialEq)]`, `assert_eq!` compares the whole struct at
once:

```rust,ignore
{{#include ../code/reading-files/v2/src/lib.rs:test}}
```

(That `"\` at the start of the string literal is a line-continuation —
it swallows the newline so the content starts cleanly on the next line.)

## Write enough code to make it pass

The metadata lines all follow the same shape — a fixed prefix, then a value
— so a helper captures that, and the body is everything after the `---`:

```rust,ignore
{{#include ../code/reading-files/v2/src/lib.rs:code}}
```

A few things worth slowing down for.

**The separator check is a `match`-then-`if`, not a one-liner — and there's a
real reason.** The natural thing to write is a match guard:

```rust,ignore
match lines.next() {
    Some(line) if line? == SEPARATOR => {}
    _ => return Err(...),
}
```

but the compiler stops you cold:

```text
error[E0507]: cannot move out of `line` in pattern guard
 --> src/lib.rs:6:23
  |
6 |         Some(line) if line.unwrap() == "---" => true,
  |                       ^^^^ move occurs because `line` has type `Result<String, std::io::Error>`, which does not implement the `Copy` trait
  |
  = note: variables bound in patterns cannot be moved from until after the end of the pattern guard
```

This is a genuinely subtle borrow-checker rule and it's worth understanding:
a match guard runs *before* the arm is chosen, so if the guard consumed
`line` and then the arm *didn't* match, the value would be gone with nowhere
to go. Rust forbids moving out of a pattern binding inside its own guard. The
fix is to pull the value out first, then test it — which reads more plainly
anyway.

**`collect` can build a `Result`.** Look at
`lines.collect::<std::io::Result<_>>()?` — the body lines are an iterator of
`Result<String>`, and `collect` knows how to turn an *iterator of Results*
into a *Result of a collection*: `Ok(vec)` if every line read cleanly, or the
first `Err` otherwise. This is one of `collect`'s quiet superpowers, and it's
why one `?` at the end handles a read failure on any body line. The
[Iterators chapter](iterators-and-closures.md) promised `collect` was
cleverer than it looked.

**`read_meta_line` is generic over the iterator**, taking
`&mut impl Iterator<Item = std::io::Result<String>>` so it can be handed the
same `lines` iterator repeatedly, consuming a line each call — the
[Traits and generics](traits-and-generics.md) machinery doing ordinary work.

## Now the file system

We've done the hard part with zero disk access. The remaining job — walk a
directory, read each file — is small, and *this* is the layer that talks to
`std::fs`:

```rust,ignore
{{#include ../code/reading-files/v3/src/lib.rs:from_dir}}
```

- **`impl AsRef<Path>`** is the idiomatic "accepts a path-like thing"
  parameter: a `&str`, a `String`, a `PathBuf`, or a `&Path` all work,
  because they all implement `AsRef<Path>`. It's the ergonomic front door the
  standard library's own file functions use.
- **`read_dir` yields `Result`s** (a file could vanish mid-iteration), so we
  `collect` into a `Result<Vec<_>>` again — same trick as the body lines.
- **We `sort` the paths.** `read_dir` returns entries in whatever order the
  OS provides — *not* alphabetical, and not stable across machines. For tests
  that assert on "the first post" to mean anything, we impose an order
  ourselves. (Directory iteration order being unspecified is a classic
  cross-platform test flake; sorting is the cheap cure.)
- **Each file becomes a `BufReader`** wrapping the `File` and handed straight
  to the `post_from_reader` we already built and tested. `BufReader` adds the
  buffering that makes `BufRead`'s line-reading efficient.

For this crate's tests we *do* use real files — a `testdata/` folder beside
the source — because now we're specifically testing the file-walking. The
parser's own tests stayed in-memory and fast; only the thin file-system layer
pays for disk access. That division is the whole point.

```rust,ignore
{{#include ../code/reading-files/v3/src/lib.rs:test}}
```

## Wrapping up

- **Decouple parsing from the file system by taking a `Read`/`BufRead`**, not
  a path. The logic-heavy code then tests against in-memory bytes —
  `"...".as_bytes()` — with no disk, no fixtures, no cleanup. Rust's I/O
  traits are the injection seam, the same one from the Dependency injection
  and Maths chapters.
- **Confine file-system code to a thin outer layer** (`posts_from_dir`) that
  finds files and delegates each one to the pure parser. Accept paths as
  `impl AsRef<Path>`; remember to `sort` directory listings.
- **`collect::<Result<_>>()`** turns an iterator of `Result`s into one
  `Result`, so a single `?` handles a failure anywhere in the sequence — for
  both the body lines and the directory entries.
- **`lines()` yields fallible `String`s**, and match guards can't move their
  bindings (E0507) — bind first, test after.

The Go book ends by noting this design lets consumers embed posts in the
binary, load them from a zip, or read a real folder, all without the parser
knowing. The same is true here, and more concretely: anything implementing
`Read` — a file, a TCP stream, an HTTP response body, a decompressor, an
in-memory buffer — flows through `post_from_reader` unchanged. Programming to
the standard library's small, sharp traits is how Rust code stays reusable in
situations you never imagined.
