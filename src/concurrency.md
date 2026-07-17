# Concurrency

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/concurrency)**

Here's the setup: a colleague has written a function, `check_websites`, that
checks the status of a list of URLs.

```rust,ignore
{{#include ../code/concurrency/v1/src/lib.rs:code}}
```

It returns a `HashMap` of each URL checked to a boolean value: `true` for
a good response, `false` for a bad response.

You also have to pass in a `WebsiteChecker` — a function which takes a single
URL and returns a boolean. `type WebsiteChecker = fn(&str) -> bool` is a *type
alias*: it gives the function-pointer type `fn(&str) -> bool` a name, the same
way Go's `type WebsiteChecker func(string) bool` does. We could have taken
a generic closure with an `F: Fn(&str) -> bool` bound like we did in the
[Mocking](mocking.md) chapter, but a plain function pointer keeps the
signatures in this chapter simple, and it's all our tests need.

Using [dependency injection](dependency-injection.md) has allowed them to test
the function without making real HTTP calls, making it reliable and fast.

Here's the test they've written:

```rust,ignore
{{#include ../code/concurrency/v1/src/lib.rs:test}}
```

The function is in production and being used to check hundreds of websites.
But your colleague has started to get complaints that it's slow, so they've
asked you to help speed it up.

## Write a test

Let's use a benchmark to test the speed of `check_websites` so that we can see
the effect of our changes. We set criterion up exactly as we did in the
[Iteration](iteration.md) chapter — `criterion` in `[dev-dependencies]` and
a `[[bench]]` section with `harness = false` — and put this in
`benches/check_websites.rs`:

```rust,ignore
{{#include ../code/concurrency/v1/benches/check_websites.rs:bench}}
```

The benchmark tests `check_websites` using a slice of one hundred urls and
uses a new fake implementation of `WebsiteChecker`.
`slow_stub_website_checker` is deliberately slow. It uses
`std::thread::sleep` to wait exactly twenty milliseconds and then it returns
`true`.

One new thing: the `criterion_group!` macro's longer form lets us configure
the run. Criterion's default is 100 samples, but each call to our function is
going to take around two seconds (100 websites × 20ms), so we dial
`sample_size` down to 10 and give it a 30-second budget — otherwise we'd be
waiting minutes for an answer we can get in half a minute.

When we run the benchmark with `cargo bench -p concurrency-v1`:

```text
Benchmarking check_websites: Collecting 10 samples in estimated 40.383 s (20 iterations)
check_websites          time:   [2.0189 s 2.0205 s 2.0222 s]
```

`check_websites` has been benchmarked at about two seconds — one hundred
20-millisecond checks, one after another. Let's try and make this faster.

### The tea-making principle

Now we can finally talk about concurrency which, for the purposes of the
following, means "having more than one thing in progress". This is something
that we do naturally every day.

For instance, this morning I made a cup of tea. I put the kettle on and then,
while I was waiting for it to boil, I got the milk out of the fridge, got the
tea out of the cupboard, found my favourite mug, put the teabag into the cup
and then, when the kettle had boiled, I put the water in the cup.

What I *didn't* do was put the kettle on and then stand there blankly staring
at the kettle until it boiled, then do everything else once the kettle had
boiled.

If you can understand why it's faster to make tea the first way, then you can
understand how we will make `check_websites` faster. Instead of waiting for
a website to respond before sending a request to the next website, we will
tell our computer to make the next request while it is waiting.

Normally when we call a function we wait for it to return — the operation
*blocks* us until it's finished. An operation that does not block can run in
a separate *thread* of execution. Think of a thread as reading down the page
of code from top to bottom, going "inside" each function when it gets called
to read what it does. When a separate thread starts, it's like another reader
begins reading inside the function, leaving the original reader to carry on
going down the page.

To start a new thread we call `std::thread::spawn`, handing it a closure to
run — where Go writes `go doSomething()`, Rust writes
`thread::spawn(|| do_something())`.

So let's try the obvious thing — spawn a thread per website, and let each one
write its result into the map:

```rust,ignore
pub fn check_websites(checker: WebsiteChecker, urls: &[&str]) -> HashMap<String, bool> {
    let mut results = HashMap::new();

    for &url in urls {
        std::thread::spawn(|| {
            results.insert(url.to_string(), checker(url));
        });
    }

    results
}
```

Each iteration of the loop starts a new thread, concurrent with the current
one, and each thread will add its result to the results map. It's a faithful
translation of what the equivalent Go code would be.

But when we run `cargo test`... it doesn't run at all:

```text
error[E0373]: closure may outlive the current function, but it borrows `results`, which is owned by the current function
  --> src/lib.rs:9:28
   |
 9 |         std::thread::spawn(|| {
   |                            ^^ may outlive borrowed value `results`
10 |             results.insert(url.to_string(), checker(url));
   |             ------- `results` is borrowed here
   |
help: to force the closure to take ownership of `results` (and any other referenced variables), use the `move` keyword
   |
 9 |         std::thread::spawn(move || {
   |                            ++++

error[E0499]: cannot borrow `results` as mutable more than once at a time
  --> src/lib.rs:9:28
   |
 9 |           std::thread::spawn(|| {
   |           -                  ^^ `results` was mutably borrowed here in the previous iteration of the loop
   |  _________|
   | |
10 | |             results.insert(url.to_string(), checker(url));
   | |             ------- borrows occur due to use of `results` in closure
11 | |         });
   | |__________- argument requires that `results` is borrowed for `'static`

error[E0521]: borrowed data escapes outside of function
  --> src/lib.rs:9:9
   |
 5 |   pub fn check_websites(checker: WebsiteChecker, urls: &[&str]) -> HashMap<String, bool> {
   |                                                  ----    - let's call the lifetime of this reference `'1`
   |                                                  |
   |                                                  `urls` is a reference that is only valid in the function body
...
 9 | /         std::thread::spawn(|| {
10 | |             results.insert(url.to_string(), checker(url));
11 | |         });
   | |          ^
   | |__________`urls` escapes the function body here
   |            argument requires that `'1` must outlive `'static`
```

(Plus a couple more in the same vein.)

### A quick aside into the concurrency universe...

It's worth pausing to appreciate what just happened, because this is the
moment where Rust's whole ownership system pays for itself.

In Go, this exact code **compiles and ships**. Then it fails at runtime,
differently on different days:

1. Most days the test fails with an *empty map* — `CheckWebsites` returns
   before any goroutine has had time to write its result.
2. The traditional "fix" is to add `time.Sleep(2 * time.Second)` before the
   return and hope two seconds is enough. Now the test passes... *if you're
   lucky*.
3. If you're unlucky, two goroutines write to the map at the same instant and
   the whole program dies with `fatal error: concurrent map writes`.
4. To actually understand the bug you have to know to re-run your tests with
   Go's (excellent, but opt-in) race detector, `go test -race`, and it can
   only report races that *actually happened* on that particular run.

Every one of those runtime disasters is one of the compile errors above:

- **E0373** ("closure may outlive the current function") is the empty-map
  bug. The spawned threads can keep running after `check_websites` returns,
  so they'd be writing into a map that no longer exists. Rust won't let the
  function return while borrowed data is still at risk.
- **E0499** ("`results` was mutably borrowed here in the previous iteration
  of the loop") is `fatal error: concurrent map writes`. Two threads holding
  a mutable reference to the same map at the same time *is* a data race, and
  the borrow checker's oldest rule — one mutable borrow at a time — forbids
  it before it can happen.
- **E0521** ("`urls` escapes the function body") says the threads might
  outlive the slice they're reading URLs from.

And note what there *isn't*: a sleep hack. Adding
`thread::sleep(Duration::from_secs(2))` before the return changes these
errors not at all, because they aren't about timing — they're about who is
allowed to touch what. The borrow checker is a race detector that is always
on, runs at compile time, and reports races that *could* happen, not just
ones that did.

### ... and we're back.

The compiler suggested `move`, so let's do as we're told. `move` makes the
closure take ownership of the variables it uses instead of borrowing them:

```rust,ignore
        std::thread::spawn(move || {
            results.insert(url.to_string(), checker(url));
        });
```

```text
error[E0382]: use of moved value: `results`
  --> src/lib.rs:14:5
   |
 6 |     let mut results = HashMap::new();
   |         ----------- move occurs because `results` has type `HashMap<String, bool>`, which does not implement the `Copy` trait
 8 |     for &url in urls {
   |     ---------------- inside of this loop
 9 |         std::thread::spawn(move || {
   |                            ------- value moved into closure here, in previous iteration of loop
14 |     results
   |     ^^^^^^^ value used here after move
```

Of course. Only one owner at a time — the first thread took the map with it,
so there is nothing left for the second thread, let alone for us to return.
The compiler suggests cloning the map into each closure, but that's no good
either: a hundred threads each writing into their own private copy, every
copy thrown away at the end.

This is a genuine dead end, and it's the same dead end the Go version was in
— we just found out at compile time instead of in production. What we need is
what concurrent code always needs: a way for many threads to *produce*
results and one place to *consume* them, without sharing the map.

### Channels

Rust's standard library has exactly this, and its name is practically the
design document: `std::sync::mpsc` — **m**ulti-**p**roducer,
**s**ingle-**c**onsumer. A channel is a pair of a `Sender` and a `Receiver`:
any number of threads can `send` values into their (cloned) `Sender`s, and
one thread pulls them out of the `Receiver`, one at a time.

That's the plan: every spawned thread gets a `Sender` and sends back a
`(url, result)` pair; the main thread receives the pairs and inserts them
into the map. Many producers, one consumer — nobody shares the `HashMap`.

One more tool and we can write it. Plain `thread::spawn` demands its closure
own everything for `'static` — that was E0521, the thread that might outlive
`urls`. But `std::thread::scope` creates a *scope* that guarantees every
thread spawned inside it has finished before the scope returns. Because the
threads provably can't outlive the function, they're allowed to borrow its
locals — `urls` included.

```rust,ignore
{{#include ../code/concurrency/v2/src/lib.rs:code}}
```

Walking through it:

- `mpsc::channel()` gives us the `(sender, receiver)` pair. The channel
  carries `(url, bool)` tuples — where Go's version declared a little
  `result` struct with anonymous fields, a tuple is Rust's way of saying
  "a pair of values that don't need naming".
- Inside the scope, the loop spawns one thread per URL. Each thread gets its
  own clone of the sender (`Sender` is designed to be cloned — that's the
  "multi-producer" part) and `move`s it, along with its `url`, into the
  closure. Moving a clone is fine; it was made to be given away.
- `sender.send((url, checker(url)))` does the check on the spawned thread
  and sends the result back. This is the send side of Go's
  `resultChannel <- result{url, wc(url)}`.
- Back on the main thread, `receiver.recv()` blocks until a value arrives —
  the receive side of Go's `r := <-resultChannel`. We receive exactly
  `urls.len()` times, inserting each pair into the map. Only the main thread
  ever touches `results`, so there is nothing to race over.

We have used concurrency for the part of the code that we wanted to make
faster, while making sure that the part that cannot happen simultaneously
still happens linearly. The hundred checks run at once; the hundred map
inserts happen one at a time, on one thread, as the results drain out of the
channel.

`cargo test`:

```text
running 1 test
test tests::checks_all_the_websites ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

And the benchmark — remember criterion compares against the previous run:

```text
Benchmarking check_websites: Collecting 10 samples in estimated 30.406 s (1375 iterations)
check_websites          time:   [22.120 ms 22.139 ms 22.166 ms]
                        change: [−98.906% −98.905% −98.903%] (p = 0.00 < 0.05)
                        Performance has improved.
```

From 2.02 seconds to 22 milliseconds — about a hundred times as fast, which
is what you'd hope for from running a hundred 20ms waits at the same time
instead of back to back. A great success.

## Wrapping up

This exercise has been a little lighter on the TDD than usual. In a way we've
been taking part in one long refactoring of the `check_websites` function;
the inputs and outputs never changed, it just got faster. But the tests we
had in place, as well as the benchmark we wrote, allowed us to refactor
`check_websites` in a way that maintained confidence that the software was
still working, while demonstrating that it had actually become faster.

In making it faster we learned about

- **threads**, the basic unit of concurrency in Rust, which let us manage
  more than one website check request — and `std::thread::scope`, which lets
  threads borrow from the function that spawned them by guaranteeing they
  finish before it returns.
- **`move` closures**, which hand ownership of captured variables to the
  thread that will use them.
- **channels** (`std::sync::mpsc`), to organize and control the
  communication between threads, allowing us to avoid a *data race* — many
  producers sending into one consumer.
- **the borrow checker as an always-on race detector**. Every failure mode
  the equivalent Go code exhibits at runtime — the empty map, the flaky
  sleep-and-hope fix, `fatal error: concurrent map writes` — arrived here as
  a compile error, before the code could ever run. This is what people mean
  by *fearless concurrency*: not that concurrent code is easy to design, but
  that the class of bug which is hardest to reproduce, the data race, simply
  doesn't survive compilation.

### Make it fast

One formulation of an agile way of building software, often misattributed to
Kent Beck, is:

> [Make it work, make it right, make it fast](http://wiki.c2.com/?MakeItWorkMakeItRightMakeItFast)

Where 'work' is making the tests pass, 'right' is refactoring the code, and
'fast' is optimizing the code to make it, for example, run quickly. We can
only 'make it fast' once we've made it work and made it right. We were lucky
that the code we were given was already demonstrated to be working, and
didn't need to be refactored. We should never try to 'make it fast' before
the other two steps have been performed because

> [Premature optimization is the root of all evil](http://wiki.c2.com/?PrematureOptimization)
> -- Donald Knuth
