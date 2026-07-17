# Sync

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/sync)**

We want to make a counter which is safe to use concurrently.

The Go version of this chapter has a simple plan: build an unsafe counter,
verify it works single-threaded, then *exercise its unsafeness* with a test
that hammers it from a hundred goroutines and watch the wrong number come out.
Our plan is the same — but you already know from the
[Concurrency](concurrency.md) chapter how the middle act ends in Rust. An
unsafe counter doesn't produce wrong numbers here; it produces compile errors.
The interesting part is what the errors teach us, and which tool finally
satisfies the compiler.

## Write the test first

We want our API to give us a method to increment the counter and then retrieve
its value.

```rust,ignore
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incrementing_the_counter_3_times_leaves_it_at_3() {
        let counter = Counter::new();
        counter.inc();
        counter.inc();
        counter.inc();

        assert_eq!(counter.value(), 3);
    }
}
```

## Try to run the test

```text
error[E0433]: cannot find type `Counter` in this scope
 --> src/lib.rs:7:23
  |
7 |         let counter = Counter::new();
  |                       ^^^^^^^ use of undeclared type `Counter`
```

## Write the minimal amount of code for the test to run and check the failing test output

Let's define `Counter` and give it the conventional `new` constructor:

```rust,ignore
pub struct Counter;

impl Counter {
    pub fn new() -> Counter {
        Counter
    }
}
```

Try again and it fails with the following (once for each call):

```text
error[E0599]: no method named `inc` found for struct `Counter` in the current scope
  --> src/lib.rs:16:17
   |
 1 | pub struct Counter;
   | ------------------ method `inc` not found for this struct
...
16 |         counter.inc();
   |                 ^^^ method not found in `Counter`
```

So to finally make the test run we can define those methods as stubs:

```rust,ignore
impl Counter {
    pub fn new() -> Counter {
        Counter
    }

    pub fn inc(&self) {}

    pub fn value(&self) -> u32 {
        0
    }
}
```

It should now run and fail:

```text
assertion `left == right` failed
  left: 0
 right: 3
```

## Write enough code to make it pass

We need to keep some state for the counter in our datatype and then increment
it on every `inc` call:

```rust,ignore
{{#include ../code/sync/v1/src/lib.rs:code}}
```

Two things changed beyond adding the field. `inc` now takes `&mut self` — it
mutates the counter, and in Rust a method that mutates must say so in its
signature. And the compiler immediately holds our test to the same standard:

```text
error[E0596]: cannot borrow `counter` as mutable, as it is not declared as mutable
  --> src/lib.rs:25:13
   |
25 |         let counter = Counter::new();
   |             ^^^^^^^ not mutable
26 |         counter.inc();
   |         ------- cannot borrow as mutable
```

`let mut counter` in the test, and we're green.

## Refactor

There's not a lot to refactor, but given we're going to write more tests
around `Counter` we'll write a small assertion helper so the tests read a bit
clearer — with `#[track_caller]` so failures point at the test, not the
helper, just like we've done since the Hello, World chapter:

```rust,ignore
{{#include ../code/sync/v1/src/lib.rs:helper}}
```

## Next steps

That was easy enough, but now we have a requirement that the counter must be
safe to use in a concurrent environment. We will need to write a failing test
to exercise this.

## Write the test first

We want 1000 increments, each from its own thread, and then to assert on the
total. The Go version reaches for `sync.WaitGroup` here — `Add(1000)` before
the loop, `Done()` in every goroutine, `Wait()` before asserting — three
pieces of manual bookkeeping to guarantee everyone has finished before we
count. We already own a tool that *is* that guarantee: `thread::scope` won't
return until every thread spawned inside it is done. The WaitGroup dissolves
into the block structure:

```rust,ignore
    #[test]
    fn it_runs_safely_concurrently() {
        let wanted_count = 1000;
        let mut counter = Counter::new();

        std::thread::scope(|scope| {
            for _ in 0..wanted_count {
                scope.spawn(|| {
                    counter.inc();
                });
            }
        });

        assert_counter(&counter, wanted_count);
    }
```

## Try to run the test

In Go, this test compiles, runs, and fails *probabilistically* — `got 939,
want 1000`, a different wrong number every run, because increments from
different goroutines trample each other. Here:

```text
error[E0499]: cannot borrow `counter` as mutable more than once at a time
  --> src/lib.rs:45:29
   |
43 |           std::thread::scope(|scope| {
   |                               ----- has type `&'1 Scope<'1, '_>`
44 |               for _ in 0..wanted_count {
45 |                   scope.spawn(|| {
   |                   -           ^^ `counter` was mutably borrowed here in the previous iteration of the loop
   |  _________________|
   | |
46 | |                     counter.inc();
   | |                     ------- borrows occur due to use of `counter` in closure
47 | |                 });
   | |__________________- argument requires that `counter` is borrowed for `'1`
```

Our old friend E0499. `inc` takes `&mut self`, a thousand closures each want
that exclusive borrow at once, and exclusive means exclusive. This *is* the
`got 939` bug — the compiler is describing the exact interleaving that
produces it, before it can ever happen.

So the test did its job: it's failing, loudly, at the earliest possible
moment. Now we make it pass.

## Write enough code to make it pass

What we're asking for is many threads mutating one value through shared
references. We've solved "mutation through a shared reference" before — in
the [Mocking](mocking.md) chapter, `RefCell` gave our spy interior
mutability. Let's try it: store `value: RefCell<u32>`, have `inc` take
`&self` and `borrow_mut()` internally.

```text
error[E0277]: `RefCell<u32>` cannot be shared between threads safely
  --> src/lib.rs:49:29
   |
49 |                   scope.spawn(|| {
   |                               ^^ `RefCell<u32>` cannot be shared between threads safely
   |
   = help: within `Counter`, the trait `Sync` is not implemented for `RefCell<u32>`
   = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
```

Closer! The single-threaded test passes now — but the concurrent one exposes
`RefCell`'s limit. Its borrow-counting bookkeeping is plain arithmetic with
no protection against two threads doing it simultaneously, so `RefCell`
doesn't implement `Sync` — the marker trait for "safe to share `&T` across
threads". This is worth savouring: *whether a type may cross a thread
boundary is part of Rust's type system*. The tools divide cleanly:

| single-threaded | thread-safe |
|---|---|
| `RefCell<T>` | `Mutex<T>` / `RwLock<T>` |
| `Rc<T>` | `Arc<T>` |

The compiler suggested `RwLock` (a lock that lets many readers *or* one
writer in); the simpler tool, and the star of Go's chapter too, is `Mutex` —
a **mut**ual **ex**clusion lock. Only one thread can hold it at a time;
everyone else queues:

```rust,ignore
{{#include ../code/sync/v2/src/lib.rs:code}}
```

`lock()` blocks until this thread gets its turn, then returns a *guard* that
is your proof of holding the lock: dereference it (`*`) to reach the data.
Two things about it are very Rust:

- **The data lives inside the lock.** Go's `Counter` has `mu` and `value`
  side by side, and correctness depends on every method remembering to lock
  before touching `value` — nothing stops a future method from forgetting.
  `Mutex<u32>` *contains* the number; the only path to the data runs through
  `lock()`. Forgetting is a compile error, not a code-review hazard.
- **There is no `Unlock`.** Go writes `defer c.mu.Unlock()`; here the guard
  unlocks the mutex when it drops — for our one-line methods, at the end of
  the statement. The same RAII that closed last chapter's test servers
  releases this chapter's locks.

(The `.unwrap()` on `lock()`? A mutex becomes *poisoned* if a thread panics
while holding it, and `lock()` returns a `Result` so you can decide what
a half-finished update means. Unwrapping — treating poison as fatal — is the
standard default.)

And note what happened to the signatures: `inc` is back to `&self`. Interior
mutability means the counter can be *shared* again — which is exactly what
a thousand closures wanted. Delete the `mut` from both tests, run:

```text
running 2 tests
test tests::incrementing_the_counter_3_times_leaves_it_at_3 ... ok
test tests::it_runs_safely_concurrently ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

One thousand every time.

## The chapter Go has that we don't need

The Go original spends its final act on two footguns, and it's worth seeing
why neither survives the trip:

**Embedding the mutex.** Go lets you embed `sync.Mutex` in a struct, which
accidentally makes `Lock` and `Unlock` part of your *public API* — inviting
callers to unlock your counter's internals at nefarious moments. In Rust the
mutex is a private field; privacy is the default; there is nothing to
accidentally export.

**Copying the mutex.** Go's `sync.Mutex` must never be copied after first
use, structs copy silently when passed by value, and only the separate
`go vet` linter catches it — the Go chapter has to introduce a constructor
returning a pointer just to steer users away. Rust's `Mutex` simply doesn't
implement `Clone` or `Copy`: passing our `Counter` by value *moves* it
(fine — there's still one of it), and sharing it is a borrow. The bug `go
vet` warns about is unwritable. The ownership rules from half a book ago
turn out to have been the concurrency rules all along.

## Sharing beyond a scope

One loose end. Scoped threads let everything borrow `counter` because the
scope promises the threads die first. But what about threads that *outlive*
the function that made them — a worker pool, a background job? Nobody can
*borrow* the counter then. It needs shared *ownership*: the counter stays
alive until its last user is done.

We've met shared ownership: `Rc` from the Mocking chapter. And look at the
table above — you can already predict what the compiler will say:

```text
error[E0277]: `Rc<Counter>` cannot be sent between threads safely
   |
35 |             handles.push(std::thread::spawn(move || {
   |                                             ^^^^^^^
   = help: the trait `Send` is not implemented for `Rc<Counter>`
```

`Rc`'s reference count is non-atomic arithmetic, so `Rc` isn't `Send` (the
marker trait for "safe to hand to another thread" — `Sync`'s sibling). Its
thread-safe twin is `Arc` — **a**tomically **r**eference **c**ounted — with
an identical API:

```rust,ignore
{{#include ../code/sync/v3/src/lib.rs:arc_test}}
```

Each thread gets its own `Arc::clone` — cloning the *handle*, not the
counter; all the clones point at one `Counter` — and the counter is freed
when the last clone drops. Since these free-range threads answer to no
scope, we wait for them by collecting each `thread::spawn`'s `JoinHandle`
and calling `join()` — this is what Go's `WaitGroup` looks like when you
build it by hand. `Arc<Mutex<T>>` is *the* canonical shape for shared
mutable state in concurrent Rust; you'll see it everywhere.

## Wrapping up

- **`Mutex`** locks data — and in Rust, *contains* it: the type system won't
  let you reach the value without holding the lock, and the guard's `Drop`
  means you can't forget to release it.
- **`thread::scope` is a structural `WaitGroup`** for threads with
  a boundary; **`JoinHandle::join`** is the explicit version for threads
  without one.
- **`Send` and `Sync`** are how the borrow checker's guarantees extend
  across threads: thread-safety is a compile-time property of types.
  Single-threaded tools (`RefCell`, `Rc`) are cheaper because they skip the
  synchronisation — and the compiler ensures "single-threaded" stays true.
- **`Arc`** is shared ownership across threads; `Arc<Mutex<T>>` is shared
  mutable state.

### When to use locks over channels?

The Go wiki's advice ports directly, and Rust's ownership system sharpens
it:

- **Use channels when passing ownership of data** — and in Rust that phrase
  is literal: `send` *moves* the value; the sender provably can't touch it
  afterwards.
- **Use mutexes for managing state** — one value, in place, taken in turns.

A common newbie mistake in both languages is to over-use channels because
they feel clever. Don't be afraid of a `Mutex` when "take turns updating
one number" is genuinely the shape of your problem — as it was today.

### What about `go vet`?

Go's chapter closes by telling you to add `go vet` to your build scripts to
catch bugs like the copied mutex. Rust's equivalent advice would be... the
compiler you're already running. The checks that need a separate linter and
programmer discipline in Go — don't copy locks, don't share what isn't
thread-safe — were `rustc` hard errors throughout this chapter. (Do still
run `clippy`, as we do in CI — but for style and sharper idioms, not for
memory-safety holes.)
