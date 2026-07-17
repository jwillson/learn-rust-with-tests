# Errors and Result

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/errors)**

We left the [last chapter](ownership-and-borrowing.md) with a wallet that can
`withdraw`, and a question: what should happen if you withdraw more than is in
the account? For now, our requirement is that there is no overdraft facility.

We also left a threat hanging — that `self.balance -= amount` on a `u64` would do
something bad. This is TDD, so let's not speculate. Let's write the test and
watch.

## Write the test first

```rust,ignore
#[test]
fn refuses_to_withdraw_more_than_the_balance() {
    let mut wallet = Wallet {
        balance: Bitcoin(20),
    };

    wallet.withdraw(Bitcoin(100));

    assert_balance(&wallet, Bitcoin(20));
}
```

## Try to run the test

```text
---- tests::refuses_to_withdraw_more_than_the_balance stdout ----
thread 'tests::refuses_to_withdraw_more_than_the_balance' panicked at src/lib.rs:8:9:
attempt to subtract with overflow
```

There's the [Integers chapter](integers.md) cashing its cheque: 20 minus 100
underflowed the `u64`, and the debug build stopped us on the exact line. (In a
release build it would have wrapped around to roughly eighteen quintillion BTC —
a career-limiting way to implement an overdraft.)

The panic is a *symptom*, though. The disease is that `withdraw`'s signature is
a lie:

```rust,ignore
pub fn withdraw(&mut self, amount: Bitcoin)
```

This type says "give me an amount and I will withdraw it. This always works."
It doesn't always work. The function needs a way to say no.

## How do we signal failure?

In Go, a function that can fail returns an extra value — `(result, error)` — and
the caller is trusted to check whether `err` is `nil` before carrying on. It's a
convention, and it works, but both halves are load-bearing trust: nothing stops
you using the result without looking at the error, and nothing stops you
forgetting the check entirely. (The Go book, honestly, ends its chapter by
installing a third-party linter, `errcheck`, to catch exactly that.)

Rust folds the two values into one type:

```rust,ignore
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

This is an **enum** — a type whose value is one of a fixed set of *variants*.
Enums are a big deal in Rust and get a proper treatment in the
[next chapter](option-and-pattern-matching.md); today we only need this one. A
`Result<T, E>` is *either* an `Ok(T)` carrying the success value *or* an
`Err(E)` carrying the failure — never both, never neither. You can't reach the
success value without going through the fact that it might not exist. Where Go
returns success *and* error side by side and trusts you to look left, Rust
returns success *or* error and doesn't need to trust you.

Our `withdraw` succeeds with nothing to report or fails with an error, so it
will return `Result<(), SomethingWentWrong>` — `()` is Rust's "nothing" type,
called *unit*.

Update the test to capture and check the result:

```rust,ignore
{{#include ../code/errors/v1/src/lib.rs:test}}
```

## Write enough code to make it pass

What's the simplest error type to start with? A `String`:

```rust,ignore
{{#include ../code/errors/v1/src/lib.rs:code}}
```

Both new tests pass — we return early with `Err(...)` before the subtraction can
underflow, so the guard *is* the fix for the panic. `Ok(())` looks like a
typographic accident but reads as "success, containing nothing".

## The compiler does errcheck's job

Look at the *old* happy-path test, though. `wallet.withdraw(Bitcoin(10))` now
returns a `Result`, and the test just... drops it. In Go, adding a return value
to a function whose callers ignore it is exactly the bug class `errcheck` exists
to find. Run `cargo test`:

```text
warning: unused `Result` that must be used
  --> src/lib.rs:27:9
   |
27 |         wallet.withdraw(10);
   |         ^^^^^^^^^^^^^^^^^^^
   |
   = note: this `Result` may be an `Err` variant, which should be handled
```

`Result` is marked `#[must_use]` in the standard library: ignoring one is a
warning from the compiler itself, no third-party linter required — and since our
CI treats warnings as errors, for us it simply doesn't build. The whole
"unchecked errors" section of the Go chapter is a built-in.

So the happy-path test must now say what it expects of the result:

```rust,ignore
let result = wallet.withdraw(Bitcoin(10));

assert!(result.is_ok());
assert_balance(&wallet, Bitcoin(10));
```

Which is no chore — "the withdrawal succeeded" was always part of the
specification; the compiler just made us write it down.

## Refactor: errors are values — with types

"oh no" was never going to survive review. Let's assert on the message so we're
forced to improve it... and immediately feel the trap:

```text
assertion `left == right` failed
  left: Err("oh no")
 right: Err("cannot withdraw, insufficient funds")
```

We can make that pass by improving the message. But now the test is coupled to
the *prose*. Reword the error — even fix a typo — and the test breaks. We don't
care about the exact wording; we care that *the* insufficient-funds error came
back, as opposed to some other failure.

The Go book solves this with a sentinel error *value*,
`var ErrInsufficientFunds = errors.New(...)`, and compares against it. Rust goes
one better: make the error a *type*.

```rust,ignore
{{#include ../code/errors/v2/src/lib.rs:error}}
```

A struct with no fields (a *unit struct*) — there's nothing to store; the type
itself is the information. The signature becomes:

```rust,ignore
pub fn withdraw(&mut self, amount: Bitcoin) -> Result<(), InsufficientFundsError>
```

And the test compares values, not strings:

```rust,ignore
assert_eq!(result, Err(InsufficientFundsError));
```

Read that signature again, because it's doing something the Go version can't:
it's *documentation with enforcement*. A Go `error` return tells you failure is
possible but not what kinds; you read the docs, or the source, and hope. This
signature says there is exactly one way `withdraw` fails. Add a second failure
mode someday — a frozen account, say — and the error type grows a variant, and
every caller that matches on it is *told by the compiler* to decide what to do
about frozen accounts. The Go book's advice at this point is "don't just check
errors, handle them gracefully" — good advice enforced by nothing. Here it's
enforced by the signature.

## Refactor: being a good error citizen

Two small jobs remain before `InsufficientFundsError` is a grown-up Rust error.

The error crosses our API boundary, so someone will eventually show it to a
human. That's `Display` — same trait as Bitcoin's, and TDD works for error
messages too:

```rust,ignore
{{#include ../code/errors/v3/src/lib.rs:display_test}}
```

(`to_string()` comes free with any `Display` implementation.) And Rust has a
standard `Error` trait that marks a type as an error, letting it interoperate
with everyone else's error handling. Once `Debug` and `Display` exist, joining
is one empty line:

```rust,ignore
{{#include ../code/errors/v3/src/lib.rs:error}}
```

Why bother? The same reason the wallet took `&impl Shape` in the
[structs chapter](structs-methods-and-traits.md): code that doesn't care *which*
error can now accept ours. When a later chapter's function returns
`Box<dyn Error>` — "some error, any kind" — `InsufficientFundsError` will
already be welcome.

The final state of the code and tests is in
[`v3`](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/errors/v3/src/lib.rs).

## Not pictured: `?`

One thing this chapter deliberately hasn't shown: what a *caller* does with a
`Result` besides asserting on it in a test. The full answer is pattern matching,
which is the [next chapter](option-and-pattern-matching.md) — but real Rust
functions mostly don't match on every Result either; they punt errors upward
with the `?` operator:

```rust,ignore
fn pay_rent(wallet: &mut Wallet) -> Result<(), InsufficientFundsError> {
    wallet.withdraw(Bitcoin(800))?; // Err? return it. Ok? carry on.
    Ok(())
}
```

That one character is Go's `if err != nil { return err }` — the four lines Go
programmers type until their keyboards wear out. File it away; from the
[Reading files](reading-files.md) chapter onward it's everywhere.

## Wrapping up

### Result

- Functions that can fail return `Result<T, E>` — success *or* failure, one
  value, no convention to remember
- `Result` is `#[must_use]`: the compiler warns when a caller drops one, which
  is the Go chapter's `errcheck` linter as a language feature
- A test that provokes the failure (`is_err()`, or `assert_eq!` against the
  error) and a test that confirms success (`is_ok()`) are both part of the
  specification

### Error types

- Asserting on error *strings* couples tests to prose; asserting on error
  *values* is robust — and Rust error values have types, so signatures document
  their failure modes and the compiler holds callers to them
- A unit struct is enough when the failure carries no data
- Implement `Display` (test the message!) and the `Error` marker trait, and your
  error works with everyone else's

### And the panic that started it all

The underflow panic wasn't bad luck — it was a missing branch of the function's
type. Almost every panic is. When you find one, the fix is usually not "add a
check" but "change the signature so the failure has somewhere to go". The panic
can't come back: there is no way to write the underflow anymore, because the
guard that returns `Err` is the same code that protects the subtraction.
