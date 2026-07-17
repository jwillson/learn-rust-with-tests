# JSON, routing and embedding

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/json)**

We're continuing the player-score server from the [HTTP server](http-server.md)
chapter. So far it can get and record a player's wins. Now we want a *league
table*: `GET /league` returning a JSON list of players sorted by wins.

This gets us into two topics: adding a route (axum's router makes this trivial,
so the Go book's "embedding" digression mostly evaporates for us), and turning
Rust values into JSON with [serde](https://serde.rs), the ecosystem's
serialization framework.

## Write the test first

Following the Go book's excellent advice: **don't test the JSON string,
decode it into data and test that.** Asserting on a raw JSON string is
brittle (reformatting breaks it), hard to debug, and really just re-tests the
serializer — which is already tested by the people who wrote it. Test the
*data* you care about.

So the test hits `/league`, parses the response body back into a `Vec<Player>`,
and asserts on that:

```rust,ignore
{{#include ../code/json/v1/src/lib.rs:test}}
```

`serde_json::from_slice(&bytes)` is the decode step — the mirror of Go's
`json.NewDecoder(...).Decode(&got)`. It's fallible (bad JSON), so we `.expect`
with a message that prints what couldn't be parsed, exactly as the Go version
does with `t.Fatalf`.

## Data modelling

A `Player` is a name and a win count, and to cross the JSON boundary it needs
serde's two derive macros:

```rust,ignore
{{#include ../code/json/v1/src/lib.rs:player}}
```

`Serialize` teaches the type how to *become* JSON; `Deserialize` how to be
*built from* it. `#[derive(...)]` generates both — no reflection, no runtime
schema, just code written for this exact struct at compile time. (That's serde's
whole philosophy, and it's why it's fast: the conversion is monomorphized, like
the generics in the [Traits and generics](traits-and-generics.md) chapter.) Our
test needs `Deserialize` to read the response and the handler needs `Serialize`
to write it, so both are derived here.

## Write enough code to make it pass

Adding the route is one line, and rendering JSON is `axum::Json`:

```rust,ignore
{{#include ../code/json/v1/src/lib.rs:code}}
```

`.route("/league", get(get_league))` chains onto the router next to the
existing `/players/{name}` route. This is where the Go book spends a long
detour: Go's naive `strings.TrimPrefix` routing assumed every request was for
a player, so adding `/league` forced them to introduce `ServeMux` and a whole
discussion of *embedding* to compose it into their server. axum has had a real
router since line one — we just add a route — so that entire section reduces
to a single method call. (Embedding is a Go-specific answer to a problem
Rust's trait system and axum's design don't create; there's nothing to port.)

`axum::Json(league)` wraps our `Vec<Player>` and, because `Player` is
`Serialize`, axum serializes it to JSON and sets the `Content-Type:
application/json` header for us. `.into_response()` normalises it to the same
`Response` type our other handlers return.

## Get the league from the store

Hard-coding Chris was just to get the endpoint green. Now the data should come
from the store, so the trait grows a method:

```rust,ignore
{{#include ../code/json/v2/src/lib.rs:store}}
```

And this is where Rust's compiler does the Go book's homework for it. In Go,
adding `GetLeague()` to the interface produces a cascade of "does not implement"
errors across every store — real ones, stub ones — that you fix one by one.
Same here: the moment you add `league` to the trait, every `impl PlayerStore`
that lacks it fails to compile (`E0046`, "not all trait items implemented"),
listing exactly what's missing. The compiler is your checklist. The stub returns
a stored league; the handler calls the trait:

```rust,ignore
{{#include ../code/json/v2/src/lib.rs:code}}
```

The test stubs a league of three players and asserts it round-trips through
JSON unchanged:

```rust,ignore
{{#include ../code/json/v2/src/lib.rs:test}}
```

## The real store, properly this time

The Go book, being disciplined about minimal steps, first makes its real
`InMemoryPlayerStore.GetLeague()` return `nil` just to satisfy the compiler,
noting the discomfort and parking it. That discomfort is a *missing test*, so
let's write it and implement for real. Building a league means walking the
score map into `Player`s and sorting by wins, descending:

```rust,ignore
{{#include ../code/json/v2/src/lib.rs:in_memory}}
```

A few familiar tools in one method: the `Mutex` guarding shared state
([Sync](sync.md)), the [iterator pipeline](iterators-and-closures.md)
(`.iter().map(...).collect()`) turning map entries into `Player`s, and
`sort_by_key` with `std::cmp::Reverse` to get *descending* order — `Reverse`
is the standard idiom for "sort biggest-first" without writing a backwards
comparison. And because the store's data lives behind a `&self` shared
reference, building the league reads through the lock and clones out owned
`Player`s — the borrow rules deciding, correctly, that we can't hand out
references into a mutex-guarded map.

A direct unit test on the store confirms the sort, no HTTP involved:

```rust,ignore
{{#include ../code/json/v2/src/lib.rs:in_memory_test}}
```

## Wrapping up

- **serde is Rust's JSON (and everything-else) layer.** `#[derive(Serialize,
  Deserialize)]` generates the conversion code at compile time; `axum::Json`
  uses it to render responses, and `serde_json::from_slice` to parse them in
  tests.
- **Test the decoded data, not the JSON string** — string assertions are
  brittle and re-test the serializer. Parse into your types and assert on
  those.
- **Adding a route is one line.** axum's router means the Go book's `ServeMux`
  and *embedding* detour has no Rust counterpart — trait composition and a
  real router were there from the start.
- **Growing a trait is a compiler-checked refactor**: add `league` to
  `PlayerStore` and every implementer that lacks it fails to build, with the
  missing method named. The "make it return `nil` to shut the compiler up"
  temptation becomes "write the test and implement it", because the type
  system won't let the gap hide.

Scores still live in memory. Next we make the store persist to a file — and in
doing so, revisit the `Read`/`Write` traits from the
[Reading files](reading-files.md) chapter in a new setting.
