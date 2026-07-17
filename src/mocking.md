# Mocking

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/mocking)**

You have been asked to write a program which counts down from 3, printing each
number on a new line (with a 1-second pause), and when it reaches zero it prints
"Go!" and exits.

```
3
2
1
Go!
```

We'll tackle this by writing a `countdown` function, which we'll put inside a
`main` program so it looks something like this:

```rust,ignore
fn main() -> std::io::Result<()> {
    countdown(/* ... */)
}
```

While this is a pretty trivial program, to test it fully we will need, as
always, to take an *iterative*, *test-driven* approach.

What do I mean by iterative? We make sure we take the smallest steps we can to
have *useful software*. **It's an important skill to be able to slice up
requirements as small as you can so you have working software** — that's how you
avoid disappearing down rabbit holes with code that will "theoretically work
after some hacking".

Here's how we can divide our work up and iterate on it:

- Print 3, 2, 1 and Go!
- Wait a second between each line

## Write the test first

Our software needs to print to stdout, and the [DI chapter](dependency-injection.md)
showed us exactly how to make that testable:

```rust,ignore
{{#include ../code/mocking/v1/src/main.rs:test}}
```

## Try and run the test

The familiar dance: `cannot find function`, define a stub that takes the writer
and returns `Ok(())`, and get a good failure:

```text
assertion `left == right` failed
  left: ""
 right: "3\n2\n1\nGo!"
```

## Write enough code to make it pass

```rust,ignore
{{#include ../code/mocking/v1/src/main.rs:code}}
```

Counting *down* in Rust is worth a sentence, because there's no
`for i := 3; i > 0; i--` to translate. Ranges only count up — so you take the
range going up and reverse it: `(1..=COUNTDOWN_START).rev()`. (An amusing porting
footnote: the Go book has a *bonus section* showing how Go 1.23's brand-new
iterators finally let you write a declarative `countDownFrom`. `.rev()` is that
feature; Rust ranges have had it since 1.0, so our first draft is the Go
chapter's epilogue.)

`writeln!` is `write!` plus a trailing newline, and the magic values are already
constants. Run the program — `cargo run` — and be amazed at your handiwork.

Yes, this seems trivial, but this approach is what I would recommend for any
project: **take a thin slice of functionality and make it work end-to-end,
backed by tests.**

Next, we need the dramatic pauses.

## The problem with `sleep`

Add the pause the obvious way — `std::thread::sleep(Duration::from_secs(1))`
inside the loop — and run the tests. They pass. The program works.

And your test suite now takes **three seconds**. Every forward-thinking post
about software development emphasises quick feedback loops; **slow tests ruin
developer productivity**, and every future test of `countdown` will pay the
three-second tax again.

Worse: we haven't actually *tested* the pausing — the very requirement we just
added. The sleeping is real, which makes it slow, and invisible, which makes it
untestable. We have a dependency on sleeping that we need to extract so we can
*control* it in our tests.

If we can *mock* `sleep`, we can use dependency injection to swap the real one
out, and then **spy on the calls** to make assertions about them.

## Write the test first

Let's define the dependency as a trait, so `main` can use a real sleeper and our
tests a spy — `countdown` neither knows nor cares:

```rust,ignore
{{#include ../code/mocking/v2/src/main.rs:sleeper}}
```

(A design decision, same as the Go book's: `countdown` is not responsible for
how *long* a sleep is. That's the caller's business — a decision that will pay
off at the end of the chapter.)

Now the **spy**. Spies are a kind of mock which record how a dependency was used
— how many times, with what, in what order — so the test can assert on it
afterwards. Ours just counts:

```rust,ignore
{{#include ../code/mocking/v2/src/main.rs:spy}}
```

There's one piece of new Rust here, and it's load-bearing: **`RefCell`**. Watch
the borrows to see why it's needed. The trait says `sleep(&self)` — a shared
borrow — but a spy *records*, which is mutation. The
[ownership chapter's](ownership-and-borrowing.md) rule says shared XOR mutable,
so a `u32` field can't be incremented through `&self`… and the test, which still
holds its own borrow of the spy, wants to read the count afterwards.

`RefCell` is the standard library's escape hatch for exactly this: a container
that moves the borrow-checking **from compile time to run time**. You can ask it
for a mutable borrow through a shared reference (`borrow_mut()`), and it keeps
count of outstanding borrows, enforcing the same XOR rule — but by panicking at
runtime instead of failing the build:

```text
thread 'tests::double_borrow' panicked at src/lib.rs:12:15:
RefCell already borrowed
```

That's the trade, stated plainly: *interior mutability* buys flexibility and
costs you a compile-time guarantee. It's the right trade for test doubles —
short-lived, single-threaded, simple access patterns — and you should feel a
small pang whenever you reach for it in production code.

Update the test to inject the spy and assert on the count:

```rust,ignore
{{#include ../code/mocking/v2/src/main.rs:test}}
```

## Write enough code to make it pass

Thread the sleeper through — `countdown` takes `&impl Sleeper` and calls it in
the loop; `main` passes `&DefaultSleeper`. The compiler chases you through each
call site that hasn't heard about the new parameter, as usual:

```rust,ignore
{{#include ../code/mocking/v2/src/main.rs:code}}
```

The tests pass, and they no longer take three seconds.

## Still some problems

There's an important property we *still* haven't tested: `countdown` should
sleep **between** prints — print, sleep, print, sleep. Our test only counts
sleeps; three sleeps in a row up front would pass it.

When you're not confident your tests are giving you enough confidence, just
break the code! (Commit first.) Move all the sleeping into its own loop before
any printing — and watch both tests stay green even though the countdown now
does nothing dramatic at all. We need to record the *interleaving* of writes
and sleeps.

The Go book does this with one spy object implementing both interfaces, passed
as both arguments: `Countdown(spy, spy)`. Try the same trick in Rust:

```text
error[E0499]: cannot borrow `spy` as mutable more than once at a time
  --> src/lib.rs:41:29
   |
41 |         countdown(&mut spy, &mut spy).unwrap();
   |         --------- --------  ^^^^^^^^ second mutable borrow occurs here
   |         |         |
   |         |         first mutable borrow occurs here
```

The borrow checker vetoes the Go design outright: two mutable handles to one
object is precisely what the ownership rules exist to prevent. This is the most
instructive dead end in the book so far, so don't rush past it — the *goal*
(one shared record of events) is legitimate; it's the *shape* (one object worn
as two hats) that Rust won't have.

So express the goal directly: **two spies, one shared log.** The log is the
`RefCell` we already know, and to let two owners share it we wrap it in `Rc` —
a *reference-counted* pointer, the standard tool for "this value has more than
one owner". `Rc::clone` doesn't copy the data; it hands out another handle to
the same allocation and bumps a counter.

```rust,ignore
{{#include ../code/mocking/v3/src/main.rs:spy}}
```

(That `write_fmt` override deserves its comment: a single `writeln!` turns into
several tiny `write` calls under the hood, one per formatted fragment.
Intercepting `write_fmt` — a trait method with a default body, which we're
allowed to replace — records one operation per *print*, which is the level our
test thinks at. Also note `Operation` is an enum, not the Go version's
stringly-typed `"write"`/`"sleep"` constants — typo a variant and it won't
compile.)

The test injects both spies and asserts on the one log:

```rust,ignore
{{#include ../code/mocking/v3/src/main.rs:test}}
```

Against our deliberately-broken countdown:

```text
assertion `left == right` failed
  left: [Sleep, Sleep, Sleep, Write, Write, Write, Write]
 right: [Write, Sleep, Write, Sleep, Write, Sleep, Write]
```

The failure *is* the bug report: all the sleeping happens first. Revert
`countdown` to the honest implementation and everything greens. We now have two
tests: one for *what* is printed, one for the *rhythm* it's printed with.

`Rc<RefCell<T>>` — shared ownership of mutable state — is a famous Rust
combination, and you've now met it in its natural habitat. Keep it in mind: the
[Sync](sync.md) chapter's `Arc<Mutex<T>>` is the same idea wearing thread-safe boots.

## Extending Sleeper to be configurable

A nice feature would be for the sleep duration to be configurable in `main`.
The Go book does this with a struct holding a duration and a *function* to call
with it — and the Rust translation introduces one genuinely new thing:

```rust,ignore
{{#include ../code/mocking/v4/src/main.rs:configurable}}
```

`ConfigurableSleeper` is our first **generic struct**: `F` is "some type that
can be called like a function taking a `Duration`" — the `Fn(Duration)` bound.
That covers ordinary functions *and* closures, which is the point, because the
test wants to hand in a closure that records instead of sleeping:

```rust,ignore
{{#include ../code/mocking/v4/src/main.rs:configurable_test}}
```

Stub `sleep` as a no-op first and enjoy the failing test —
`left: 0ns, right: 5s` — then make it pass by actually calling the function
with the duration (already shown above). Wire it into `main`, where the "real"
function is just `std::thread::sleep` itself, which is an `fn(Duration)` and
therefore already an `Fn(Duration)`:

```rust,ignore
{{#include ../code/mocking/v4/src/main.rs:main}}
```

Run the program: same behaviour, now configurable, and `DefaultSleeper` can be
deleted. Closures get their [own chapter](iterators-and-closures.md); this was their trailer.

## But isn't mocking evil?

You may have heard mocking is evil. Like anything in software, it can be used
for evil, just like DRY. People normally get into a bad state when they don't
*listen to their tests* and don't *respect the refactoring stage*.

If your mocking code is becoming complicated, or you have to mock out lots of
things to test something, *listen* to that bad feeling. It usually means:

- The thing you're testing does too much → break the module apart
- Its dependencies are too fine-grained → consolidate them
- Your test is too concerned with implementation details → test behaviour
  instead

Normally a lot of mocking points to *bad abstraction*. What people see as TDD's
weakness here is actually its strength: **hard-to-test code is the design
feedback, delivered early.**

And some rules of thumb, straight from the original because they don't need
translation:

- If a refactor (behaviour unchanged) forces you to rewrite lots of tests, you
  were testing implementation details
- More than 3 mocks in one test is a red flag
- Use spies with caution — they see the insides of your algorithm, which
  couples test to implementation. **Be sure you actually care about those
  details.** We did here: the pause-between-prints *is* the requirement
- Test private functions rarely if at all; they're implementation detail

One Rust-specific note on frameworks: crates like `mockall` will generate mocks
from your traits, and teams use them for consistency. Everything in this
chapter was a handful of lines per double, though — you don't *need* a
framework, and writing a few spies by hand first will make you a better judge
of what the frameworks generate.

## Wrapping up

### More on TDD approach

- Slice requirements thinly; get to *working software backed by tests* fast,
  then iterate
- > "When to use iterative development? You should use iterative development
  > only on projects that you want to succeed." — Martin Fowler

### Mocking

- **Without mocking, important areas of your code go untested** — we couldn't
  assert on pausing until we owned the sleep. Databases, failing services,
  clocks: same story
- Real dependencies mean slow tests and fragile tests; **slow feedback loops**
- A *spy* records usage so you can assert on it; it's one member of the
  [test double](https://martinfowler.com/bliki/TestDouble.html) family, along
  with stubs and mocks proper

### And the Rust of it

- Traits + DI work exactly as in Go — until a test double needs to *record*,
  and then the ownership rules make you decide who mutates what
- **`RefCell`** moves borrow-checking to runtime: interior mutability for
  test doubles, with a panic instead of a compile error if you misuse it
- **`Rc`** gives a value multiple owners; `Rc<RefCell<T>>` is the shared
  mutable log two spies can both write to — Rust's answer to Go's
  one-object-two-interfaces trick, with the sharing made visible in the types
- Generic structs with `Fn` bounds let "a thing that can be called" be a
  field — functions and closures both qualify
