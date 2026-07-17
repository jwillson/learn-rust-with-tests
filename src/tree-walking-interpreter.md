# A tree-walking interpreter

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/interpreter)**

Last chapter we turned source text into an AST. Now we make the language
*run*: given the tree for `1 + 2 * 3`, produce `7`. The simplest way to execute
an AST is to **walk** it — recurse over the tree, computing each node from its
children. That's a tree-walking interpreter, and it's about twenty lines.

## The evaluator

Evaluating an `Expr` is a recursive `match`, and it's a small marvel of how well
the AST design pays off:

```rust,ignore
{{#include ../code/interpreter/v3/src/lib.rs:eval}}
```

Each arm handles one kind of node:

- A `Number` evaluates to itself.
- A `Unary` negation evaluates its operand (recursively) and negates it.
- A `Binary` evaluates *both* children first, then applies the operator.

The recursion mirrors the tree's structure exactly, and because the tree already
encodes precedence — `1 + 2 * 3` is `1 + (2 * 3)` — the evaluator doesn't think
about precedence *at all*. It computes children before parents, and the right
answer falls out. All the hard work happened in the parser; evaluation is the
easy downhill.

Two things worth noticing. First, **division by zero is a typed error**, not a
panic: `BinOp::Divide if right == 0` is a match guard (the
[Option/pattern-matching chapter](option-and-pattern-matching.md)'s guards) that
catches the bad case and returns `RuntimeError::DivisionByZero`. The `?` on each
recursive `eval` propagates that error up through the whole tree — if any
subexpression divides by zero, the entire evaluation short-circuits to `Err`.
Second, the `match` is **exhaustive**: add a new operator to `BinOp` and this
function won't compile until you handle it. The compiler maintains the
evaluator's completeness for you.

(One honest limitation: `i64` arithmetic can overflow on huge inputs, which — as
the [Integers chapter](integers.md) warned — panics in debug builds. A
production calculator would use `checked_add` and friends to turn overflow into
another `RuntimeError` variant. We've kept the evaluator readable and left that
as a known edge.)

## Tying it together, and meeting `?`-conversion at last

A user hands us *source*, not an AST, so we want one function that lexes, parses,
and evaluates. But each stage has its *own* error type — `LexError`,
`ParseError`, `RuntimeError`. How do we write a clean pipeline over three
different error types?

This is the moment several earlier chapters pointed at. Back in the
[Cancellation chapter](cancellation.md), the compiler told us `?` "performs a
conversion on the error value using the `From` trait." Here's where we use it.
We define one umbrella error and teach it how to absorb each stage's error via
`From`:

```rust,ignore
{{#include ../code/interpreter/v3/src/lib.rs:run}}
```

Look how clean `run` is: three stages, a `?` after each, and *nothing* about
error conversion in sight. When `lex` returns a `LexError`, the `?` automatically
calls `From::from` to turn it into `InterpretError::Lex`; likewise for parse and
runtime errors. The `From` implementations are the one-time cost; every `?` after
that is free. This is *the* idiomatic Rust pattern for a function that
orchestrates several fallible steps with different error types, and building an
interpreter is the perfect place to finally need it.

## One specification, tested once (and soon, twice)

Here's a design decision that will pay off next chapter. Instead of scattering
example programs through the tests, we define them as **public data** — a list of
programs paired with the value each must produce:

```rust,ignore
{{#include ../code/interpreter/v3/src/lib.rs:cases}}
```

The tests iterate over this specification:

```rust,ignore
{{#include ../code/interpreter/v3/src/lib.rs:test}}
```

This is the [specification pattern](scaling-acceptance-tests.md) again, and
making `PROGRAMS` `pub` is deliberate. In the next chapter we'll build a
*completely different* execution engine — a bytecode compiler and virtual
machine — and run it against this *exact same* list, importing it directly. The
specification says what the language *means*; the tree-walker is one
implementation of that meaning, the VM will be another, and the shared tests
prove they agree. A behaviour worth testing is worth testing against every
backend that claims to implement it.

## Wrapping up

- **A tree-walking interpreter is a recursive `match` over the AST** —
  evaluate children, combine at the parent. Precedence is already baked into the
  tree, so evaluation is trivial.
- **Runtime failures are typed errors, propagated with `?`.** Division by zero
  is a `RuntimeError`, caught by a match guard, never a panic.
- **`?` converts between error types via `From`.** Define one umbrella error
  with `From` impls for each stage, and a multi-stage pipeline like
  `lex → parse → eval` reads as three clean lines — the ergonomic error handling
  the book has been building toward.
- **Express the language's behaviour as a shared, public specification.** The
  same `PROGRAMS` list will verify the bytecode VM next chapter, proving two
  very different engines compute the same answers.

The interpreter works — but walking the tree for every evaluation, re-matching
every node each time, isn't how fast languages run. Next we *compile* the AST
once into flat bytecode and execute that on a stack machine.
