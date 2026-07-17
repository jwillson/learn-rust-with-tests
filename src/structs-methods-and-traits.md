# Structs, methods and traits

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/structs)**

Suppose that we need some geometry code to calculate the perimeter of a rectangle
given a height and width. We can write a `perimeter(width, height)` function,
where the arguments are `f64` — Rust's 64-bit floating-point type, for numbers
like `123.45`.

The TDD cycle should be pretty familiar to you by now.

## Write the test first

```rust,ignore
{{#include ../code/structs/v1/src/lib.rs:test}}
```

## Try to run the test

```text
error[E0425]: cannot find function `perimeter` in this scope
```

## Write the minimal amount of code for the test to run and check the failing test output

```rust,ignore
pub fn perimeter(width: f64, height: f64) -> f64 {
    0.0
}
```

```text
assertion `left == right` failed
  left: 0.0
 right: 40.0
```

## Write enough code to make it pass

You might naturally write this:

```rust,ignore
pub fn perimeter(width: f64, height: f64) -> f64 {
    2 * (width + height)
}
```

And Rust will refuse:

```text
error[E0277]: cannot multiply `{integer}` by `f64`
 --> src/lib.rs:2:7
  |
2 |     2 * (width + height)
  |       ^ no implementation for `{integer} * f64`
```

Rust **never mixes numeric types implicitly** — not int with float, not `i32` with
`i64`. No silent promotion, no coercion; if you want to cross a type boundary, you
convert explicitly. Here the fix is just to write the literal as a float:

```rust,ignore
{{#include ../code/structs/v1/src/lib.rs:code}}
```

This strictness feels pedantic on line one of a geometry file. It stops feeling
pedantic the first time you remember that implicit numeric conversion is where a
lot of real-world bugs — truncated divisions, overflowing intermediates — come
from in other languages.

So far, so easy. Now write an `area(width, height)` function yourself, TDD cycle
and all. You should end up with the code above.

> A brief aside, since we're comparing floats with `assert_eq!`: float equality is
> bit-for-bit, and computed floats often aren't bit-identical to the literal you
> expect (`0.1 + 0.2 != 0.3`). It works in this chapter because our values are
> exact in binary. When you assert on genuinely computed floats, compare within a
> tolerance — `assert!((got - want).abs() < 1e-9)` — or use a crate like
> `approx`. File this away; it will matter in the [Maths](maths.md) chapter.

## Refactor

Our code does the job, but it doesn't contain anything explicit about rectangles.
An unwary developer might try to supply the width and height of a triangle to
these functions without realising they will return the wrong answer.

We could just give the functions more specific names, like `rectangle_area`. A
neater solution is to define our own *type* which encapsulates the concept.

We can create a simple type using a **struct** — a named collection of fields:

```rust,ignore
{{#include ../code/structs/v2/src/lib.rs:struct}}
```

Field names are `snake_case`, like everything else that isn't a type or a
constant. Now refactor the tests to use it — build a struct value by naming each
field:

```rust,ignore
{{#include ../code/structs/v2/src/lib.rs:test}}
```

A porting note: the Go version of this chapter initialises structs positionally —
`Rectangle{10.0, 10.0}` — and then, twenty pages later, refactors to named fields
for readability. **Rust doesn't have positional struct literals.** Fields are
named at every construction site, always. The readability refactor is mandatory
and already done.

Remember to run your tests before attempting to fix them — let the compiler tell
you what to change:

```text
error[E0061]: this function takes 2 arguments but 1 argument was supplied
  --> src/lib.rs:21:19
   |
21 |         let got = perimeter(&rectangle);
   |                   ^^^^^^^^^------------
   |                            ||
   |                            |expected `f64`, found `&Rectangle`
   |                            argument #2 of type `f64` is missing
```

Fix the functions — fields are accessed with `.`:

```rust,ignore
{{#include ../code/structs/v2/src/lib.rs:code}}
```

Note we take `&Rectangle` — the function only needs to *look at* the rectangle, so
the caller lends it rather than handing it over, exactly as with slices. Take
references by default when you only read; the full reasoning arrives in
[Ownership and borrowing](ownership-and-borrowing.md).

I hope you'll agree that passing a `Rectangle` conveys our intent more clearly.

Our next requirement is to calculate the area of circles.

## Write the test first

```rust,ignore
#[test]
fn area_of_a_circle() {
    let circle = Circle { radius: 10.0 };

    let got = area(&circle);
    let want = 314.1592653589793;

    assert_eq!(got, want);
}
```

## Try to run the test

```text
error[E0422]: cannot find struct, variant or union type `Circle` in this scope
```

## Write the minimal amount of code for the test to run and check the failing test output

Define `Circle`:

```rust,ignore
pub struct Circle {
    radius: f64,
}
```

Now try again — `area` still wants a `&Rectangle` and we're giving it a
`&Circle`. Some languages would let us declare `area` twice:

```rust,ignore
pub fn area(rectangle: &Rectangle) -> f64 { ... }
pub fn area(circle: &Circle) -> f64 { ... }
```

But like Go, Rust has no function overloading:

```text
error[E0428]: the name `area` is defined multiple times
  --> src/lib.rs:14:1
   |
10 | pub fn area(rectangle: &Rectangle) -> f64 {
   | ----------------------------------------- previous definition of the value `area` here
...
14 | pub fn area(circle: &Circle) -> f64 {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `area` redefined here
```

We have two choices:

- Put the two functions in different modules — overkill here.
- Define **methods** on our types instead.

### What are methods?

So far we have only been writing functions, but we've been *using* methods all
along — `numbers.iter()`, `name.is_empty()`, `sums.push(...)` are all methods:
functions attached to a type, called on an instance of it.

In Rust, methods live in an `impl` block. Change the tests first:

```rust,ignore
{{#include ../code/structs/v3/src/lib.rs:test}}
```

Run them:

```text
error[E0599]: no method named `area` found for struct `Rectangle` in the current scope
  --> src/lib.rs:21:29
   |
 1 | pub struct Rectangle {
   | -------------------- method `area` not found for this struct
```

I would like to reiterate how great the compiler is here. It is so important to
take the time to slowly read the error messages you get; it will repay you.

## Write enough code to make it pass

```rust,ignore
{{#include ../code/structs/v3/src/lib.rs:code}}
```

New things:

- **`impl Rectangle`** opens a block of functions associated with `Rectangle`. The
  struct holds the data; the `impl` block holds the behaviour. They're written
  separately, and a type can have several `impl` blocks.
- **`&self`** is the receiver. It's short for `self: &Rectangle` — the method
  borrows the instance it's called on, for the same reason our functions took
  `&Rectangle`. Where Go lets you name the receiver (`func (r Rectangle)`) and
  other languages give you an implicit `this`, Rust always calls it `self` and
  makes you write it in the signature — its form tells you whether the method
  reads (`&self`), mutates (`&mut self`), or consumes (`self`) the value. That
  one-character difference will carry a lot of weight in later chapters.
- **π** comes from the standard library: `std::f64::consts::PI`.

The tests pass.

## Refactor

There is some duplication in our tests: both build a shape, call `.area()`, and
compare. And there's a design smell in the code: `Rectangle` and `Circle` both
*happen* to have an `area` method, but nothing says they're related.

What we want is to write a `check_area` helper that accepts both rectangles and
circles — but fails to compile if you hand it something that isn't a shape. In
Rust you codify this with a **trait**:

```rust,ignore
{{#include ../code/structs/v4/src/lib.rs:trait}}
```

A trait declares a capability: "a Shape is anything with an `area` method
returning `f64`". It's Go's `interface`, C#'s interface, Haskell's typeclass —
Rust's version of [ad hoc polymorphism](https://en.wikipedia.org/wiki/Ad_hoc_polymorphism).

Now the helper:

```rust,ignore
{{#include ../code/structs/v4/src/lib.rs:test}}
```

`&impl Shape` reads as: "a reference to *some* type that implements `Shape` — I
don't care which". The helper is *decoupled* from the concrete types; it knows
only the one thing it needs.

Run the tests:

```text
error[E0277]: the trait bound `Rectangle: Shape` is not satisfied
```

### Wait, what?

Here is this chapter's biggest divergence from its Go original, and it's a genuine
philosophical difference between the languages, so let's take it slowly.

In Go, interface satisfaction is **implicit**. The moment `Rectangle` has an
`Area() float64` method, it *is* a `Shape` — nobody declares anything. The Go book
presents this, fairly, as a superpower.

In Rust, trait implementation is **explicit**. Having an `area` method is not
enough; you must say the words:

```rust,ignore
{{#include ../code/structs/v4/src/lib.rs:impls}}
```

`impl Shape for Rectangle` — "Rectangle implements Shape, and here is how". (Note
we've moved the method bodies *into* the trait impls; the plain `impl Rectangle`
block is gone, since `Shape` now expresses the relationship we actually meant.)

What do you get in exchange for the extra line?

- **Implementing a trait is a promise, not a coincidence.** In Go, a type can
  satisfy an interface by accident, and code you've never seen can treat your
  type as something you never intended it to be. In Rust, if `Rectangle` is a
  `Shape`, it's because someone decided it should be, and there's a line of code
  with a name on it saying so.
- **The trait can be implemented for types you don't own.** You can
  `impl Shape for` a type from someone else's crate — try adding methods to
  someone else's struct in most languages.
- **The compiler can say exactly what's missing.** Watch what happens with
  Triangle below.

The tests pass. (If you're wondering why `check_area` could compile *before* any
type implemented `Shape` — a trait with no implementors is fine; it's a promise
nobody has made yet.)

## Further refactoring

Now that you have some understanding of structs, we can introduce **table-driven
tests** — useful when you want a list of cases all tested in the same manner.

Go builds its table with an anonymous struct. Rust's closest natural fit for a
small table is a slice of tuples, with one new type in the annotation:

```rust,ignore
let area_tests: &[(&dyn Shape, f64)] = &[
    (&Rectangle { width: 12.0, height: 6.0 }, 72.0),
    (&Circle { radius: 10.0 }, 314.1592653589793),
];

for (shape, want) in area_tests {
    assert_eq!(shape.area(), *want);
}
```

`&dyn Shape` is the second way to say "some Shape" — and it's *different* from
`&impl Shape` in an important way. `&impl Shape` means "one specific
shape type, the compiler knows which"; every call site gets its own compiled
version, so a *collection* of `&impl Shape` would still be a collection of one
single type. `&dyn Shape` means "any shape, decided **at runtime**" — the `dyn` is
honest labelling for a *dynamic* dispatch through a lookup table, which is what
lets a rectangle and a circle sit in the same list. It's what Go does for every
interface value; Rust makes you choose it, and label it, at the one place you need
it.

You can see how easy it would be for a developer to introduce a new shape,
implement `area`, and add it to the cases. Let's do exactly that: a triangle.

## Write the test first

Add `(&Triangle { base: 12.0, height: 6.0 }, 36.0)` to the list.

## Try to run the test

```text
error[E0422]: cannot find struct, variant or union type `Triangle` in this scope
```

Define it:

```rust,ignore
#[derive(Debug)]
pub struct Triangle {
    base: f64,
    height: f64,
}
```

Try again:

```text
error[E0277]: the trait bound `Triangle: Shape` is not satisfied
  --> src/lib.rs:33:26
   |
33 |             ("triangle", &Triangle { base: 12.0, height: 6.0 }, 36.0),
   |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `Shape` is not implemented for `Triangle`
help: the trait `Shape` is implemented for `Rectangle`
```

This is the explicitness paying rent: the compiler names the missing promise
*and* lists who has made it. Add an empty implementation to get the test running:

```rust,ignore
impl Shape for Triangle {
    fn area(&self) -> f64 {
        0.0
    }
}
```

The code compiles and we get our failing test. Fix it:

```rust,ignore
impl Shape for Triangle {
    fn area(&self) -> f64 {
        (self.base * self.height) * 0.5
    }
}
```

And our tests pass!

## Make sure your test output is helpful

Suppose a bug creeps in and one of twenty table cases starts failing:

```text
assertion `left == right` failed
  left: 0.0
 right: 36.0
```

*Which case?* The developer has to go hunting. Two fixes, both cheap.

First, name the cases — add a `&str` to each tuple. Second, print the shape itself
in the failure. That means every `Shape` needs to be Debug-printable, and traits
can *require* that of their implementors — a **supertrait**:

```rust,ignore
{{#include ../code/structs/v5/src/lib.rs:trait}}
```

`Shape: Debug` reads "to be a Shape you must also be Debug". And `#[derive(Debug)]`
— which we've been quietly attaching to structs above — is how a struct gets
`Debug`: you *derive* it, and the compiler writes the boring implementation for
you. Derives are everywhere in real Rust; this is the first, not the last.

The final test:

```rust,ignore
{{#include ../code/structs/v5/src/lib.rs:test}}
```

Now a failure identifies itself completely — name, fields, and values:

```text
assertion `left == right` failed: triangle: Triangle { base: 12.0, height: 6.0 }
  left: 0.0
 right: 36.0
```

(Go gets the case names bonus of running one case by name, via `t.Run` subtests.
Rust's tests-are-functions model doesn't subdivide that way — if a table grows
enough that you want to run cases individually, that's usually the sign it should
become individual `#[test]` functions.)

## Wrapping up

This was more TDD practice, iterating on basic maths problems and learning new
language features motivated by our tests:

- **Structs**: your own data types, with named fields — always named
- **Methods** in `impl` blocks, and `&self` — the receiver whose form (`&self` /
  `&mut self` / `self`) declares what the method does to the value
- **Traits** for ad hoc polymorphism — declared like Go interfaces, but
  implemented *explicitly* with `impl Trait for Type`
- Two ways to accept "any Shape": `&impl Shape` (one type, resolved at compile
  time) and `&dyn Shape` (any type, dispatched at runtime, mixable in
  collections)
- **Supertraits** (`Shape: Debug`) and **`#[derive(Debug)]`**
- Table-driven tests, and failure messages that identify the failing case

Defining your own types is essential for building software that is easy to
understand and to test — and traits are how those types advertise what they can
do. As you become more familiar with Rust you'll find the standard library is
organised around a small set of traits used *everywhere* — `Debug`, `Clone`,
`Iterator`, `From`, `Display` — and implementing them for your own types buys you
enormous amounts of free functionality. That story continues in
[Traits and generics](traits-and-generics.md).
