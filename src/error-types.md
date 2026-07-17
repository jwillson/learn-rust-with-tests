# Error types

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/error-types)**

Here's a question that comes up constantly: *if my function fails with a
formatted message like `format!("did not get 200 from {url}, got {status}")`,
how do I test that failure without comparing the exact string?*

If you're comparing error strings in tests, your tests are telling you
something. This short chapter shows the fix, and it's a place where Rust is
markedly nicer than most languages: **make your errors a type — an enum with
data — and match on it.**

## The smell: stringly-typed errors

Imagine a `dumb_getter` that fetches a URL and can fail two ways: the request
itself fails, or the server answers with a non-200 status. The tempting way to
test the second case is to reconstruct the exact error string the production
code builds and compare. That test is bad, and listening to it tells us why:

- The test rebuilds the *same string* the production code does, so it's really
  testing that two copies of a format string agree — not behaviour.
- It's annoying to write and read.
- Is the exact wording actually what we care about? No — we care that it was a
  *bad-status* failure, with *this* status.

And the test's discomfort is a preview of your *users'* discomfort. A caller who
wants to react differently to a bad status than to a network failure has nothing
to work with but the error string — which means brittle substring matching in
their code, too. Stringly-typed errors push everyone toward parsing prose.

## The fix: errors are data

We met unit-struct errors in the [Errors chapter](errors-and-result.md); here we
take the next step. When a function can fail in genuinely *different* ways, model
that with an **enum**, each variant carrying the data relevant to that failure:

```rust,ignore
{{#include ../code/error-types/v1/src/lib.rs:error}}
```

`GetterError` isn't a magic string — it's structured data. `Display` still gives
a human-readable message (so it prints nicely and satisfies `std::error::Error`),
but the message is now *derived from* the data, not the source of truth. And
`#[derive(Debug, PartialEq)]` means we can compare error values directly.

The getter returns this type, choosing the right variant for each failure:

```rust,ignore
{{#include ../code/error-types/v1/src/lib.rs:code}}
```

## Testing becomes trivial — and this is where Rust shines

Now the test that felt so awkward becomes a clean comparison of *values*:

```rust,ignore
{{#include ../code/error-types/v1/src/lib.rs:status_test}}
```

We assert the error *equals* `GetterError::BadStatus { url, status: 418 }`. No
string reconstruction, no substring matching — we check the type and the data,
which is exactly what we care about. When we only care about the *variant*, not
its fields, `matches!` is even terser:

```rust,ignore
{{#include ../code/error-types/v1/src/lib.rs:other_tests}}
```

`matches!(err, GetterError::Fetch { .. })` reads as "is this a fetch error?" and
ignores the rest.

Compare this to how the Go version has to work: a type assertion
(`err.(BadStatusError)`) with a boolean "did that even work" check, because Go's
`error` is an interface and you're reaching through it at runtime. In Rust,
because the error type *is* an enum, `match` and `==` and `matches!` all work
directly — the same exhaustive pattern matching we've used since the
[Option chapter](option-and-pattern-matching.md). Typed errors aren't a special
technique here; they're just what errors naturally are.

And the benefit flows to *users* of the code, not just tests. A caller can
`match` on `GetterError` and handle a bad status differently from a network
failure — retry the network error, surface the status to the user — with the
compiler guaranteeing they've considered the variants. That's impossible with a
stringly-typed error and effortless with an enum.

## A word of proportion

Don't reach for a bespoke error enum for *every* function. If a function fails
one way, or the caller genuinely only needs "did it work?", a simple error (or
propagating an existing one with `?`) is fine — over-modelling errors is its own
kind of clutter. The signal to create a type is when a function fails in
*meaningfully different ways that callers will want to distinguish*, or when you
catch yourself testing error strings. Then, an enum pays for itself immediately.

## Wrapping up

- **Don't test error strings.** If you're reconstructing a production format
  string in a test, that's a smell pointing at a design problem.
- **Model distinct failures as an enum with data.** `Display` derives the
  human message from the data; the data is what you and your callers actually
  program against.
- **Rust makes typed errors the path of least resistance.** `#[derive(Debug,
  PartialEq)]` plus `match` / `==` / `matches!` means testing and handling
  specific errors is ordinary pattern matching — no runtime type assertions.
- **Match the modelling to the need.** One failure mode? A simple error is
  fine. Several that callers must tell apart? Reach for the enum.
