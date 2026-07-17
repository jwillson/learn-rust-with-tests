# Cancellation

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/cancellation)**

Software often kicks off long-running, resource-intensive work — usually on
another thread. If the action that caused it is abandoned (the user gives up,
the request times out, the deadline passes), you need a consistent way to
tell that work to stop. If you don't, your snappy application quietly fills
up with orphaned work, and you get the kind of performance problem that's
miserable to debug.

The Go book solves this with `context.Context`, a standard-library object
threaded through every function between an incoming request and its
outgoing calls. Rust's standard library has no equivalent — deliberately —
so this chapter does two things: builds the Rust-shaped replacement (it's
about fifteen lines, and every line is a concept you already own), and shows
the discipline that makes cancellation work in any language, because that
part is universal: **cancellation is cooperative**. Nothing can safely
murder a thread from the outside; the work has to *check* whether it should
stop, and the design question is who tells it.

Our example is the Go chapter's, minus the HTTP (which arrives later in the
book): a "server" that responds to a request by fetching data from
a potentially slow `Store` and writing it to the response.

## The happy path

Here's the starting code, using two tools from earlier chapters — a trait
for the store (so tests can substitute it) and `impl Write` for the
response, our [Dependency injection](dependency-injection.md) friend:

```rust,ignore
{{#include ../code/cancellation/v1/src/lib.rs:code}}
```

And a passing test, with a stub store:

```rust,ignore
{{#include ../code/cancellation/v1/src/lib.rs:test}}
```

The Go version of this chapter ends up hand-rolling a `SpyResponseWriter`
because its recorder can't answer "was anything written at all?". Our
response is a `Vec<u8>` — `response.is_empty()` will answer that question
for free when we need it later. Small dividend of the `Write` trait.

Now the real requirement: the user can cancel the request before the store
finishes, and when that happens the fetch should *stop working* and the
server should write *no response*.

## First, we need a way to say "stop"

The signal has to travel between threads — the canceller lives on one, the
worker on another. After the [Sync](sync.md) chapter you already know the
shape this must take: shared state, thread-safe, behind an `Arc`. We'll
build a `CancellationToken`, test-first.

## Write the test first

```rust,ignore
{{#include ../code/cancellation/v2/src/lib.rs:token_tests}}
```

Four tests, four promises: a fresh token isn't cancelled; cancelling flips
it; *clones share the signal* (that's the crucial one — the canceller and
the worker each hold a clone of the same token); and it works across
threads.

## Write enough code to make it pass

```rust,ignore
{{#include ../code/cancellation/v2/src/lib.rs:token}}
```

One new tool here: `AtomicBool`. The Sync chapter's table said "for shared
mutation across threads, use `Mutex`" — atomics are the third column we
didn't need then: for primitive values (bools, integers), the hardware can
do the synchronisation itself, no lock required. `store` and `load` are its
setter and getter; the `Ordering` argument tunes how this atomic interacts
with *other* memory operations, a genuinely deep topic — for an independent
flag like ours, `Relaxed` is correct, and we'll leave the deeper orderings
for the day you need them.

Note `#[derive(Clone)]`: cloning the token clones the `Arc` *handle*, not
the flag — exactly like the Sync chapter's counter — which is what makes
the "clones share the signal" test pass. All four green.

## Threading the signal through

Who checks the token? The Go chapter takes a wrong turn first — giving the
store a `Cancel()` method and making the *server* decide when to call it —
and then talks itself out of it: the server shouldn't micromanage the
store's cleanup, and the store's own downstream calls need cancelling too.
The conclusion (Google's rule for Go, and the reason `context` exists) is:
**pass the cancellation signal down the call chain, and let each layer be
responsible for stopping itself.** We can go straight to the right design,
because in a synchronous call the wrong one doesn't even work — while
`respond` is blocked inside `store.fetch()`, the server can't *do*
anything about a cancellation. Only the code doing the work can check.

So fetching becomes fallible: either you get the data, or you learn it was
cancelled. We know how to say that:

```rust,ignore
{{#include ../code/cancellation/v2/src/lib.rs:code}}
```

The `Cancelled` error is the unit-struct pattern from the
[Errors chapter](errors-and-result.md). And the compiler now walks us
through the consequences, which is what changing a trait is *supposed* to
feel like. The old stub store no longer compiles:

```text
error[E0050]: method `fetch` has 1 parameter but the declaration in trait `Store::fetch` has 2
   |
 6 |     fn fetch(&self, cancel: &CancellationToken) -> Result<String, Cancelled>;
   |              --------------------------------- trait requires 2 parameters
...
14 |     fn fetch(&self) -> String {
   |              ^^^^^ expected 2 parameters, found 1
```

And inside `respond`, the tempting one-liner `write!(writer, "{}",
store.fetch(cancel)?)` gets a lesson about `?` we've been saving:

```text
error[E0277]: `?` couldn't convert the error to `std::io::Error`
   |
16 |     write!(writer, "{}", store.fetch(cancel)?)
   |                                -------------^ the trait `From<Cancelled>` is not implemented for `std::io::Error`
   |
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
```

`?` doesn't just propagate errors — it *converts* them, via `From`, into
the calling function's error type. We could teach `io::Error` to absorb
a `Cancelled`... but read the requirement again: cancellation isn't an
I/O failure to report, it's a signal to quietly write nothing. The `match`
in `respond` above says exactly that: data gets written, `Cancelled` gets
swallowed, real I/O errors still propagate from `write!`. (File the `From`
mechanism away — it's how multi-error-type functions stay ergonomic, and
it'll return when our applications grow real error hierarchies.)

## The spy store

The test double needs to act like a real slow fetch: build the response
character by character, checking the token as it goes — this is what
"cooperative" looks like in the flesh:

```rust,ignore
{{#include ../code/cancellation/v2/src/lib.rs:spy}}
```

## The tests

The happy path, plus the new one: schedule a cancellation 5 milliseconds
from now on another thread (our `time.AfterFunc`), call `respond`, and
assert nothing was written:

```rust,ignore
{{#include ../code/cancellation/v2/src/lib.rs:test}}
```

Worth proving the spy's token check is load-bearing: delete the
`if cancel.is_cancelled()` block and the compiler immediately flags the
now-unused `cancel` parameter — the signature stops being true, just like
the Select chapter's unused `timeout` — and the test fails honestly:

```text
test tests::writes_no_response_when_the_request_is_cancelled ... FAILED

thread 'tests::writes_no_response_when_the_request_is_cancelled' panicked at src/lib.rs:148:9:
a response should not have been written
```

Restore the check and everything is green: six tests, and a server that
abandons cancelled work in ~5ms instead of grinding through all 120.

## Wrapping up

### What we've covered

- **Cancellation is cooperative.** The worker checks a shared signal at
  its own safe points and stops itself — returning an error so callers
  know the result never arrived. Nothing external can (or should) kill it.
- **A cancellation token is just shared state**: `Arc<AtomicBool>`, clone
  it to everyone involved. Atomics are the lock-free tool for primitive
  shared values; `Relaxed` ordering suffices for independent flags.
- **Pass the signal down the call chain** and let each layer stop itself —
  Google's `context` rule, minus the `context`. The signature
  `fetch(&self, cancel: &CancellationToken) -> Result<String, Cancelled>`
  tells every reader this operation is long, stoppable, and honest about
  it.
- **`?` converts errors via `From`** — and when an "error" is really
  a signal (like cancellation), a `match` that handles it locally beats
  converting it.

### What about `context.Value`?

Go's `context` doubles as a grab-bag of request-scoped values, and the Go
book's verdict is blunt: *"if a function needs some values, put them as
typed parameters rather than trying to fetch them from `context.Value`."*
Rust's version of that advice is shorter: there is no bag. Pass parameters.
The language never offered the temptation, which is one reason it never
grew a `Context` — the only job left was the cancel signal, and as we've
seen, fifteen lines cover it.

### The async footnote

This chapter is honest for threaded Rust, but you should know the word on
the street: much real-world Rust networking uses `async` (with the
[tokio](https://tokio.rs) runtime), where cancellation has an extra,
sharper edge — dropping a future stops polling it, and
`tokio_util::sync::CancellationToken` exists ready-made, with the same
shape as the one we built (`cancel()`, checks, clones sharing a signal).
The design discipline transfers unchanged: make long work interruptible,
propagate the signal, return "cancelled" as a real result.
