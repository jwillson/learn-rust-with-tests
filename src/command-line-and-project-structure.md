# Command line and project structure

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/command-line)**

Our product owner wants a second way to interact with the league: a command-line
program for playing poker, where typing `{name} wins` records a win. It should
share the same store as the web server, so scores stay in sync no matter how
they're entered.

This is really a chapter about **project structure**: how to have one body of
shared logic feeding two different front ends. In Rust that structure is
built into Cargo, and it's clean.

## One library, two binaries

A Cargo package can be *both* a library and one-or-more binaries at once. The
convention:

- `src/lib.rs` — the library. All the reusable logic (types, the
  `PlayerStore` trait, the file store, the axum server, and now a `Cli` type)
  lives here and is `pub`.
- `src/bin/server.rs` and `src/bin/cli.rs` — each file in `src/bin/` compiles
  to its own executable. They're thin: parse the environment, build the store,
  hand off to the library.

`cargo run --bin server` and `cargo run --bin cli` run each one. Both depend on
the same `command_line_v1` library, so there is exactly one copy of the poker
logic, tested once, used twice. This is Cargo's answer to the Go book's `cmd/`
directory layout — same idea, less ceremony, and the library/binary split is a
first-class concept rather than a folder convention.

## Write the test first

We want a `Cli` that reads user input and records wins to a `PlayerStore`. As
always, we design it from the test's point of view first, and — crucially — we
make its input a **`BufRead`**, not "stdin". This is the
[dependency-injection](dependency-injection.md) reflex the whole book has
drilled: depend on the *capability* (something you can read lines from), not
the concrete source. In `main` we'll pass real stdin; in tests we pass a string:

```rust,ignore
{{#include ../code/command-line/v1/src/lib.rs:test}}
```

`"Chris wins\n".as_bytes()` is a `&[u8]`, and — just like the
[Reading files](reading-files.md) chapter — `&[u8]` implements `BufRead`, so it
*is* a valid input source. No stdin, no interaction, no flakiness: the test
types "Chris wins" by handing over some bytes.

The spy is our familiar `Arc<dyn PlayerStore>` test double, recording win calls
behind a `Mutex` (the store is shared, so `&self` methods plus interior
mutability, exactly as in the [Sync chapter](sync.md)):

```rust,ignore
{{#include ../code/command-line/v1/src/lib.rs:spy}}
```

## Write enough code to make it pass

Start by faking it — record a hard-coded `"Chris"` — then the second test
(`Cleo wins`) forces us to actually *read and parse* the input:

```rust,ignore
{{#include ../code/command-line/v1/src/lib.rs:cli}}
```

`Cli<R>` is generic over its reader `R: BufRead`, so it works with `&[u8]` in
tests and real stdin in production with no runtime cost — the monomorphization
from the [Traits and generics](traits-and-generics.md) chapter. `play_poker`
reads a line, trims the newline, and `strip_suffix(" wins")` gives us the name
if the line has the expected shape — the same `Option`-returning string method
we used for parsing in the [Reading files](reading-files.md) and
[Property-based tests](property-based-tests.md) chapters. `if let Some(name)`
records the win only when the input matched.

## The two binaries

The server binary is what we already had, unchanged — opening the database
file and serving HTTP:

```rust,ignore
{{#include ../code/command-line/v1/src/bin/server.rs:server}}
```

The CLI binary opens the *same* database file, then reads real user input by
wrapping `std::io::stdin()` in a `BufReader` (which adds the line-buffering
`BufRead` needs) and handing it to `Cli`:

```rust,ignore
{{#include ../code/command-line/v1/src/bin/cli.rs:cli_main}}
```

That `BufReader::new(std::io::stdin())` is the whole trick: `Cli` asked for
"anything I can read lines from", and both a test's `&[u8]` and the terminal's
stdin qualify. The dependency injection we've practiced since early in the book
is what lets the *exact same* `Cli::play_poker` run under test and in a user's
terminal.

Run it for real — `echo "Ruth wins" | cargo run --bin cli` — and the database
file gains `[{"name":"Ruth","wins":1}]`. Start the server afterwards and
`Ruth` is in the league. Two programs, one store, one source of truth.

## Wrapping up

- **A Cargo package is a library plus binaries.** Put shared logic in
  `src/lib.rs` and thin entry points in `src/bin/*.rs`; each becomes an
  executable, all sharing one tested library. This is Rust's project structure
  for "several front ends, one core".
- **Depend on `BufRead`, not stdin.** Making `Cli` generic over its reader
  meant the same code is driven by a `&[u8]` in tests and the terminal in
  production — dependency injection over the standard library's I/O traits,
  the pattern this whole book keeps returning to.
- **Simple abstractions make re-use free.** The `PlayerStore` trait, written
  chapters ago for the web server, dropped straight into a command-line
  program with no changes. A second consumer of your core logic should be
  cheap; if it isn't, your abstractions are telling you something.

Next we add a *time* dimension to the poker game — blinds that increase on a
schedule — which means testing code that depends on the clock.
