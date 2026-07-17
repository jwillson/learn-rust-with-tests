# Select

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/select)**

You have been asked to make a function called `racer` which takes two website
addresses and "races" them by connecting to each one and returning the address
which responded first. If neither of them respond within 10 seconds then it
should return an error.

For this, we will be using:

- `std::net` to make the network calls.
- `std::net::TcpListener` to help us test them.
- threads.
- channels — and a new trick on the receiving end — to synchronise processes.

A note before we start: the Go version of this chapter reaches for `net/http`,
because Go ships an HTTP client in its standard library. Rust doesn't — HTTP
lives in excellent third-party crates which we'll meet when we build a real
application later in the book. But this chapter's lesson isn't about HTTP,
it's about *racing* — who answers on the wire first — and for that, plain TCP
from the standard library is all we need. Our racers will take addresses like
`"quii.dev:80"`, open a TCP connection, and wait for the server's response.

## Write the test first

Let's start with something naive to get us going.

```rust,ignore
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_the_faster_of_two_servers() {
        let slow_addr = "facebook.com:80";
        let fast_addr = "quii.dev:80";

        let want = fast_addr;
        let got = racer(slow_addr, fast_addr);

        assert_eq!(got, want);
    }
}
```

We know this isn't perfect and has problems, but it's a start. It's important
not to get too hung-up on getting things perfect first time.

## Try to run the test

```text
error[E0425]: cannot find function `racer` in this scope
  --> src/lib.rs:11:19
   |
11 |         let got = racer(slow_addr, fast_addr);
   |                   ^^^^^ not found in this scope
```

## Write the minimal amount of code for the test to run and check the failing test output

```rust,ignore
pub fn racer(a: &str, b: &str) -> &str {
    ""
}
```

```text
error[E0106]: missing lifetime specifier
 --> src/lib.rs:1:35
  |
1 | pub fn racer(a: &str, b: &str) -> &str {
  |                 ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
help: consider introducing a named lifetime parameter
  |
1 | pub fn racer<'a>(a: &'a str, b: &'a str) -> &'a str {
  |             ++++     ++          ++          ++
```

Interesting! We haven't even got to the failing assertion yet and the compiler
has a question for us, and it's a genuinely good one: *this function returns
a borrowed string — borrowed from whom?*

Every function we've written so far that returned a reference took exactly one
reference in, so the compiler could work out the answer for itself (the output
must borrow from the only input there is). With *two* inputs, it can't guess —
the winner might be `a` or might be `b` — so it asks us to say. The suggested
fix, `<'a>`, declares a named *lifetime parameter* and uses it to say "the
returned `&str` borrows from one of these two arguments, so it's valid as long
as they both are." There's a whole chapter on lifetimes later; for now, we can
do exactly what the compiler told us to:

```rust,ignore
pub fn racer<'a>(a: &'a str, b: &'a str) -> &'a str {
    ""
}
```

Now the test runs, and fails, which is what we wanted:

```text
thread 'tests::returns_the_faster_of_two_servers' panicked at src/lib.rs:17:9:
assertion `left == right` failed
  left: ""
 right: "quii.dev:80"
```

## Write enough code to make it pass

```rust,ignore
use std::io::Read;
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub fn racer<'a>(a: &'a str, b: &'a str) -> &'a str {
    let start_a = Instant::now();
    ping(a);
    let a_duration = start_a.elapsed();

    let start_b = Instant::now();
    ping(b);
    let b_duration = start_b.elapsed();

    if a_duration < b_duration { a } else { b }
}

fn ping(addr: &str) {
    if let Ok(mut stream) = TcpStream::connect(addr) {
        let mut response = String::new();
        let _ = stream.read_to_string(&mut response);
    }
}
```

For each address:

1. We use `Instant::now()` to record just before we try to reach the address —
   `Instant` is the standard library's monotonic clock, the equivalent of Go's
   `time.Now()`.
2. `TcpStream::connect` opens a TCP connection, and `read_to_string` waits
   until the server has said everything it has to say and closed the
   connection. We're not actually interested in the response — only in how
   long it took — so we read it into a string and throw it away. (The
   `let _ =` tells the compiler we know `read_to_string` returns a `Result`
   we're choosing to ignore; a failed read just means a slow racer, not a
   broken one.)
3. `start.elapsed()` gives us a `Duration` — `time.Since` in Go.

Once we have done this we simply compare the durations to see which is the
quickest.

### Problems

This may or may not make the test pass for you. The problem is we're reaching
out to real websites to test our own logic.

In the [mocking](mocking.md) and [dependency injection](dependency-injection.md)
chapters, we covered why we don't want to be relying on external services to
test our code:

- it's slow
- it's flaky
- we can't test edge cases

Go's standard library has `httptest` for spinning up mock HTTP servers. We can
build the same thing with `TcpListener` in a handful of lines — and learn its
best trick along the way. Put this helper inside the `tests` module:

```rust,ignore
{{#include ../code/select/v1/src/lib.rs:make_delayed_server}}
```

Three things worth pausing on:

- **Port `0` means "you pick".** Binding to `127.0.0.1:0` asks the operating
  system for any free port, and `local_addr()` tells us which one we got.
  This is exactly how Go's `httptest.NewServer` finds an open port, and it
  means we can run many tests in parallel without them fighting over
  addresses.
- The spawned thread sits in a loop accepting connections; for each one it
  sleeps for `delay` and then writes a minimal HTTP response.
  (`.flatten()` on the `incoming()` iterator quietly skips any connections
  that failed to establish — `incoming()` yields `Result`s.)
- Where the Go version needs `defer slowServer.Close()` so the server doesn't
  keep listening after the test, we need... nothing. This pattern is called
  RAII — when a value goes out of scope, Rust runs its destructor, and
  `TcpListener`'s destructor closes the socket. Go's `defer` is a manual
  convention; `Drop` is automatic and impossible to forget.

Now the test uses servers we control:

```rust,ignore
{{#include ../code/select/v1/src/lib.rs:test}}
```

The slow server takes 20 milliseconds to answer; the fast one answers
immediately. If you re-run the test it will definitely pass now and will be
faster. Play with these delays to deliberately break the test.

## Refactor

We have some duplication in our production code — measure a start time, ping,
take the elapsed time, twice. Extracting it:

```rust,ignore
{{#include ../code/select/v1/src/lib.rs:code}}
```

This DRY-ing up makes our `racer` code a lot easier to read, and it's
a reasonable solution given the tools we've covered so far. But we can make
the solution simpler.

### Synchronising processes

- Why are we testing the speeds of the websites one after another when Rust is
  great at concurrency? We should be able to check both at the same time.
- We don't really care about *the exact response times* of the requests, we
  just want to know which one comes back first.

The Go book solves this with `select`, a Go language construct that waits on
*multiple channels* at once and runs the case belonging to whichever channel
delivers first. Rust's standard library doesn't have a `select` statement —
and for this problem it doesn't need one, because of something we learned last
chapter: an mpsc channel is **multi**-producer. Go's shape is *two channels,
one racer each, and a construct to watch both*. Rust's shape turns that inside
out: *one channel, two racers sending into it* — and whoever's message lands
first is simply the first thing `recv()` returns. The race is built into the
channel.

```rust,ignore
{{#include ../code/select/v2/src/lib.rs:code}}
```

#### `ping`

`ping` spawns a thread that connects to the address, waits out the response,
and then sends the address back down the channel as its way of saying "done".

A few deliberate choices in here:

- **Why `thread::spawn` and not the scoped threads from last chapter?**
  Because a scope *waits for every thread it spawned* before it lets the
  function return — that guarantee was exactly why scoped threads could
  borrow. But here the whole point is to return as soon as the *winner*
  reports in, while the loser may still be mid-request. So we use free-range
  `thread::spawn`, and pay its price: the thread can't borrow `addr` from the
  caller, it needs to own its data. `addr.to_string()` makes an owned copy
  for the thread to take with it.
- **`let _ = sender.send(addr)`** — send can fail. Once `racer` has received
  the winner and returned, the channel's `Receiver` is dropped; when the
  losing thread finishes its request and tries to report in, there is nobody
  left to listen, and `send` returns an `Err` to say so. That's not a bug —
  it's exactly what we expect to happen to the loser — so we explicitly
  discard the `Result`. (If we'd written `.unwrap()`, the losing thread would
  panic quietly in the background: harmless here, but sloppy.)

#### One channel, many racers

Back in `racer`: we make one channel, hand a clone of the `sender` to the
first ping and the original to the second, then block on `receiver.recv()`.
The first thread to finish its request sends its address; `recv()` hands it
to us; we're done. We don't even look at the second message — we compare the
winning address against `a` so we can return the caller's own borrowed `&str`
(that lifetime annotation is still doing its job) and get out.

After these changes, the intent behind our code is very clear and the
implementation is actually simpler — no clocks, no duration arithmetic, just
"first one back wins".

### Timeouts

Our final requirement was to return an error if `racer` takes longer than 10
seconds.

## Write the test first

If `racer` can fail, we know from the [errors chapter](errors-and-result.md)
what its return type should be: a `Result`. Let's update the happy-path test
to expect one, and add a sad-path test:

```rust,ignore
    #[test]
    fn returns_an_error_if_a_server_does_not_respond_in_time() {
        let addr = make_delayed_server(Duration::from_secs(11));

        let result = racer(&addr, &addr);

        assert!(result.is_err(), "expected an error but didn't get one");
    }
```

## Try to run the test

```text
error[E0308]: mismatched types
  --> src/lib.rs:97:25
   |
97 |         assert_eq!(got, want);
   |                         ^^^^ expected `Result<&str, RacerError>`, found `&str`
   |
   = note:   expected enum `Result<&str, RacerError>`
           found reference `&str`
help: try wrapping the expression in `Ok`
   |
97 |         assert_eq!(got, Ok(want));
   |                         +++    +
```

This is the Rust version of Go's "2 variables but Racer returns 1 value"
moment — the compiler telling us that changing a function's contract means
every caller, tests included, has to acknowledge the change. Rather than
wrapping `want` in `Ok` we'll have the happy test `expect` success, which
both unwraps the value and documents that an error here would be a test
failure.

## Write the minimal amount of code for the test to run and check the failing test output

We need an error type — ours holds the two addresses we gave up waiting for,
and follows the same pattern as the errors chapter: `Display` for the human,
`std::error::Error` to mark it as an error:

```rust,ignore
{{#include ../code/select/v3/src/lib.rs:error}}
```

Then the minimal change to `racer` is to wrap what it already does in `Ok`:

```rust,ignore
pub fn racer<'a>(a: &'a str, b: &'a str) -> Result<&'a str, RacerError> {
    let (sender, receiver) = mpsc::channel();

    ping(a, sender.clone());
    ping(b, sender);

    let winner = receiver.recv().unwrap();
    if winner == a { Ok(a) } else { Ok(b) }
}
```

Everything compiles, the happy test passes... and the sad test hangs for 11
seconds before failing, because nothing in our code knows how to give up yet.

### Slow tests

An 11-second wait for such a simple bit of logic doesn't feel great — and it
would sit in our test suite forever, taxing every single `cargo test` run
until the end of the project. Before paying that, let's *listen to the test*:

- Do we care that the timeout is *exactly ten seconds* to prove the give-up
  logic works? No — we care that a configurable deadline is enforced.
- The requirements *were* explicit about 10 seconds, so that default should
  live somewhere and be visible.

The design that serves both: a `configurable_racer` that takes the timeout as
a parameter — which tests can set to something tiny — and a `racer` that
simply calls it with the official 10 seconds. Here are the final tests:

```rust,ignore
{{#include ../code/select/v3/src/lib.rs:test}}
```

The sad test races a server that takes 25 milliseconds against a deadline of
20 — it fails fast now:

```text
warning: unused variable: `timeout`

test tests::returns_the_faster_of_two_servers ... ok
test tests::returns_an_error_if_a_server_does_not_respond_in_time ... FAILED

thread 'tests::returns_an_error_if_a_server_does_not_respond_in_time' panicked at src/lib.rs:95:9:
expected an error but didn't get one
```

Look at that warning. We wrote `configurable_racer(a, b, timeout)` as
a signature, but nothing in the body uses `timeout` yet — the compiler is
pointing directly at the lie in our code.

## Write enough code to make it pass

Where Go reaches for `select` with a `time.After` case, Rust's receiver has
the tool built in: [`recv_timeout`](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.recv_timeout).
It's `recv` with a deadline — it blocks until *either* a message arrives
(`Ok`) *or* the timeout passes (`Err`). Which is precisely our race, so the
whole implementation is one `match`:

```rust,ignore
{{#include ../code/select/v3/src/lib.rs:code}}
```

If either ping reports in before the deadline, we get `Ok(winner)` and return
the winning address. If the deadline arrives first, `recv_timeout` returns an
`Err`, which we translate into our own `RacerError` naming the two addresses
that let us down. Notice there's no way to forget the timeout arm: `match` on
the `Result` *must* handle both cases or the code won't compile — compare
Go, where deleting the `time.After` case leaves a `select` that blocks
forever and no one warns you.

```text
running 2 tests
test tests::returns_the_faster_of_two_servers ... ok
test tests::returns_an_error_if_a_server_does_not_respond_in_time ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Wrapping up

### Where Go's `select` went

Go's `select` waits on many channels and takes whichever speaks first. This
chapter's Rust translation dissolved the construct into the channel itself:

- **Racing many producers**: give each contestant a clone of one channel's
  `Sender` — the first `send` wins `recv()`. Multi-producer, single-consumer
  is the race, no extra syntax required.
- **Timing out**: `recv_timeout` covers the overwhelmingly common
  `select`-with-`time.After` pattern, and returns a `Result` so the compiler
  makes you decide what a timeout means.

That covers this chapter's needs honestly, but it isn't the whole story: if
you genuinely need to wait on several *different* channels — different types,
different sources, first-of-any — the community-standard
[`crossbeam-channel`](https://docs.rs/crossbeam-channel) crate provides
a `select!` that is Go's construct in Rust. Worth knowing about; not worth
a dependency today.

### What else we learned

- **`TcpListener::bind("127.0.0.1:0")`** — the OS-picks-a-port trick behind
  every good test server, Go's `httptest.NewServer` included. A dozen lines
  of standard library gave us controllable, parallel-safe fake servers.
- **RAII instead of `defer`**: our servers and channels clean themselves up
  when they go out of scope. `Drop` is the automatic version of the `Close()`
  you might one day forget to defer.
- **Our first lifetime annotation**: `racer<'a>(a: &'a str, b: &'a str) ->
  &'a str` answered the compiler's very reasonable question — *borrowed from
  whom?* — when one returned reference could come from either of two inputs.
  The Lifetimes chapter will make a proper meal of this; today it was one
  compiler suggestion, applied verbatim.
- **Threads that outlive their spawner own their data** — `thread::spawn`
  plus `to_string`, versus last chapter's borrowing scoped threads. Choosing
  between them is a design decision: *must everyone finish before we return*
  (scope) or *do we leave the losers behind* (spawn)?
