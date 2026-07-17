# HTTP server

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/http-server)**

You've been asked to create a web server where users can track how many games
players have won. `GET /players/{name}` returns a player's total wins;
`POST /players/{name}` records a win.

This is the first chapter of the book's application section, and it introduces
two big things at once: **HTTP in Rust**, and **async**. Neither lives in the
standard library — Rust leaves web serving to the ecosystem — so we'll use the
stack most Rust jobs use: [axum](https://docs.rs/axum) for the web framework
and [tokio](https://tokio.rs) as the async runtime underneath it. We'll meet
`async`/`await` as we go, in small doses, driven by tests.

## A word on async, before we start

Every network handler we write will be an `async fn`. Here's the whole idea in
one breath: an `async fn` doesn't run when you call it — it returns a *future*,
a value representing "work that will produce a result later". The runtime
(tokio) polls futures forward, and when one is waiting on I/O — a slow network
read, say — the runtime runs *other* futures on the same thread instead of
blocking. It's the [Concurrency](concurrency.md) chapter's tea-making
principle, but for thousands of connections on a handful of threads. You get a
future's value by `.await`-ing it, which is only allowed inside another
`async` context.

That's enough theory. The mechanics will make sense once you see them work.

## Write the test first

Start by faking it, à la Kent Beck: a handler that returns a hard-coded `"20"`.
This gets the whole project skeleton — routing, the test harness, async — in
place before we worry about logic.

Testing an axum app doesn't need a running server on a real port. axum apps are
[tower](https://docs.rs/tower) *services*, and tower gives us `oneshot`: feed a
service one request, get one response, all in memory. That's our
`httptest.ResponseRecorder` equivalent, and it's faster and more direct.

```rust,ignore
{{#include ../code/http-server/v1/src/lib.rs:test}}
```

The `#[tokio::test]` attribute (instead of plain `#[test]`) spins up a tokio
runtime for the test, so we're allowed to `.await` inside it. Everything else
is wrapped in a helper:

```rust,ignore
{{#include ../code/http-server/v1/src/lib.rs:helpers}}
```

Reading it: build a `GET /players/{name}` request, hand it to the server with
`oneshot(...).await`, then collect the streamed response body into bytes and
turn it into a `String`. (Response bodies are streamed, hence the
`.collect().await` dance — a body might arrive in pieces off a socket.) The
helper returns both status and body so later tests can check each.

## Write enough code to make it pass

```rust,ignore
{{#include ../code/http-server/v1/src/lib.rs:code}}
```

`player_server` builds a `Router` — axum's request map — with one route:
`GET /players/{name}` handled by `get_score`. The `{name}` is a path
parameter; we ignore it for now. `get_score` is our first `async fn` handler,
and returning a `String` is all it takes for axum to send a `200 OK` with that
text as the body. Green.

Notice how little ceremony there is compared to Go's `ResponseWriter`/`Request`
pair: axum uses the return value as the response, and (as we'll see) function
parameters to pull data *out* of the request. This is axum's whole design —
handlers are ordinary functions, and their signatures declare what they need.

## Wire up an application

A test passing is good; software running is better. A `src/main.rs` beside the
library turns this into a real server:

```rust,ignore
{{#include ../code/http-server/v4/src/main.rs:main}}
```

`#[tokio::main]` is the mirror of `#[tokio::test]`: it wraps `main` so it can
be `async` and starts the runtime. We bind a `TcpListener` to port 5000 and
hand it to `axum::serve` along with our router. `axum::serve(...).await` runs
forever, accepting connections. (This is the finished version's main — it uses
the store we're about to build — but the shape is fixed from here.)

## Break the constant

Add a test for a second player and the hard-coded `"20"` falls over — Floyd
should score `10`. Now we must actually *look at* the request:

```rust,ignore
{{#include ../code/http-server/v2/src/lib.rs:code}}
```

`Path(name): Path<String>` is an axum **extractor**. By adding a parameter of
type `Path<String>`, we're telling axum "pull the path parameter out and give
it to me" — and it does, or rejects the request if it can't. This is the
pattern-matching destructuring from the [Option chapter](option-and-pattern-matching.md)
applied to a function parameter: `Path(name)` unwraps the newtype so `name` is
just a `String`. Extractors are how axum handlers stay plain functions: you
declare what you need as a typed argument, and the framework wires it up.

The test drove us to *routing* before storage — exactly as the Go book notes,
this is a smaller, test-led step than diving straight into a data layer.

## Separate the concern: a store

Our server shouldn't *know* the scores. That's a storage concern, and storage
is exactly the kind of dependency we've been injecting behind a trait since the
[Dependency injection](dependency-injection.md) and [Mocking](mocking.md)
chapters. Define what the server needs:

```rust,ignore
{{#include ../code/http-server/v3/src/lib.rs:store}}
```

The `Send + Sync` bound is new but not mysterious: it's the
[Sync chapter](sync.md)'s thread-safety marker traits, and axum requires it
because the runtime may touch our store from any of its worker threads. The
compiler enforces it, so a store that isn't thread-safe won't even build into a
handler.

Now `score` returns `Option<i32>`, and this is a genuine improvement over the
Go design worth pausing on. Go's version returns an `int` and treats `0` as
"not found" — which means a real player who has genuinely scored zero is
indistinguishable from a missing one. Our `Option` says exactly what it means:
`Some(score)` is a real score (even zero), `None` is a missing player. The
comma-ok-becomes-`Option` theme from the HashMaps chapter pays off directly in
cleaner HTTP semantics:

```rust,ignore
{{#include ../code/http-server/v3/src/lib.rs:code}}
```

Two new extractors combine in one handler. `State(store)` pulls out the shared
store we registered with `.with_state(store)` — axum's typed dependency
injection. The handler now returns `Response` rather than `String`, because it
produces *different* responses: a `match` on the `Option` yields either the
score (200 with a body) or `StatusCode::NOT_FOUND`. `.into_response()`
normalises both arms to the same `Response` type — the `IntoResponse` trait is
axum saying "lots of things can become a response; tell me which and I'll do
the rest".

The store lives behind `Arc<dyn PlayerStore>` — a shared, reference-counted
trait object, the [Sync chapter](sync.md)'s `Arc` plus the
[Structs chapter](structs-methods-and-traits.md)'s `dyn Trait`. Many handler
invocations on many threads share one store; `Arc` is what makes that sound.

The test's `StubPlayerStore` is our familiar test double, implementing the
trait over a `HashMap`:

```rust,ignore
{{#include ../code/http-server/v3/src/lib.rs:stub}}
```

## Storing wins

Now `POST`. First just drive out the route and status code — the Go book's
"commit sins" step — then assert on the store interaction. The store grows a
`record_win`:

```rust,ignore
{{#include ../code/http-server/v4/src/lib.rs:store}}
```

Note `record_win(&self, ...)` — a *shared* reference, not `&mut self`. That's
deliberate and necessary: our store is shared behind an `Arc`, which only ever
hands out `&`, so mutation has to happen through interior mutability. This is
precisely the [Sync chapter](sync.md)'s lesson, and the stub spy shows the
shape — a `Mutex` guarding the recorded calls:

```rust,ignore
{{#include ../code/http-server/v4/src/lib.rs:stub}}
```

The route gains a POST handler by chaining `.post(record_win)` onto the same
path, and the handler records the win and returns `202 Accepted`:

```rust,ignore
{{#include ../code/http-server/v4/src/lib.rs:code}}
```

The POST test asserts both the status *and* that the store was told to record
the right player — spying through the shared `Arc`, reading the `Mutex` back
after the request:

```rust,ignore
{{#include ../code/http-server/v4/src/lib.rs:post_test}}
```

## The real store

The stub proved the wiring; a real `InMemoryPlayerStore` makes it run. It's the
`Arc<Mutex<...>>`-shaped shared mutable state the Sync chapter promised is
*the* canonical form, plus the `entry(...).or_insert(0)` idiom from the
[HashMaps chapter](hashmaps.md):

```rust,ignore
{{#include ../code/http-server/v4/src/lib.rs:in_memory}}
```

Wire it into `main`, `cargo run`, and it's a working server. Two POSTs then a
GET for a player really does return `2`; an unknown player really does 404.
Everything the tests promised, live on a socket.

## Wrapping up

- **axum handlers are `async fn`s whose signatures declare their needs**
  through *extractors*: `Path` for URL parameters, `State` for injected
  dependencies. The return value becomes the response, via `IntoResponse`.
- **Async is the concurrency model for servers**: `#[tokio::main]` and
  `#[tokio::test]` start the runtime; `.await` drives futures; the runtime
  juggles many connections without a thread each.
- **Test HTTP without a network** using tower's `oneshot` — one request in,
  one response out, in memory, fast.
- **The store is injected behind a trait** (`Arc<dyn PlayerStore>`), so the
  server is tested against a stub and run against a real store, unchanged —
  the DI and Mocking chapters, now over HTTP. `Send + Sync` and
  `Arc<Mutex<_>>` are the Sync chapter's guarantees, made mandatory by the
  concurrent runtime.
- **`Option<i32>` beat Go's `int`-with-zero-sentinel**: "missing" and "scored
  zero" became distinct at the type level, and the HTTP status codes fell out
  cleanly.

Our server keeps scores in memory, so they vanish on restart. The next chapter
persists them — and starts turning this into a real application, with JSON, a
league table, and more routes.
