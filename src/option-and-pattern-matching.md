# Option and pattern matching

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/option)**

This is the second chapter with no Go counterpart, and it completes a set. The
[ownership chapter](ownership-and-borrowing.md) ended by noting that Rust
references can't be `nil`; the [errors chapter](errors-and-result.md) introduced
`Result` as "an enum" and promised the full story later. This is later. By the
end of this chapter you'll have Rust's answer to `nil`, the machinery `Result`
is built from, and the most important control-flow construct in the language.

We'll earn it through two small pieces of code: a return to our shapes, and a
function that has to admit it might not have an answer.

## Enums: types with a fixed set of shapes

In the [structs chapter](structs-methods-and-traits.md) we modelled shapes as
separate structs unified by a `Shape` trait. There's a second way to model "a
shape is a rectangle *or* a circle", and Go has nothing like it, which is why
this chapter exists. An **enum**:

```rust,ignore
#[derive(Debug)]
pub enum Shape {
    Rectangle { width: f64, height: f64 },
    Circle { radius: f64 },
}
```

If you know enums from C or Java, adjust upward: each *variant* here isn't just
a label, it carries its own fields. A `Shape` value is a rectangle with its
dimensions *or* a circle with its radius — exactly one, and the type says these
are the only possibilities.

Let's TDD an `area` function over it. Tests first; they look almost identical to
the trait version, except we construct variants:

```rust,ignore
{{#include ../code/option/v1/src/lib.rs:test}}
```

Stub `area` to return `0.0`, watch the failure, then write the real thing. To do
anything with an enum you must take it apart, and the tool is `match` — which
you met in [Hello, World](hello-world.md) doing small string chores, and which
was built for this:

```rust,ignore
{{#include ../code/option/v1/src/lib.rs:code}}
```

Each arm names a variant and *destructures* its fields into variables — the
pattern `Shape::Rectangle { width, height }` both asks "is it a rectangle?" and,
if so, hands you the width and height. No casting, no field access on the wrong
variant possible.

Notice what's missing compared to Hello, World's `match`: the catch-all `_` arm.
We listed every variant, so there's nothing left to catch. That absence is about
to become the whole point.

## The compiler-driven refactor

New requirement: triangles. Test first — add a triangle case — and it won't
compile, `Triangle` doesn't exist. Add the variant:

```rust,ignore
pub enum Shape {
    Rectangle { width: f64, height: f64 },
    Circle { radius: f64 },
    Triangle { base: f64, height: f64 },
}
```

Run the tests:

```text
error[E0004]: non-exhaustive patterns: `&Shape::Triangle { .. }` not covered
  --> src/lib.rs:9:11
   |
 9 |     match shape {
   |           ^^^^^ pattern `&Shape::Triangle { .. }` not covered
   |
 5 |     Triangle { base: f64, height: f64 },
   |     -------- not covered
```

**`match` is exhaustive, and the compiler enforces it.** We changed the *data*
— added one variant — and the compiler walked the codebase for us and pointed at
every piece of *logic* that hasn't heard the news. In a codebase with fifty
matches on `Shape`, all fifty become compile errors, each with a file and line.
Nothing is forgotten because forgetting doesn't compile.

This is a workflow, and it has a name — *compiler-driven development* — and it
changes how you make changes: encode the new reality in the type, then fix
errors until silence. It's the "listen to the compiler" advice from Hello, World
grown into a load-bearing technique. Add the arm:

```rust,ignore
{{#include ../code/option/v2/src/lib.rs:code}}
```

Tests pass.

One caution now that you know both tools: **the catch-all `_` arm switches this
off.** A `match` with `_` handles new variants "by default", which sounds
convenient and means the compiler can no longer tell you where your logic went
stale. Use `_` when you truly mean "everything else, forever" (like Hello,
World's default greeting for unknown languages); list the variants when each
one deserves a decision. Shapes deserve decisions.

### So... enum or trait?

We've now modelled shapes both ways, which makes this the right moment for the
design question, because Rust will ask it of you constantly:

- A **trait** is *open*: anyone, in any crate, can `impl Shape` for a new type
  — but no function can ever know it has seen every shape.
- An **enum** is *closed*: the variants are a complete list, so `match` can be
  exhaustive and the compiler can hunt down every consequence of a change — but
  adding a variant means editing the enum itself.

Neither is "the Rust way"; the question is *who adds cases*. If users of your
library invent new shapes, trait. If shapes are your domain and you want the
compiler riding shotgun when the list changes, enum. (Go only offers the open
option — interfaces — which is why the Go book's shape code can never make the
compiler do what E0004 just did.)

## Option: the enum that replaces `nil`

Now the second function. We want `largest`, which finds the biggest number in a
collection. Simple — until you ask what the largest number in an *empty*
collection is. There isn't one. The function needs to be able to say "no
answer".

Go's chapter on this territory returns `-1`, or a zero value, or a pointer that
might be `nil`, or a `(value, ok bool)` pair, depending on the API's mood. Rust
has one answer for "a value that might not be there", it's in the standard
library, and you already have the tools to read it — it's just an enum:

```rust,ignore
enum Option<T> {
    Some(T),
    None,
}
```

An `Option<i32>` is `Some(7)` or `None`. That's the whole type. What makes it
better than `nil` isn't cleverness — it's that **the possibility of absence is
in the type, and `match` won't let you ignore it**.

## Write the test first

```rust,ignore
{{#include ../code/option/v3/src/lib.rs:test}}
```

The empty-collection case isn't an edge case we're grudgingly handling — it's
the reason the return type is what it is. Stub with `None`, and the failure
reads exactly as informative as you'd hope:

```text
assertion `left == right` failed
  left: None
 right: Some(7)
```

## Write enough code to make it pass

```rust,ignore
{{#include ../code/option/v3/src/lib.rs:code}}
```

Reading the `match`, two new pattern tricks:

- **Guards**: `Some(current) if number > current` — a pattern plus a condition.
  The arm matches only when both hold.
- **`Some(_)`**: "it's a Some, and I don't need the contents". The wildcard
  works inside patterns, not just as a whole arm.

(Also `for &number in numbers` — patterns work in `for` loops too; the `&`
destructures the reference the slice iterator hands out, so `number` is a plain
`i32`.)

Both tests pass. Now try an experiment: delete the `None` arm.

```text
error[E0004]: non-exhaustive patterns: `None` not covered
 --> src/lib.rs:2:11
  |
2 |     match maybe_number {
  |           ^^^^^^^^^^^^ pattern `None` not covered
```

*There* is the difference between `Option` and `nil`, in one error. Go's
chapter warns "you need to make sure you check if it's nil or you might raise a
runtime exception — the compiler won't help you here". Rust's compiler just
helped. Forgetting the empty case isn't a 3am page anymore; it's a red squiggle
before you've left the editor.

## Refactor

Finding a maximum is not a new problem, and by now you know to check the
standard library before keeping a hand-written loop:

```rust,ignore
{{#include ../code/option/v4/src/lib.rs:code}}
```

`.max()` exists, and — the satisfying part — **it returns `Option` too**,
because the standard library has the same empty-collection problem and gives the
same honest answer. Our hand-rolled version and std agree about the type, which
is a good sign we chose well. (`.copied()` converts the `Option<&i32>` the
iterator gives us into the `Option<i32>` we promised — iterators lend
references; you've seen this since the arrays chapter.)

## Living with Option day to day

`match` is the foundation, but matching every Option in full would be
ceremonious, so the standard library ships shortcuts. Four cover most of life:

```rust,ignore
// A match that only cares about one arm:
if let Some(number) = largest(&numbers) {
    println!("the largest is {number}");
}

// A default for the None case:
let biggest = largest(&numbers).unwrap_or(0);

// Transform the value if it's there, stay None if not:
let doubled = largest(&numbers).map(|n| n * 2);

// And the loud one:
let biggest = largest(&numbers).unwrap();
```

About `unwrap`: it's "give me the value or **panic**". This looks like it
reintroduces the nil crash, and mechanically it does:

```text
thread 'tests::unwrapping_nothing' panicked at src/lib.rs:12:41:
called `Option::unwrap()` on a `None` value
```

The difference is that it's *opt-in and greppable*. Absence can never crash you
by surprise; it crashes you where you wrote `unwrap`, a written-down assertion
that None can't happen here. In **tests**, that's often exactly right — if the
Option is None the test *should* die on the spot, and `.expect("should have
found a largest number")` does the same with a better epitaph. In production
code, an `unwrap` deserves the same suspicion as an ignored error in Go: ask
"what actually happens when this is None?", and if the answer matters, `match`
or `map` or `unwrap_or` it instead. Everything here applies verbatim to
`Result` — it has `unwrap`, `map`, `unwrap_or`, and friends too; they're
siblings from the same enum family.

## Wrapping up

- **Enums** define a type as a fixed set of variants, each carrying its own
  data — the "or" to structs' "and"
- **`match` destructures** variants and is **exhaustive**: when the data grows
  a case, every match that hasn't caught up becomes a compile error. Changing
  the type first and following the errors is a legitimate, everyday refactoring
  technique — and a `_` arm opts out of it, so spend catch-alls carefully
- **Guards** (`if` in a pattern) and nested wildcards (`Some(_)`) keep matches
  precise
- **`Option<T>`** is absence done in the type system: no `nil`, no "check
  before use" convention, just an enum the compiler won't let you half-read
- `unwrap`/`expect` convert absence into a panic *on purpose* — normal in
  tests, a design question in production
- `Result` is the same machinery with a story in both variants — everything you
  learned today about matching, `map`, and `unwrap` transfers directly

The [next chapter](hashmaps.md) puts this straight to work: looking things up
in a dictionary is the canonical "might not be there" problem, and Rust's
`HashMap::get` returns — you already know what it returns.
