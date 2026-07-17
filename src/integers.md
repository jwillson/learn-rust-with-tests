# Integers

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/integers)**

Integers *mostly* work as you would expect — and the place where they don't is the
most instructive thing in this chapter, so we'll save it for the end. Let's write an
`add` function to try things out.

This time we want a *library*, not a program — there's no `main`, just code for other
code to call:

```sh
cargo new integers --lib
```

That gives you `src/lib.rs` instead of `src/main.rs`. The distinction matters more
than it looks, and by the end of this chapter you'll see why: **doctests only run
for library targets**, and doctests are the best thing in this chapter.

> A note on this book's repo: each step of each chapter is its own crate, so the
> code below is in a crate called `integers_v2`. If you're following along, yours
> is probably just called `integers`. Mentally substitute as needed.

## Write the test first

```rust,ignore
{{#include ../code/integers/v1/src/lib.rs:test}}
```

Note we're calling our variable `sum` rather than `got` this time — pick names that
say something. There's no format string to think about, unlike Go's `%d` versus
`%q`; `assert_eq!` will print whatever you give it.

## Try and run the test

Run `cargo test` and inspect the compilation error:

```text
error[E0425]: cannot find function `add` in this scope
 --> src/lib.rs:7:19
  |
7 |         let sum = add(2, 2);
  |                   ^^^ not found in this scope
```

## Write the minimal amount of code for the test to run and check the failing test output

Write enough code to satisfy the compiler *and that's all* — remember, we want to
check that our test fails for the correct reason.

```rust,ignore
pub fn add(x: i32, y: i32) -> i32 {
    0
}
```

`pub` makes the function public. Unlike Go, where capitalisation decides visibility,
Rust has a keyword, and everything is private unless you say otherwise. Our test can
still see private functions (it lives inside the module), but we want `add` to be
usable by others, so `pub` it is.

Now run the test, and we should be happy that it's correctly reporting what's wrong:

```text
---- tests::adding_two_numbers stdout ----
thread 'tests::adding_two_numbers' panicked at src/lib.rs:14:9:
assertion `left == right` failed
  left: 0
 right: 4
```

## Write enough code to make it pass

In the strictest sense of TDD we should now write the *minimal amount of code to make
the test pass*. A pedantic programmer may do this:

```rust,ignore
pub fn add(x: i32, y: i32) -> i32 {
    4
}
```

Ah hah! Foiled again. TDD is a sham, right?

We could write another test with different numbers to force that to fail, but that
feels like [a game of cat and mouse](https://en.m.wikipedia.org/wiki/Cat_and_mouse).

Once we're more familiar with Rust's syntax, we'll cover a technique called
[property-based testing](property-based-tests.md), which stops annoying developers and helps you find real
bugs.

For now, let's fix it properly:

```rust,ignore
{{#include ../code/integers/v1/src/lib.rs:code}}
```

No `return`, no semicolon. `x + y` is the last expression in the function body, so
it's the return value. If you re-run the tests they should pass.

## Refactor

There's not a lot in the *actual* code we can improve. But we can improve how
someone *uses* it.

It's preferable that a user can understand your code from its type signature and its
documentation, without reading the body. Rust has documentation comments — `///`,
three slashes — and they support Markdown:

```rust,ignore
/// Adds two integers and returns the sum.
pub fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

Run `cargo doc --open` and you'll see it, rendered, in the same interface as the
standard library docs. This is not a separate tool you install or a comment format
you hope something parses. It's built in, and it's why Rust's library documentation
is so consistently good — everyone gets the same tooling for free.

### Doctests

Here's the best feature in this chapter.

Code examples that live outside the codebase — in a README, in a wiki, in a blog post
— rot. They rot silently, because nothing checks them. Six months later your README
confidently demonstrates an API that no longer exists.

Rust's answer is the **doctest**. Put a code block in a documentation comment, and
`cargo test` compiles *and runs* it:

```rust,ignore
{{#include ../code/integers/v2/src/lib.rs:code}}
```

The `# Examples` heading is a convention that rustdoc renders specially, and the
fenced code block inside it is a real test. Run `cargo test`:

```text
running 1 test
test tests::adding_two_numbers ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

   Doc-tests integers_v2

running 1 test
test code/integers/v2/src/lib.rs - add (line 6) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Two test runs: your unit tests, then your documentation. Your docs are now part of
your test suite. If the code changes so the example is no longer true, the build
fails — try it, change the example to claim `add(1, 5)` is `7`:

```text
   Doc-tests intlib

running 1 test
test src/lib.rs - add (line 5) ... FAILED

---- src/lib.rs - add (line 5) stdout ----
thread 'main' panicked at doctest_bundle_2024.rs:10:1:
assertion `left == right` failed
  left: 6
 right: 7
```

Your documentation cannot lie. That is a remarkable thing to get for free.

#### Doctests test your public API, from outside

Look closely at the first line of the example:

```rust,ignore
use integers_v2::add;
```

Why does the example have to import the very crate it's written in?

Because **each doctest is compiled as its own separate crate that depends on yours**.
It sees exactly what a real user sees, and nothing more. This is different from Go's
`Example` functions, which live in the package and can reach its internals.

The consequence is worth pausing on: a doctest can only demonstrate your *public*
API. You cannot accidentally write documentation showing off a private helper that
your users can't call. The compiler enforces that your examples are examples someone
could actually follow.

It also means doctests are a genuine design tool. If your example is awkward to
write, your API is awkward to use, and you've found that out at the cheapest possible
moment.

#### A few tricks

- Lines starting with `#` are run but hidden from the rendered docs — useful for
  boilerplate that would distract from the point.
- ```` ```no_run ```` compiles the example but doesn't run it. This is the equivalent
  of Go's Examples without an `// Output:` comment: good for code that hits the
  network or blocks forever, where you still want a compile-time guarantee.
- ```` ```should_panic ```` asserts the example panics.
- ```` ```ignore ```` skips it entirely. Use sparingly — it's how docs start rotting
  again.

## A wrinkle: which integer?

Go has `int`, and you mostly don't think about it. Rust makes you choose, and it's
worth knowing why before it surprises you.

We wrote `i32`: a signed, 32-bit integer. Rust also has `i8`, `i16`, `i64`, `i128`,
the unsigned `u8` through `u128`, and `usize`/`isize`, which are pointer-sized and
what you'll see used for indexing and lengths. `i32` is the default the compiler
picks when it has no other information, and it's a fine choice until you have a
reason for another.

Choosing a size means the size is finite, which raises the obvious question. What
does our `add` do here?

```rust,ignore
#[test]
fn adding_one_to_the_largest_i32() {
    let sum = add(i32::MAX, 1);

    assert_eq!(sum, i32::MIN);
}
```

Run `cargo test`:

```text
---- tests::adding_one_to_the_largest_i32 stdout ----
thread 'tests::adding_one_to_the_largest_i32' panicked at src/lib.rs:2:5:
attempt to add with overflow
```

It panicked. Not wrapped around silently — *stopped*, and told us exactly which line
did it. Now run the same test with `cargo test --release`:

```text
running 1 test
test tests::adding_one_to_the_largest_i32 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

It passes. The same test, the same code, the opposite result.

This is not a bug, it's a deliberate trade. In debug builds Rust checks every
arithmetic operation for overflow and panics if one happens, so you find the bug
while developing. In release builds those checks are removed for speed, and the
arithmetic wraps around — `i32::MAX + 1` becomes `i32::MIN`, which is why the release
run agrees with our assertion.

Two things to take from this:

1. **Run your tests in debug.** That's the default, and it's the mode that catches
   overflow. This is the one and only time this book will suggest that `cargo test`
   and `cargo test --release` can legitimately disagree.
2. **When overflow is a real possibility, say what should happen.** Rust makes you
   ask rather than guess:

   ```rust,ignore
   x.checked_add(y)      // Option<i32> — None on overflow
   x.saturating_add(y)   // clamps at i32::MAX
   x.wrapping_add(y)     // wraps, deliberately and in every build
   x.overflowing_add(y)  // (i32, bool) — the result, and whether it wrapped
   ```

`checked_add` returns an `Option`, which is Rust's way of making "this might not have
worked" impossible to ignore. That's the subject of a [later chapter](option-and-pattern-matching.md).

For adding 2 and 2, `x + y` is right. But notice that Rust made the question visible
rather than letting you discover it in production.

## Wrapping up

What we've covered:

- More practice of the TDD workflow
- Integers, addition, and the fact that Rust makes you pick a size
- Library crates, `pub`, and expression-position returns
- Writing documentation with `///` and `cargo doc`
- **Doctests**: examples that are compiled and run as part of your test suite, so
  your documentation can't drift from your code — and which test your public API
  from the outside, the way a real user meets it
- Integer overflow: a panic in debug, a wrap in release, and the `checked_`/
  `saturating_`/`wrapping_` family for when you need to be explicit

Doctests are worth the detour. Most languages ask you to choose between
documentation that's readable and documentation that's correct. Rust just compiles
it.
