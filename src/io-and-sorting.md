# IO and sorting

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/io-sorting)**

Our league server works, but everything vanishes on restart because the store
is in memory. This chapter gives it a memory: we'll persist the league to a
file as JSON. Along the way we revisit the `Read`/`Write` traits from the
[Reading files](reading-files.md) chapter, meet their partner `Seek`, and hit
a classic file-rewriting bug that teaches why overwriting isn't as simple as it
looks.

We're storing JSON in a plain file. It won't scale to millions of players, but
it's portable, simple, and — thanks to the `PlayerStore` trait — trivial to
swap for a real database later. That's the payoff of the abstraction: the
server neither knows nor cares what's behind the trait.

## The testable seam

Go's book tests its file store against `strings.NewReader` and a real file
interchangeably, because both satisfy `io.ReadWriteSeeker`. Rust has the exact
same idea, spelled with three traits: `Read`, `Write`, and `Seek`. A
`std::fs::File` implements all three — and so does `std::io::Cursor<Vec<u8>>`,
an in-memory buffer that *pretends to be a file*, seek position and all. That
`Cursor` is our test double: no disk, no tempfiles, fast, and it exercises the
identical code path a real file will.

So our store is generic over the backing type:

```rust,ignore
pub struct FileSystemPlayerStore<F> { /* ... */ }

impl<F: Read + Write + Seek + Truncate> FileSystemPlayerStore<F> { /* ... */ }
```

(`Truncate` is a small extra trait we'll explain when the bug that needs it
shows up.) Tests instantiate `FileSystemPlayerStore<Cursor<Vec<u8>>>`; `main`
instantiates `FileSystemPlayerStore<File>`. Same code, different `F`.

## Write the test first

Start with reading a league out of the store. The helper builds a store from a
JSON string via `Cursor`, and the test checks it comes back sorted:

```rust,ignore
{{#include ../code/io-sorting/v1/src/lib.rs:helper}}
```

```rust,ignore
    #[test]
    fn reads_a_league_sorted_by_wins() {
        let store = store_from(
            r#"[{"name": "Cleo", "wins": 10}, {"name": "Chris", "wins": 33}]"#,
        );

        let got = store.league();

        // Chris (33) comes before Cleo (10)
    }
```

`r#"..."#` is a *raw string* — no escaping needed for the JSON's quotes, which
is why it's perfect for embedding JSON, regex, or Windows paths.

## An important design decision: read once

The naive implementation re-reads and re-parses the whole file on every single
call. The Go book does this first, then flags it as a performance problem and
refactors to parse once. We'll skip straight to the good design, because Rust's
ownership model nudges us there anyway: we parse the file in the constructor and
cache the league in memory.

```rust,ignore
{{#include ../code/io-sorting/v1/src/lib.rs:store}}
```

A lot to notice, all of it drawing on earlier chapters:

- **`new` reads the whole file once.** `read_league` slurps the contents,
  treats an empty file as an empty league (a real case — a brand-new database),
  and otherwise parses with serde, converting a JSON error into an `io::Error`
  so the constructor has one error type. This is the `?`-friendly error
  handling from the [Errors chapter](errors-and-result.md).
- **State lives behind a `Mutex`.** The `PlayerStore` methods take `&self` (the
  server shares the store across async tasks via `Arc`), but recording a win
  *mutates* — both the cached league and the file. That's shared mutable state,
  and the [Sync chapter](sync.md) taught us its name: `Mutex`. `record_win`
  locks, mutates, and rewrites; `score` and `league` lock and read.
- **`league()` sorts descending** with `sort_by_key(|p| Reverse(p.wins))`, the
  idiom from the [JSON chapter](json-routing-embedding.md).

## Seeking problems

Now `record_win`, and here's where files get interesting. To rewrite the file
we `seek` back to the start and write the new JSON. But watch what happens if
the new content is *shorter* than the old — say we had a big league and now
write a small one, without truncating:

```text
[{"name":"Cleo","wins":1}]}]
```

The `}]` on the end is *leftover* — bytes from the longer previous content that
our shorter write didn't cover. Seeking to position 0 and writing overwrites
from there, but it doesn't shorten the file. The result is invalid JSON, and
the next read fails. Go's book solves this with a "tape" abstraction that
truncates on write; we solve it by truncating explicitly, and that's what our
little `Truncate` trait is for:

```rust,ignore
{{#include ../code/io-sorting/v1/src/lib.rs:truncate}}
```

This is a nice illustration of *defining exactly the capability you need*. A
`File` can truncate via `set_len`; a `Cursor<Vec<u8>>` truncates by shortening
its inner `Vec`. Neither method lives on the `Read`/`Write`/`Seek` traits, so
we declare the one operation we require and implement it for both backings.
Now `record_win` writes the new JSON and truncates to its exact length:

```rust,ignore
        let bytes = serde_json::to_vec(&inner.league).expect("a league always serializes");
        inner.database.seek(SeekFrom::Start(0)).unwrap();
        inner.database.write_all(&bytes).unwrap();
        inner.database.truncate(bytes.len() as u64).unwrap();
```

Seek to start, overwrite, cut off any tail. The full set of store tests — read
league, read score, record for an existing player, record for a new player,
and start from an empty file — all pass against in-memory `Cursor`s.

```rust,ignore
{{#include ../code/io-sorting/v1/src/lib.rs:test}}
```

## Wiring it into the server

The file store satisfies the same `PlayerStore` trait the server already
depends on, so plugging it in changes *nothing* about the server. We implement
the trait (the inherent methods become trait methods, gaining the `Send` bound
the shared server requires):

```rust,ignore
{{#include ../code/io-sorting/v2/src/lib.rs:impl}}
```

And because the store is behind `Arc<dyn PlayerStore>`, an integration test can
drive the *whole stack* — HTTP in, file store underneath — proving persistence
survives across requests:

```rust,ignore
{{#include ../code/io-sorting/v2/src/lib.rs:integration_test}}
```

Three POSTs then a GET returns `3`, the wins accumulating in the (in-memory,
for the test) file between requests. Swapping `Cursor` for a real `File` in
`main` makes it survive across *process restarts*:

```rust,ignore
{{#include ../code/io-sorting/v2/src/main.rs:main}}
```

`OpenOptions` with `read`, `write`, and `create` opens (or creates) the
database file with the permissions our store needs — read to load it, write to
update it, `truncate(false)` so opening an existing database doesn't wipe it.
Run it, POST some wins, kill it, restart it: the scores are still there. It's a
real, persistent application.

## Wrapping up

- **`Read + Write + Seek` is Rust's `io.ReadWriteSeeker`**, and
  `Cursor<Vec<u8>>` is the in-memory implementation that makes file logic
  testable without touching disk — the exact analogue of testing with a
  string reader.
- **Rewriting a file in place needs truncation.** Seeking to the start and
  writing overwrites but doesn't shorten; leftover bytes from longer previous
  content corrupt the result. We fixed it by declaring a narrow `Truncate`
  capability and implementing it for both our backings — Go's "tape", Rust's
  way.
- **Parse once, cache in memory**, and guard the mutable cache with a `Mutex`
  because the store is shared (`&self`) and record-a-win mutates. The Sync
  chapter's `Arc<Mutex<_>>` shape, again.
- **The `PlayerStore` trait made the swap free.** The server was written and
  tested against a stub, run against an in-memory store, and now runs against
  a file store — none of the server code changed. That is what a good
  abstraction buys you.

The application now persists. Next we give it a proper command-line interface
and clean up its project structure so it can grow.
