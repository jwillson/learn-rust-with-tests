# Lifetimes

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/lifetimes)**

This chapter has no counterpart in the Go book, because Go has no lifetimes —
its garbage collector keeps every value alive as long as anything points at
it, and cleans up afterwards. Rust made a different trade: no garbage
collector, and in exchange the compiler must be able to *prove*, at compile
time, that no reference ever outlives the data it points to. Lifetimes are
the vocabulary of that proof.

Here's the reassuring part: you've been using them all book. Every `&str`
you've passed, every `&self` on a method, every borrowed return value had
a lifetime — the compiler just never needed your help to work it out. Then it
started asking. In the [HashMaps](hashmaps.md) chapter, a free function
returning `Option<&str>` from two inputs stopped compiling until we dodged
into a method. In [Select](select.md), `racer` wouldn't compile until we
wrote `<'a>` and did what the compiler told us. This is the chapter where we
stop dodging and understand the question.

We'll build a couple of small text utilities for a blogging platform —
functions that hand out *views into* articles without copying them — and let
the compiler's questions drive, test-first as ever.

## Write the test first

First requirement: given the text of an article, return its first sentence.

```rust,ignore
    #[test]
    fn returns_everything_up_to_the_first_full_stop() {
        let article = "Ownership is the star of the show. Everything else orbits it.";

        let got = first_sentence(article);

        assert_eq!(got, "Ownership is the star of the show");
    }
```

## Try to run the test

The usual `E0425: cannot find function first_sentence` — so stub it:

```rust,ignore
pub fn first_sentence(text: &str) -> &str {
    ""
}
```

Notice the signature: borrowed text in, borrowed text out. No `<'a>` in
sight, and the compiler accepts it without complaint. Hold that thought.

```text
assertion `left == right` failed
  left: ""
 right: "Ownership is the star of the show"
```

## Write enough code to make it pass

```rust,ignore
{{#include ../code/lifetimes/v1/src/lib.rs:first_sentence}}
```

`find` returns an `Option<usize>` — the byte position of the first `'.'` if
there is one — and we `match` on it as usual: slice up to the full stop, or
hand back the whole text. Add a test for the no-full-stop case (it already
passes — the type made us handle it) and we're green.

The important thing about `&text[..position]` is what it *doesn't* do: no
copying. The returned `&str` is a view into the caller's own article — the
whole point of our little library. Which raises the question the compiler has
been quietly answering for us: *how long is that view safe to use?*

### The rule you've been benefiting from

Clearly, the returned slice is only good as long as the article it points
into. If Rust let you keep the slice after the article was gone, you'd have
a dangling reference — reading freed memory, the bug entire security
advisories are made of. So the compiler tracks, for every reference,
a **lifetime**: the region of code the borrow must remain valid for. When we
write no annotations, it fills them in by the *elision rules*, the most
important of which is:

> If there is exactly one input reference, the output reference borrows from
> it.

What the compiler actually recorded for our function was:

```rust,ignore
pub fn first_sentence<'a>(text: &'a str) -> &'a str
```

Read `'a` as "some region of code, to be determined at each call site" — the
tick syntax is a lifetime name, and by convention they're short: `'a`, `'b`.
The signature says: *the output is only valid while the input is.* One input
means one possible answer, so Rust doesn't make you write it. That's why an
entire book's worth of `&self` methods never asked: "borrowed from `self`"
is the elision rules' other favourite answer.

## The question the compiler can't answer alone

Next requirement: given two article excerpts, return the longer one.

```rust,ignore
    #[test]
    fn returns_the_longer_of_two_excerpts() {
        let got = longer("short", "a good deal longer");

        assert_eq!(got, "a good deal longer");
    }
```

The obvious implementation:

```rust,ignore
pub fn longer(a: &str, b: &str) -> &str {
    if b.len() > a.len() { b } else { a }
}
```

## Try to run the test

```text
error[E0106]: missing lifetime specifier
 --> src/lib.rs:1:36
  |
1 | pub fn longer(a: &str, b: &str) -> &str {
  |                  ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
help: consider introducing a named lifetime parameter
  |
1 | pub fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
  |              ++++     ++          ++          ++
```

There it is — the same error the HashMaps chapter ran away from. And now we
can read it properly. Two inputs, so the elision rules have no single answer:
does the output borrow from `a` or from `b`? *The compiler cannot know,
because it depends on which is longer at runtime.* It isn't asking us to
prove anything clever; it's asking us to state the contract.

The suggested fix — one lifetime parameter shared by all three references —
*is* the contract we mean:

```rust,ignore
{{#include ../code/lifetimes/v1/src/lib.rs:longer}}
```

In words: "there is some region `'a`; both inputs live at least that long;
the output is only promised for that long." At each call site the compiler
picks `'a` for us — the region where *both* arguments are still alive, since
the winner might be either. Green.

One thing lifetimes never do: change what the code does at runtime.
There's no lifetime data in the compiled program, nothing is kept alive
longer, nothing freed sooner. Annotations are purely descriptions, checked
and then discarded — which is why getting them wrong can never crash your
program, only fail your build.

## The contract is enforced at every call site

Annotations would be empty ceremony if nothing checked them. Watch what
happens to a caller that breaks the contract:

```rust,ignore
    #[test]
    fn the_winner_cannot_outlive_the_contestants() {
        let article = String::from("Ownership is the star of the show.");
        let winner;
        {
            let other = String::from("A somewhat longer article about borrowing.");
            winner = longer(&article, &other);
        }
        assert_eq!(winner, "A somewhat longer article about borrowing.");
    }
```

`other` is dropped when the inner block ends — it's a `String`, an owner, and
its scope is over. But `winner` might be borrowing from it, and we go on to
use `winner` afterwards:

```text
error[E0597]: `other` does not live long enough
  --> src/lib.rs:15:39
   |
14 |             let other = String::from("A somewhat longer article about borrowing.");
   |                 ----- binding `other` declared here
15 |             winner = longer(&article, &other);
   |                                       ^^^^^^ borrowed value does not live long enough
16 |         }
   |         - `other` dropped here while still borrowed
17 |         assert_eq!(winner, "A somewhat longer article about borrowing.");
   |         ---------------------------------------------------------------- borrow later used here
```

Declared here, dropped here, used here — the whole story in three
annotations. Our signature promised "the result lives only while both inputs
do", the caller tried to use the result after one input died, and the
compiler held the line. In a garbage-collected language this program would
work; in C it would compile and read freed memory; in Rust it's Tuesday.
(This test can't be in the book's test suite for the usual reason: it doesn't
compile. The crime is in the *caller* — `longer` itself is fine.)

## When borrowing is the wrong answer

New requirement: a function that returns an upper-case version of a headline.
Following today's borrow-everything pattern:

```rust,ignore
pub fn shout(text: &str) -> &str {
    let shouted = text.to_uppercase();
    &shouted
}
```

```text
error[E0515]: cannot return reference to local variable `shouted`
 --> src/lib.rs:3:5
  |
3 |     &shouted
  |     ^^^^^^^^ returns a reference to data owned by the current function
```

And no lifetime annotation can fix it, because the problem isn't
a description problem. `to_uppercase` can't edit the borrowed input; it
builds a brand-new `String`, owned by this function, destroyed when the
function returns. A reference to it would dangle — always, not just for
careless callers. The compiler is telling us our *design* is wrong: this
function doesn't hand out a view of existing data, it *creates* data, and
created data should be given away whole:

```rust,ignore
{{#include ../code/lifetimes/v2/src/lib.rs:shout}}
```

That's the deeper lesson of this chapter. A `&str` return says "I'm showing
you part of something you already own"; a `String` return says "I made this,
it's yours now". Lifetime errors are very often the compiler noticing that
you picked the wrong sentence — and E0515 means "you can't *show* someone
a thing that's about to be destroyed; *give* it to them."

## Structs that hold references

Last requirement: a `Preview` for article listings — a shouty title plus the
first sentence of the body. The title we build fresh (ownership, as we just
learned), but the excerpt should stay a zero-copy view into the article:

```rust,ignore
pub struct Preview {
    pub title: String,
    pub excerpt: &str,
}
```

```text
error[E0106]: missing lifetime specifier
 --> src/lib.rs:3:18
  |
3 |     pub excerpt: &str,
  |                  ^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
1 ~ pub struct Preview<'a> {
2 |     pub title: String,
3 ~     pub excerpt: &'a str,
  |
```

Same question, struct edition: if a `Preview` contains a borrow, then
a `Preview` *as a whole* can dangle, so the struct's type must carry the
borrow's lifetime. `Preview<'a>` reads as "a Preview that borrows from
something that lives for `'a`" — the struct can never outlive the article
its excerpt points into, and the compiler will enforce that against every
`Preview` anyone ever constructs.

```rust,ignore
{{#include ../code/lifetimes/v2/src/lib.rs:preview}}
```

The test-first habit still applies — here's the test that drove it:

```rust,ignore
{{#include ../code/lifetimes/v2/src/lib.rs:preview_test}}
```

Details worth noticing in `new`:

- `impl<'a> Preview<'a>` declares the lifetime once for the whole block,
  just like a function's `<'a>`.
- The two `&str` parameters are treated differently, and the signature
  documents it: `title: &str` (elided — we only *read* it to build our owned
  `String`) versus `body: &'a str` (named — the excerpt will keep borrowing
  from it). The lifetime annotation is doing real API-documentation work
  here: a caller can see at a glance that the `Preview` stays tied to
  `body` but is independent of `title`.
- `first_sentence` from earlier slots straight in — its elided signature
  unifies happily with `'a`.

## `'static`

One named lifetime you'll meet constantly: `'static`, the region that lasts
the whole program. String literals have it — `"hello"` is baked into the
binary, so `&'static str` is its full type, which is why literals can be
returned from anywhere and sent anywhere. You met `'static` as a demand in
the [Concurrency](concurrency.md) chapter: `thread::spawn` requires its
closure's captures to be `'static` precisely because a free-range thread
might run forever, and "forever" is the only lifetime long enough to be
safe. (That, remember, is why `to_string` appeared in `ping` — owned data
satisfies `'static`; borrows of a caller's locals don't.)

## Wrapping up

- A **lifetime** is the region of code a borrow must stay valid for. Every
  reference has one; you only *write* them when the compiler genuinely
  cannot infer who borrows from whom.
- The **elision rules** cover the common cases: one input reference, or
  a `&self` — which is why they've been invisible all book.
- **`E0106`** means "state the contract": say which inputs the output
  borrows from. **`E0597`** means a *caller* broke a stated contract.
  **`E0515`** means the design wants ownership, not borrowing — return the
  `String`, don't reference the local.
- **Structs that hold references carry lifetime parameters**, making "this
  value must not outlive that one" part of the type.
- Annotations **describe, never extend**: no runtime cost, no effect on
  when anything is dropped.

When a lifetime error has you cornered, the fix is almost always one of
three moves: *annotate* (tell the compiler which input the output follows),
*reorder* (keep the owner alive as long as its borrowers), or *own* (stop
borrowing and hand over a fresh value). We used all three today.
