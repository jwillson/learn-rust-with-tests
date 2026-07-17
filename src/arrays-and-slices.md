# Arrays, vectors and slices

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/arrays)**

Arrays let you store multiple elements of the same type in a variable in a
particular order.

When you have arrays, it is very common to have to iterate over them. So let's use
[our new-found knowledge of `for`](iteration.md) to make a `sum` function. `sum`
will take an array of numbers and return the total.

Let's use our TDD skills.

## Write the test first

```rust,ignore
{{#include ../code/arrays/v1/src/lib.rs:test}}
```

Two new things here.

`[1, 2, 3, 4, 5]` is an **array**: fixed size, known at compile time. Its type is
written `[i32; 5]` — the element type *and the length*. We didn't have to write
that type out; the compiler infers it.

And look at the assertion: `assert_eq!` takes optional extra arguments, `println!`
style, which are added to the failure message. It is sometimes useful to print the
inputs to the function when a test fails. The `{numbers:?}` is new too — `?`
selects **Debug formatting**. Plain `{}` only works for types with a human-facing
text form (strings, numbers); `{:?}` is the "just show me the value, I'm
debugging" form, and it works on arrays, vectors, and nearly everything else.
You'll use it constantly in tests.

## Try and run the test

The familiar error, minimally satisfied:

```text
error[E0425]: cannot find function `sum` in this scope
```

## Write the minimal amount of code for the test to run and check the failing test output

```rust,ignore
pub fn sum(numbers: [i32; 5]) -> i32 {
    0
}
```

Your test should now fail with *a clear error message* — including our input,
thanks to that extra argument:

```text
---- tests::sums_a_collection_of_numbers stdout ----
thread 'tests::sums_a_collection_of_numbers' panicked at src/lib.rs:16:9:
assertion `left == right` failed: given [1, 2, 3, 4, 5]
  left: 0
 right: 15
```

## Write enough code to make it pass

If you arrived here from a C-flavoured language, you might reach for an index:

```rust,ignore
pub fn sum(numbers: [i32; 5]) -> i32 {
    let mut total = 0;
    for i in 0..5 {
        total += numbers[i];
    }
    total
}
```

`numbers[i]` gets the value out at a particular index, and this compiles and
passes. But before moving on, run `cargo clippy`:

```text
warning: the loop variable `i` is only used to index `numbers`
 --> src/lib.rs:3:14
  |
3 |     for i in 0..5 {
  |              ^^^^
  = note: `#[warn(clippy::needless_range_loop)]` on by default
help: consider using an iterator
  |
3 -     for i in 0..5 {
3 +     for <item> in &numbers {
```

This is what clippy is for. The indexed loop isn't *wrong*, but it makes the
reader (and the compiler) verify that `i` stays in bounds, when all we wanted was
each element in turn. Rust's `for` gives you the elements directly:

```rust,ignore
{{#include ../code/arrays/v1/src/lib.rs:code}}
```

Where Go's `range` hands you an index-value pair and you discard the index with
`_`, Rust's `for` hands you just the values, and if you ever *do* want the index,
you ask for it with `.enumerate()` — which we'll meet properly in the
[Iterators and closures](iterators-and-closures.md) chapter.

## Arrays and their type

An interesting property of arrays is that the size is part of the type. If you try
to pass an `[i32; 3]` into a function expecting `[i32; 5]`, it won't compile —
it's just the same as passing a `String` where an `i32` was wanted.

```text
error[E0308]: mismatched types
  --> src/lib.rs:17:23
   |
17 |         let got = sum(numbers);
   |                   --- ^^^^^^^ expected an array with a size of 5, found one with a size of 3
```

You may be thinking it's quite cumbersome that arrays have a fixed length, and most
of the time you probably won't be using them!

What we want is a type that means "some numbers, I don't care how many". Rust
calls that a **slice**: `&[i32]`. You've already met its close relative — `&str`
is to `String` roughly what `&[i32]` is to an array (or a vector, coming shortly):
a borrowed view of elements that live somewhere else, with the length carried at
runtime instead of in the type.

The next requirement is to sum collections of varying sizes.

## Write the test first

```rust,ignore
{{#include ../code/arrays/v2/src/lib.rs:test}}
```

## Try and run the test

The new test doesn't compile — a 3-element array isn't a 5-element array. The
problem is the same one the Go book hits here, and so is the decision: we could
keep the old function and add a new one, or change `sum`'s signature and fix up
the callers. No one else is using our function, so rather than having two
functions to maintain, let's have just one:

```rust,ignore
{{#include ../code/arrays/v2/src/lib.rs:code}}
```

Note the calls in the tests became `sum(&numbers)` — we lend the function a view
of our array rather than handing the array over. A `&[i32; 5]` coerces to a
`&[i32]` automatically: "a borrowed array of exactly 5" can always stand in for
"a borrowed run of however many". Both tests pass.

The body didn't change at all — iterating a slice looks just like iterating an
array. (Strictly, iterating `&numbers` yields *references* to each element, and
`+=` happily accepts them. Rust is quietly doing the right thing here; the
machinery underneath is the borrow system, and it gets a proper explanation in
[Ownership and borrowing](ownership-and-borrowing.md).)

## Refactor

The production code is fine. But now look at the *tests* with a critical eye.

It should not be a goal to have as many tests as possible; it should be a goal to
have as much *confidence* as possible. Too many tests become a maintenance
burden. **Every test has a cost.**

In our case, two tests for this function is redundant — if it works for a slice of
one size, it very likely works for any size. So delete
`sums_a_collection_of_five_numbers`.

On coverage: Rust's tooling doesn't build coverage into `cargo` the way Go builds
it into `go test -cover`, but the community tool is a one-liner away —
`cargo install cargo-llvm-cov`, then `cargo llvm-cov` — and worth pointing at your
code occasionally. Whilst striving for 100% should not be your goal, if you've
been strict with TDD you'll probably be close anyway.

One more refactor, this time the production code. Summing a sequence is not a new
problem, and Rust's standard library solves it at the source:

```rust,ignore
{{#include ../code/arrays/v3/src/lib.rs:sum}}
```

`.iter()` produces an iterator over the elements and `.sum()` consumes it. This is
the first sighting of Rust's iterator machinery, which is large, central, and has
[its own chapter](iterators-and-closures.md) — for now, enjoy that the entire function became one line that
says what it means.

## `sum_all`

We need a new function called `sum_all`, which takes a varying number of
collections and returns a new collection containing the total of each.

For example, `sum_all` of `[1, 2]` and `[0, 9]` would return `[3, 9]`.

Here's a place where the port has to be honest: the Go version uses a *variadic*
function — `SumAll(numbersToSum ...[]int)`. **Rust doesn't have variadic
functions.** The Rust way to say "any number of collections" is to take one
collection *of* collections: a slice of slices, `&[&[i32]]`. The caller's brackets
nest one deeper; nothing else changes.

## Write the test first

```rust,ignore
{{#include ../code/arrays/v3/src/lib.rs:sum_all_test}}
```

`vec![3, 9]` is our first **vector**. `Vec<i32>` is to arrays what `String` is to
`&str`: the owned, growable version. Our function will have to *build* its answer
— so it can't return a borrowed view of something that already exists; it must
return something it owns. Return `String` when you build text; return `Vec` when
you build a collection. Same rule, second appearance.

The `vec![...]` macro builds one with initial contents.

A pleasant non-event to notice: we compared two vectors with `assert_eq!` and
nobody complained. The Go book spends a section here on the fact that Go slices
can't be compared with `==`, detouring through `slices.Equal` and the
type-safety-losing `reflect.DeepEqual`. In Rust, collections compare with `==` if
their elements do, the check is deep, and it's still fully type-checked. Another
section of the original that idiomatic Rust simply deletes.

## Write the minimal amount of code for the test to run and check the failing test output

```rust,ignore
pub fn sum_all(numbers_to_sum: &[&[i32]]) -> Vec<i32> {
    Vec::new()
}
```

```text
---- tests::sums_several_collections stdout ----
assertion `left == right` failed
  left: []
 right: [3, 9]
```

## Write enough code to make it pass

Iterate over the collections, sum each with our existing `sum`, and push each
result onto the vector we return:

```rust,ignore
{{#include ../code/arrays/v3/src/lib.rs:sum_all}}
```

`push` appends an element — this is Go's `append`, except it mutates in place
(spot the `mut`) rather than returning a new slice. If you know the final size up
front you can pre-allocate with `Vec::with_capacity(n)`, which is Go's
`make([]int, 0, n)`; here the vectors are tiny and it isn't worth the ink.

The tests should now pass.

## `sum_all_tails`

Our next requirement is to change `sum_all` to `sum_all_tails`, which calculates
the totals of the "tails" of each collection. The tail of a collection is
everything except the first item (the "head").

## Write the test first

```rust,ignore
{{#include ../code/arrays/v4/src/lib.rs:test}}
```

## Write enough code to make it pass

```rust,ignore
{{#include ../code/arrays/v4/src/lib.rs:code}}
```

Slices can be sliced! `&numbers[1..]` takes everything from index 1 to the end —
ranges again, doing double duty. `[..3]` would take the first three, `[1..3]` the
middle. The test passes.

## Refactor

Not a lot to refactor this time.

But: what do you think happens if you pass an *empty* collection to our function?
What is the tail of nothing? What happens when you ask Rust for `&empty[1..]`?

## Write the test first

```rust,ignore
{{#include ../code/arrays/v5/src/lib.rs:empty_test}}
```

## Try and run the test

```text
---- tests::safely_sums_empty_collections stdout ----
thread 'tests::safely_sums_empty_collections' panicked at src/lib.rs:8:28:
range start index 1 out of range for slice of length 0
```

Oh no! The test *compiled* — and then failed at **runtime**.

This deserves a pause, because this book has spent three chapters telling you the
Rust compiler catches your mistakes, and here is one it didn't. Indexing and
slicing with `[]` are an escape hatch: the cost of their convenience is a bounds
check at runtime, and a panic if you're out of range. Rust's promise is that the
panic is immediate, loud, and points at the exact line — never silent memory
corruption — but a runtime failure is still a runtime failure, and compile-time
errors are friends where runtime errors are enemies, because runtime errors reach
users.

Rust knows `[]` is an escape hatch, and provides a front door too. Hold that
thought for one section.

## Write enough code to make it pass

The direct fix — don't take the tail of something empty:

```rust,ignore
{{#include ../code/arrays/v5/src/lib.rs:code}}
```

## Refactor

That works, but the `if`/`else` is doing something the standard library already
knows how to do. Alongside `&numbers[1..]` — "give me the tail *or crash*" — Rust
offers `.get(1..)` — "give me the tail, *if there is one*":

```rust,ignore
{{#include ../code/arrays/v6/src/lib.rs:code}}
```

`.get` returns an `Option`: a value that is explicitly either something or
nothing, and which the compiler will not let you use without saying what happens
in the nothing case. `unwrap_or(&[])` says "and if there's nothing, use an empty
slice". The panic isn't handled better — it's *gone*, unwritable, and the empty
case is right there in the code where a reader can see it.

`Option` is one of Rust's best ideas, and it has [a chapter of its own](option-and-pattern-matching.md) — this
is just the trailer.

While we're refactoring: the two tests repeat their assertion shape, and this
time extracting the helper we promised in [Hello, World](hello-world.md) is worth
it, `#[track_caller]` and all:

```rust,ignore
{{#include ../code/arrays/v6/src/lib.rs:test}}
```

The Go version makes its helper a *local closure* inside the test function, to
keep it out of the package namespace. Rust closures exist and get a proper
introduction [later](iterators-and-closures.md); we don't need one here because Rust's module system
already scopes `check_sums` to the `tests` module — invisible to production code
and other files by construction.

A handy side-effect: the helper is type-checked like everything else. If someone
mistakenly writes `check_sums(sum_all_tails(...), vec!["dave"])`, the compiler
stops them — a `Vec<&str>` is not a `Vec<i32>`. (The Go original makes this same
point *against* its own `reflect.DeepEqual`, which would happily compare a slice
to a string and return false at runtime. `assert_eq!` never left the type system,
so the problem never arose.)

## Wrapping up

We have covered:

- **Arrays** `[i32; 5]`: fixed size, size in the type
- **Slices** `&[i32]`: borrowed view, any size — the type to *accept*
- **Vectors** `Vec<i32>`: owned and growable — the type to *return* when you build
  a collection; `vec![]`, `.push()`, `with_capacity`
- Slicing with ranges, `&numbers[1..]` — and the runtime panic waiting inside `[]`
- `.get()` and `unwrap_or` as the panic-free front door, with a first glimpse of
  `Option`
- Debug formatting `{:?}` and custom `assert_eq!` messages
- clippy nudging an indexed loop into an idiomatic one, and `.iter().sum()`
  replacing a hand-written loop
- Deleting a redundant test, because every test has a cost

The trio to remember is **array / slice / Vec** — and the pattern from strings
repeats exactly: a borrowed view type you accept (`&str` / `&[T]`) and an owned
type you build and return (`String` / `Vec<T>`). Two chapters, one rule. It isn't
a coincidence, and the [Ownership and borrowing](ownership-and-borrowing.md) chapter explains the system
behind it.
