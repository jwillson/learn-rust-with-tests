# Traits and generics

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/traits-generics)**

This chapter stands in for two chapters of the Go book at once, and the reason
why is worth a moment before we write any code.

The Go book has a **Generics** chapter, because generics arrived in Go 1.18
and needed introducing as the new toy. Rust has had generics from the
beginning — you've been using them since [Arrays and slices](arrays-and-slices.md)
(`Vec<T>`), and every `Option<T>`, `HashMap<K, V>`, `Mutex<T>` and
`Result<T, E>` since. What we haven't done is *write our own*, or looked
squarely at the machinery: type parameters, trait bounds, and generic data
structures. That's today.

The Go book also has a **Reflection** chapter, where `interface{}` (Go's
"could be anything" type) forces you to inspect values at runtime to find out
what they really are. Rust deliberately has no runtime reflection — and by
the end of this chapter you'll see how traits and generics do that chapter's
job at compile time instead.

We'll follow the Go book's plan: build our own test assertion helpers, then
a generic data structure.

## Our own test helpers

Yes, Rust already gives us `assert_eq!` — we've used it in every chapter.
We're going to build our own anyway, for the same reason the Go book builds
its own despite libraries existing: an assertion helper is the smallest
genuinely useful generic function there is, and rebuilding a tool teaches you
how the tool works.

### Assert on integers

Let's start with something basic and iterate toward our goal:

```rust,ignore
{{#include ../code/traits-generics/v1/src/lib.rs:code}}
```

Nothing new here — concrete `i32` parameters, a `panic!` to fail the test,
and `#[track_caller]` so the failure points at the caller, the trick we've
used for helpers since Hello, World.

```rust,ignore
{{#include ../code/traits-generics/v1/src/lib.rs:test}}
```

### Assert on strings

Being able to assert on the equality of integers is great, but what if we
want to assert on `&str`?

```rust,ignore
    #[test]
    fn asserting_on_strings() {
        assert_equal("hello", "hello");
        assert_not_equal("hello", "Grace");
    }
```

You'll get an error:

```text
error[E0308]: arguments to this function are incorrect
  --> src/lib.rs:27:9
   |
27 |         assert_equal("hello", "hello");
   |         ^^^^^^^^^^^^ -------  ------- expected `i32`, found `&str`
```

If you take your time to read the error, you'll see the compiler is
complaining that we're trying to pass a `&str` to a function that expects an
`i32`. This is type-safety doing its everyday job: by pinning down the types
a function works with, you **constrain the number of possible valid
implementations** and stop data going where it wasn't meant to. You can't
"add" a `Person` to a `BankAccount`; you can't capitalise an integer.
Constraints are helpful. We just want *slightly more* flexibility without
giving them up.

### The generic version

Ideally, we don't want a specific `assert_x` function for every type we ever
deal with — we want *one* `assert_equal` that works with any type, but does
not let you compare apples and oranges. Let's declare a *type parameter*:

```rust,ignore
#[track_caller]
pub fn assert_equal<T>(got: T, want: T) {
    if got != want {
        panic!("assertion failed");
    }
}
```

`<T>` reads as "for some type `T`, to be chosen by each caller" — the same
angle-bracket syntax as `Vec<T>`, now on our own function. `got` and `want`
are both `T`, so whatever type the caller picks, they must pick it for
*both* arguments. But the compiler isn't satisfied:

```text
error[E0369]: binary operation `!=` cannot be applied to type `T`
 --> src/lib.rs:3:12
  |
3 |     if got != want {
  |        --- ^^ ---- T
  |        |
  |        T
  |
help: consider restricting type parameter `T` with trait `PartialEq`
  |
2 | pub fn assert_equal<T: std::cmp::PartialEq>(got: T, want: T) {
  |                      +++++++++++++++++++++
```

This is the same complaint Go makes about `[T any]` — *operator `!=` not
defined for T* — and it's fair in both languages: if `T` can be truly
anything, it can be a type that has no notion of equality. What can our
function ask of a type it's never met? In Rust the answer is always the
same: **a trait**. `PartialEq` is the trait behind `==` and `!=` (we've
been `#[derive]`-ing it onto our structs since the
[Structs chapter](structs-methods-and-traits.md) precisely so `assert_eq!`
could compare them). Writing `T: PartialEq` is a *trait bound* — "any type
`T`, provided it implements `PartialEq`".

Follow the suggestion, and the compiler immediately teaches us the next
bound:

```text
error[E0277]: `T` doesn't implement `Debug`
 --> src/lib.rs:4:21
  |
4 |         panic!("got {got:?}, want {want:?}");
  |                     ^^^^^^^ `T` cannot be formatted using `{:?}` because it doesn't implement `Debug`
```

Of course — our panic message wants to *print* the values, and printing with
`{:?}` is also behaviour, also a trait: `Debug`. Bounds combine with `+`:

```rust,ignore
{{#include ../code/traits-generics/v2/src/lib.rs:code}}
```

Both test functions pass now, and a failure reports properly, pointing at
the right line:

```text
thread 'tests::asserting_on_strings' panicked at src/lib.rs:16:9:
got "hello", want "Grace"
```

Notice how the function body could be written *before* we knew what `T` was,
because the bounds tell us everything we're allowed to do with it: compare
it, print it, nothing else. That's the Go chapter's "constraints make
implementation simpler" point with the volume turned up — in Rust the
bounds are checked *against the function body itself*, not just the
callers. There is no way to write code inside `assert_equal` that does
something to `T` the signature didn't promise.

### Still no apples and oranges

The flexibility didn't cost us the constraint that matters:

```rust,ignore
assert_equal(1, "1");
```

```text
error[E0308]: mismatched types
  --> src/lib.rs:16:25
   |
16 |         assert_equal(1, "1");
   |         ------------ -  ^^^ expected integer, found `&str`
   |         |            |
   |         |            expected all arguments to be this integer type because they need to match the type of this parameter
```

One `T` per call. Both arguments must be it. The compiler even explains the
mechanism in the error.

(A style note: when the bound list gets long, Rust offers a second syntax
that moves it out of the angle brackets — `fn assert_equal<T>(got: T,
want: T) where T: PartialEq + Debug`. Same meaning; pick whichever reads
better.)

### What it costs: nothing

Go's chapter invites you to *imagine* the specialised implementations being
"somehow generated for you". In Rust that's not imagination, it's the actual
compilation strategy, and it has a name: **monomorphization**. For every
concrete `T` your program uses, the compiler stamps out a dedicated copy of
the function — `assert_equal::<i32>`, `assert_equal::<&str>` — each compiled
and optimised as if you'd written it by hand. Generic code runs exactly as
fast as the duplicated code it replaced. (The trade-off is compile time and
binary size — the copies are real.) This is the other half of the
`&impl Trait` vs `&dyn Trait` story from the Structs chapter: generics are
the compile-time, zero-cost road; `dyn` is the runtime-dispatch road for
when you genuinely don't know the type until the program runs.

## A generic data structure

Next we'll build a [stack](https://en.wikipedia.org/wiki/Stack_(abstract_data_type)):
push items on the top, pop them back off, last in first out. Following the
Go book — and its advice about not abstracting too early — we start
*concrete*, with a stack of integers and a stack of strings. (Like the Go
book, we'll spare you the blow-by-blow TDD of each method; you know the
drill by now. The tests are real and they drove the code.)

```rust,ignore
{{#include ../code/traits-generics/v3/src/lib.rs:code}}
```

And the tests — note that they get to use `assert_equal` from the previous
section, because our helpers were the seed of a little reusable assertions
library. (In the book's repo, this crate literally depends on the previous
one; helpers earn their keep fast.)

```rust,ignore
{{#include ../code/traits-generics/v3/src/lib.rs:test}}
```

Two Rust-flavoured notes before the refactor: `pop` returns `Option<i32>`
where Go returns `(int, bool)` — no need to invent a zero value for the
empty case, the type we've had since the
[Option chapter](option-and-pattern-matching.md) *is* the empty case. And
yes, our stack is a thin wrapper over `Vec` — a `Vec` genuinely is a stack
(`push`/`pop` are right there). The exercise is the point.

### Problems

- The code for both stacks is almost identical. More code to read, write and
  maintain.
- Duplicated logic means duplicated tests, too.

We want to capture the *idea* of a stack in one type, with one set of tests.

### The escape hatch we could reach for (don't)

Go's chapter shows the pre-generics workaround: a stack of `interface{}`,
where anything goes in and shapeless anythings come out. Rust *has* this
escape hatch — `Box<dyn Any>`, a boxed "some type, ask me at runtime" — and
it's instructive to see exactly the same misery play out. Here's a stack of
`Box<dyn Any>` in use:

```rust,ignore
    stack.push(Box::new(1));
    stack.push(Box::new(2));
    let first_num = stack.pop().unwrap();
    let second_num = stack.pop().unwrap();

    assert_eq!(first_num + second_num, 3);
```

```text
error[E0369]: cannot add `Box<dyn Any>` to `Box<dyn Any>`
  --> src/lib.rs:34:30
   |
34 |         assert_eq!(first_num + second_num, 3);
   |                    --------- ^ ---------- Box<dyn Any>
```

We put integers in, but the type doesn't remember, so we can't *use* them as
integers. The fix is Rust's version of Go's type assertion, and it is just
as unpleasant as Go's:

```rust,ignore
    let first_num = *stack.pop().unwrap().downcast::<i32>().unwrap();
    let second_num = *stack.pop().unwrap().downcast::<i32>().unwrap();
```

Every caller, every value, forever — checking at runtime for something the
compiler knew when we wrote `push(Box::new(1))` and then threw away. `Any`
and `downcast` are also as close as Rust gets to Go's reflection: a narrow,
deliberately awkward trapdoor for the rare cases that truly need it, not
a tool for everyday code. This is the Reflection chapter's whole plot in
four lines, and the moral is: don't erase types you'll want back.

### Generic data structures to the rescue

```rust,ignore
{{#include ../code/traits-generics/v4/src/lib.rs:code}}
```

The syntax is consistent with generic functions: `Stack<T>` declares the
parameter on the type; `impl<T> Stack<T>` says "here are the methods, for
every `T`". Note there are *no bounds* this time — a stack never compares or
prints its items, it just holds them, so it demands nothing of them. Ask
only for what you use.

The type of the stack now constrains what you can put in it and guarantees
what you get out:

```rust,ignore
{{#include ../code/traits-generics/v4/src/lib.rs:test}}
```

The last four lines are the payoff — `pop` returns `Option<i32>`, actual
integers, and `+` just works. No downcasts, no ceremony. And pushing
a string onto this stack of integers is a compile error, the same
apples-and-oranges wall `assert_equal` gave us.

One idiom to file away: usually `Stack::new()` needs no type spelled out,
because the compiler infers `T` from the first `push`. But an empty stack
you never push to gives it nothing to work with — then you say
`Stack::<String>::new()`. That `::<>` is affectionately called the
**turbofish**, and it's Rust's version of Go's explicit
`NewStack[string]()`:

```rust,ignore
{{#include ../code/traits-generics/v4/src/lib.rs:turbofish}}
```

With the generic stack in place we delete `StackOfStrings`, its tests, and
half the file. We haven't lost coverage; we've proven the logic once,
for every `T` at a stroke.

## Wrapping up

- **Type parameters** (`<T>`) give a function or struct a type chosen by
  each caller — one type per use, so flexibility never becomes mush.
- **Trait bounds** (`T: PartialEq + Debug`, or a `where` clause) declare
  what a generic function is allowed to *do* with its mystery type — and the
  compiler enforces them against the function body, not just the callers.
- **Monomorphization** makes generics zero-cost: real specialised copies,
  compiled per type. `dyn Trait` remains the tool for types unknown until
  runtime.
- **`Any`/`downcast` is Rust's reflection-shaped trapdoor** — and the
  reason this book has no Reflection chapter is that traits and generics
  remove almost every reason to open it.

### Resist premature generalisation

The Go book's closing advice survives translation intact, so we'll keep it:
don't reach for generics until you can *see* the duplication. We wrote
`StackOfInts` and `StackOfStrings` concretely, with tests, and only then
generalised — the abstraction was extracted from working code, not imagined
up front. A little duplication is better than coupling to a bad
abstraction; three copies is a decent rule of thumb for when it stops being
"a little". The red-green-refactor cycle is what makes the extraction safe:
the tests were passing before the refactor, and they were still passing
after, so the only thing that changed was the shape of the code.

And notice you were never really a stranger here: `Vec<T>`, `Option<T>`,
`HashMap<K, V>`, `Result<T, E>` — you've been a *consumer* of generic code
since chapter three. Today you became a producer.
