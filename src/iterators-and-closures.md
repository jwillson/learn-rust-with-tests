# Iterators and closures

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/iterators)**

The Go book follows its generics chapter with "Revisiting arrays and slices",
where the newly-arrived type parameters are used to hand-build the classic
higher-order functions — `Reduce`, `Find` — because Go's standard library
doesn't ship them. Rust's does. The
[`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) trait
carries the entire vocabulary — `fold` (that's `Reduce`), `find`, `map`,
`filter`, `sum`, and some seventy others — and you've been living off it all
book: every `for` loop, the `.iter().sum()` that collapsed the
[Arrays](arrays-and-slices.md) chapter's `sum`, criterion's benchmarks.

So instead of building the wheel, this chapter is about *driving* it:

- **closures** — finally given the proper introduction we've been promising
  since the Mocking chapter,
- the **iterator adaptors** that consume them, taught by refactoring code we
  already own,
- and then the trait's party trick: implement *one method* on your own type
  and inherit the whole vocabulary.

## The bad bank

The Go chapter's best example is a tiny bank, and it ports beautifully.
Suppose we have a list of transactions and want a function that works out
a person's balance from them.

## Write the test first

```rust,ignore
{{#include ../code/iterators/v1/src/lib.rs:test}}
```

Riya received 100. Chris received 25 but sent 100, so he's down 75. Adil
sent 25. (Yes, we're asserting `==` on `f64` — fine here because these exact
values are reachable by exact arithmetic, but remember the
[Integers](integers.md) chapter's warnings before you do this with numbers
that aren't.)

## Try to run the test

The compiler asks for `Transaction` and `balance_for` in the usual way, so
we oblige with a struct and a stub returning `0.0`:

```rust,ignore
{{#include ../code/iterators/v1/src/lib.rs:types}}
```

```text
assertion `left == right` failed
  left: 0.0
 right: 100.0
```

## Write enough code to make it pass

Let's write it as if this book had no iterators chapter:

```rust,ignore
{{#include ../code/iterators/v1/src/lib.rs:code}}
```

Green. Commit. (Have some source-control discipline — we have working
software, ready to challenge Monzo, Barclays, et al. Now we're free to play
in the refactor step, because the tests will catch us if we fall.)

## Refactor — meet `fold`, and actually meet closures

That function is a *reduction*: walk a collection, feeding each element into
a running value, and return the final value. The Go chapter spends its
middle act building `Reduce[A, B any]` to capture that pattern — including
a detour where their first version can only reduce `[]A` into an `A`, and
the bank breaks it because transactions must reduce into a `float64`. Rust's
`fold` arrives with that lesson pre-learned: the accumulator's type is
independent of the element type. Here's `balance_for`, folded:

```rust,ignore
{{#include ../code/iterators/v2/src/lib.rs:code}}
```

`fold` takes two arguments: the initial accumulator value (`0.0` — and it
matters: for summing you start at 0, for multiplying you'd start at 1, the
*identity element* of your operation, as the Go book enjoys pointing out),
and a **closure**.

### Closures, properly

`|balance, transaction| { ... }` is a closure: an anonymous function you can
pass around as a value. The pipes hold the parameters — types almost always
inferred — and the body follows. We've been using small ones since the
concurrency chapters; what makes closures more than a terse function syntax
is in the name: they *close over* their environment.

Look at the body again: it uses `name`. `name` is not a parameter of the
closure — it's `balance_for`'s parameter, captured from the surrounding
scope. A plain function couldn't do this (try writing `fn adjust(balance:
f64, t: &Transaction) -> f64` — where would `name` come from?). The Go
version leans on the same trick, its `adjustBalance` literal reading `name`
from `BalanceFor`'s scope.

Because captured variables are *borrowed or moved*, closures have the same
ownership stories as everything else in Rust, and the compiler tracks them
through three traits it implements automatically for each closure:

- **`Fn`** — captures by shared reference; can be called freely. (Our
  closure here: it only *reads* `name`.)
- **`FnMut`** — captures or mutates something, needs exclusive access to
  call. `fold` accepts these.
- **`FnOnce`** — consumes a capture, so it can only be called once.

You rarely write these names except as bounds — as we did in the Mocking
chapter's `F: Fn(&str) -> bool` moments — but you've already *felt* them:
the `move` keyword from the [Concurrency](concurrency.md) chapter is
precisely the instruction "capture by taking ownership instead of
borrowing", which is what let closures outlive the function that made them.

### The rest of the vocabulary, in one bank

While we're here, two more requirements show off the adaptors you'll use
daily. *How much has a person sent in total?*

```rust,ignore
{{#include ../code/iterators/v2/src/lib.rs:total_sent}}
```

Read it aloud: iterate, keep only transactions *from* this person, take each
one's amount, add them up. `filter` and `map` each take a closure; `sum`
finishes the job. This is the shape iterator code wants to be: a pipeline
where each stage names what it does.

*Find the first payment somebody received?* The Go chapter builds
`Find[A any]` returning `(A, bool)`; Rust's `find` exists and returns — of
course — an `Option`:

```rust,ignore
{{#include ../code/iterators/v2/src/lib.rs:find}}
```

(Two borrowed parameters and a borrowed return — so the
[Lifetimes](lifetimes.md) chapter's `E0106` came calling, and `<'a>`
answered: the returned transaction borrows from the *slice*, not from
`name`. Note how the annotation documents which input the result is tied
to.)

And the tests, in the same breath:

```rust,ignore
{{#include ../code/iterators/v2/src/lib.rs:extra_tests}}
```

## Laziness — the one thing that will bite you

Iterator adaptors do *nothing* until something consumes them. This is
a feature — it means a ten-stage pipeline makes one pass over the data, no
intermediate collections — but the first time you write a `map` for its side
effects, you'll be surprised:

```rust,ignore
pub fn double_all(numbers: &[i32]) -> Vec<i32> {
    let mut doubled = Vec::new();
    numbers.iter().map(|n| doubled.push(n * 2));
    doubled
}
```

This returns an empty vector — the closure never runs. But you won't be
surprised for long, because Rust won't let it pass silently:

```text
warning: unused `Map` that must be used
 --> src/lib.rs:3:5
  |
3 |     numbers.iter().map(|n| doubled.push(n * 2));
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: iterators are lazy and do nothing unless consumed
```

The same `#[must_use]` machinery that wouldn't let us drop a `Result` in the
[Errors](errors-and-result.md) chapter also knows iterators are inert until
consumed. (The idiomatic fix isn't to consume the `map` — it's that
side-effecting loops should be `for` loops, and *transformations* should be
pipelines ending in a consumer, usually `collect`.)

Which brings us to the refactor this chapter owes you. `sum_all_tails` from
the [Arrays and slices](arrays-and-slices.md) chapter has been a `for` loop
pushing into a `Vec` all this time — a transformation wearing a side
effect's clothes:

```rust,ignore
{{#include ../code/iterators/v3/src/lib.rs:code}}
```

`collect` is the consumer that gathers a pipeline into a collection — and
a small marvel of type inference: it can build a `Vec`, a `String`,
a `HashMap`, even a `Result`, deciding from the return type what you meant.
Tests from the arrays chapter: untouched, still green. That's what
"refactor" means.

## Implement `Iterator` once, inherit everything

Where does all this power come from? One trait with one required method.
Let's earn it ourselves with a type this book has history with: a countdown.

## Write the test first

```rust,ignore
{{#include ../code/iterators/v4/src/lib.rs:test}}
```

If our type is a real iterator, `collect` should just work.

## Try to run the test

After stubbing the struct and its constructor, declare the implementation:

```rust,ignore
impl Iterator for Countdown {
    type Item = u32;
}
```

```text
error[E0046]: not all trait items implemented, missing: `next`
 --> src/lib.rs:5:1
  |
5 | impl Iterator for Countdown {
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `next` in implementation
  |
  = help: implement the missing item: `fn next(&mut self) -> Option<<Self as Iterator>::Item> { todo!() }`
```

The whole contract is this: `type Item` names what you yield (an *associated
type* — like a generic parameter, but chosen by the implementer rather than
the caller), and `next` returns `Some(item)` until you're done, then `None`.
The `Option` we've known since [its chapter](option-and-pattern-matching.md)
turns out to be the engine of iteration: "maybe a value" *is* "a sequence,
eventually exhausted".

## Write enough code to make it pass

```rust,ignore
{{#include ../code/iterators/v4/src/lib.rs:code}}
```

Green. And now the dividend — we wrote `next`, and *everything else in this
chapter arrived for free*:

```rust,ignore
{{#include ../code/iterators/v4/src/lib.rs:free_tests}}
```

`for` loops, `map`, `filter`, `sum`, `collect`, plus dozens we haven't
touched — all of them are *default methods* on the `Iterator` trait,
implemented once in the standard library in terms of `next`. This is the
trait system's compound interest: the Structs chapter showed a trait letting
many types share an interface; here a trait ships an entire library of
behaviour to any type that pays the one-method entry fee.

## Wrapping up

- **Closures** (`|args| body`) are anonymous functions that capture their
  environment — by reference, by mutation, or by `move` — and the `Fn` /
  `FnMut` / `FnOnce` traits let functions like `fold` say which kind they
  accept.
- **The adaptors are Go's hand-built helpers, pre-built**: `fold` is
  `Reduce` (accumulator type independent of element type, no redesign
  needed), `find` is `Find` returning a properly typed `Option`, and
  `filter`/`map`/`sum`/`collect` compose into pipelines that read as
  a description of the result.
- **Iterators are lazy** — nothing happens until a consumer runs the
  pipeline, and the compiler warns you when you forget.
- **`impl Iterator` needs one method.** Yield `Some` until you're done,
  yield `None`, and the whole vocabulary — including the `for` loop itself —
  works with your type.

The Go chapter closes by arguing that higher-order functions like `Reduce`
are safe to adopt because they're *well-defined, tightly-scoped
abstractions* — "the fantastic thing is you have to understand it once and
then you can reuse it forever". Rust's standard library made the same bet on
your behalf, thirty-seventy times over, and the result is that idiomatic
Rust leans functional in exactly the places Go leans on loops. Follow the
same discipline we used here, though: we made it work with a loop *first*,
committed, and only then reshaped it — the pipeline was a refactor under
green tests, not a cleverness gamble.
