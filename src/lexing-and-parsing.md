# Lexing and parsing

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/interpreter)**

Welcome to the projects section. Over the next three chapters we're going to
build a small programming language — first a calculator that understands
expressions like `1 + 2 * -3`, then an interpreter that evaluates them, and
finally a compiler that turns them into bytecode for a little virtual machine to
run. It's the most Rust-flavoured thing in the book: implementing a language is
a firehose of enums, exhaustive `match`, recursion, and typed errors, which is
exactly the toolkit the fundamentals chapters gave you.

Every language implementation starts the same way — turning source *text* into a
*structure* the rest of the program can work with. That's two steps:

1. **Lexing** (or tokenizing): `"1 + 2"` → `[Number(1), Plus, Number(2)]`. Group
   the raw characters into meaningful chunks.
2. **Parsing**: those tokens → a tree that captures *structure* and *precedence*,
   so `1 + 2 * 3` knows the multiplication happens first.

This chapter does both.

## Part 1: the lexer

A token is one meaningful unit of source — a number, an operator, a
parenthesis. That's an enum, and using an enum here is the first of many places
where the design almost writes itself:

```rust,ignore
{{#include ../code/interpreter/v1/src/lib.rs:token}}
```

### Write the test first

```rust,ignore
{{#include ../code/interpreter/v1/src/lib.rs:test}}
```

Four scenarios: a normal expression, multi-digit numbers and parentheses,
whitespace-insensitivity, and the failure case — an unexpected character is an
error, not a panic, so `lex` returns a `Result` (the
[Errors chapter](errors-and-result.md) habit).

### Write enough code to make it pass

```rust,ignore
{{#include ../code/interpreter/v1/src/lib.rs:lex}}
```

The heart of it is walking the characters with a **`Peekable` iterator**. `lex`
needs to *look at* the next character to decide what to do without necessarily
*consuming* it — when reading a multi-digit number, we keep peeking and taking
digits until we hit a non-digit, which we must leave for the next iteration.
`chars().peekable()` gives exactly that: `peek()` looks, `next()` consumes. It's
the [Iterators chapter](iterators-and-closures.md)'s machinery earning its keep
on a real problem.

The number branch is the only interesting one: keep folding digits into a
running `value` (`value * 10 + digit`) while the peeked character is a digit.
Everything else is a single character mapped straight to a token, and anything
unrecognised becomes a `LexError`. The exhaustive `match` means we can't
silently forget a character class — every `char` is either whitespace, a known
symbol, a digit, or an error.

## Part 2: the parser

Tokens are a flat list, but `1 + 2 * 3` isn't flat — the `2 * 3` is a unit
nested inside the addition. We need a *tree*, the **Abstract Syntax Tree**, and
that's where this chapter meets one of Rust's most instructive compiler errors.

### The AST needs `Box`

An expression is recursive: a binary operation contains *two more expressions*.
The obvious enum is:

```rust,ignore
pub enum Expr {
    Number(i64),
    Binary { op: BinOp, left: Expr, right: Expr },
}
```

...and it doesn't compile:

```text
error[E0072]: recursive type `Expr` has infinite size
 --> src/lib.rs:1:1
  |
1 | pub enum Expr {
  | ^^^^^^^^^^^^^
...
5 |         left: Expr,
  |               ---- recursive without indirection
  |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
  |
5 |         left: Box<Expr>,
  |               ++++    +
```

This is a genuinely deep Rust lesson delivered as a one-line error. Rust lays
values out *inline* by default — an `Expr` would need to contain two `Expr`s,
each of which contains two more, forever, so its size would be infinite. The fix
is **indirection**: `Box<Expr>` is a pointer to a heap-allocated `Expr`, and a
pointer has a known, fixed size. The recursion goes through the heap, so the
type is finite. This is *why* `Box` exists, and building an AST is the most
natural place to meet it. Here's the AST that compiles:

```rust,ignore
{{#include ../code/interpreter/v2/src/lib.rs:ast}}
```

Note the operators are their own small enums (`UnOp`, `BinOp`) rather than
reusing `Token`. The AST models *meaning* — "this is a negation", "this is a
multiply" — not the surface syntax, and keeping them separate means the
evaluator in the next chapter can `match` on operations without caring about
lexer details.

### Write the test first

Parser tests assert on the *shape* of the tree, and the shape is the whole
point — precedence and grouping have to come out right:

```rust,ignore
{{#include ../code/interpreter/v2/src/lib.rs:test}}
```

`1 + 2 * 3` must parse as `1 + (2 * 3)`, not `(1 + 2) * 3`; parentheses must
override that; unary minus, a missing `)`, and trailing junk all have their own
tests. Because `Expr` derives `PartialEq`, we compare whole trees with
`assert_eq!` — the derived equality from the [Structs chapter](structs-methods-and-traits.md)
doing the work.

### Write enough code to make it pass

This is a **recursive-descent parser**, and its elegance is that the grammar's
precedence levels become a *ladder of methods that call each other*:

```rust,ignore
{{#include ../code/interpreter/v2/src/lib.rs:parse}}
```

The three grammar rules in the comments are the design:

- `expression` handles `+` and `-` (lowest precedence) by parsing a `term`, then
  looping while it sees an additive operator, folding each into a bigger
  `Binary` node.
- `term` does the same for `*` and `/` — but because `expression` calls `term`
  for its operands, multiplication is *always* grouped before addition. **The
  precedence falls out of the call structure**, no priority numbers needed.
- `factor` handles the atoms: a number, a parenthesised sub-expression (which
  recurses back to `expression`), or a unary minus (which recurses to
  `factor`).

The `Parser` struct is a cursor over the token slice — `peek` looks at the
current token, `advance` consumes it — the same look-then-consume pattern as the
lexer, one level up. And every method returns `Result<Expr, ParseError>`, so a
malformed input (`(1 + 2` with no closing paren) propagates a *typed* error via
`?` rather than crashing. `parse` finishes by checking there are no leftover
tokens, catching `1 2`.

## Wrapping up

- **Lexing groups characters into tokens; parsing arranges tokens into an AST.**
  Both are enums-and-`match` all the way down — the fundamentals chapters were
  quietly preparing you for exactly this.
- **`Peekable` is the tool for look-then-consume scanning**, in both the lexer
  (over characters) and the parser (over tokens).
- **A recursive enum needs `Box`** for indirection — `E0072` is the compiler
  explaining that inline recursion has infinite size, and the AST is where every
  Rust programmer first truly meets `Box`.
- **Recursive descent turns grammar precedence into a ladder of methods.**
  Higher-precedence rules are called by lower ones, so the tree comes out
  correctly grouped with no explicit priority handling.
- **Errors are typed and propagated with `?`** — a bad program is a `Result`,
  never a panic.

We have a tree. Next chapter, we walk it and compute a result — our first
working interpreter.
