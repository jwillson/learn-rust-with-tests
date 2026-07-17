# Ownership and borrowing

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/ownership)**

This chapter has no counterpart in the Go book, and it's worth a sentence on why.

The Go original teaches a chapter called *Pointers and errors*, built around a
`Wallet`. Its central drama is a program that compiles, runs, and silently does
nothing: a `Deposit` method mutates a *copy* of the wallet, the test fails
mysteriously, and the reader learns about pointers by debugging it with printed
memory addresses.

We're going to build the same wallet. But that drama cannot happen in Rust — the
broken version *doesn't compile* — and the reason it can't is Rust's core idea:
**ownership**. So where Go spends this chapter on pointers, we'll spend it on
what Rust made of them. Same wallet, deeper water.

**Fintech loves Rust** and uhhh bitcoins? So let's show what an amazing banking
system we can make.

## Write the test first

```rust,ignore
#[test]
fn deposits_into_the_wallet() {
    let wallet = Wallet::default();

    wallet.deposit(10);

    let got = wallet.balance();
    let want = 10;

    assert_eq!(got, want);
}
```

In the [previous chapter](structs-methods-and-traits.md) we accessed struct
fields directly, but in our *very secure wallet* we don't want to expose our
inner state to the world. We want to control access via methods.

`Wallet::default()` is new: it constructs a value with every field zeroed — Go
gives you this implicitly as the "zero value" (`Wallet{}`); Rust gives it to you
explicitly through the `Default` trait, which we'll derive.

## Try to run the test

```text
error[E0433]: failed to resolve: use of undeclared type `Wallet`
```

## Write the minimal amount of code for the test to run and check the failing test output

Tell the compiler what a `Wallet` is, and stub the methods:

```rust,ignore
#[derive(Default)]
pub struct Wallet {
    balance: u64,
}

impl Wallet {
    pub fn deposit(&self, amount: u64) {}

    pub fn balance(&self) -> u64 {
        0
    }
}
```

The tests now compile and fail for the right reason:

```text
assertion `left == right` failed
  left: 0
 right: 10
```

## Write enough code to make it pass

The obvious implementation:

```rust,ignore
pub fn deposit(&self, amount: u64) {
    self.balance += amount;
}
```

And the compiler says no:

```text
error[E0594]: cannot assign to `self.balance`, which is behind a `&` reference
 --> src/lib.rs:8:9
  |
8 |         self.balance += amount;
  |         ^^^^^^^^^^^^^^^^^^^^^^ `self` is a `&` reference, so it cannot be written to
  |
help: consider changing this to be a mutable reference
  |
7 |     pub fn deposit(&mut self, amount: u64) {
  |                     +++
```

Stop and appreciate what just happened, because this is the moment the Go
chapter and this one part ways.

In Go, `func (w Wallet) Deposit(amount int)` — a value receiver — compiles
happily, copies the wallet on every call, mutates the copy, and throws it away.
The test fails, the code looks right, and the reader has to go digging with
printed pointer addresses to discover the mutation vanished. The bug is real,
common, and *silent*.

In Rust, the receiver's form is a contract. `&self` says "I only look". The
compiler read our method body, saw a write, and refused — pointing at the exact
fix. Take it:

```rust,ignore
pub fn deposit(&mut self, amount: u64) {
    self.balance += amount;
}
```

`&mut self` says "I mutate the thing you called me on". Run the tests again —
and the compiler isn't finished with us:

```text
error[E0596]: cannot borrow `wallet` as mutable, as it is not declared as mutable
  --> src/lib.rs:24:9
   |
24 |         wallet.deposit(10);
   |         ^^^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
22 |         let mut wallet = Wallet::default();
   |             +++
```

The contract has two ends. The method demands mutable access; the *caller* must
hold a wallet that's allowed to change. Our test said `let wallet` — an immutable
wallet, per the [Iteration chapter](iteration.md) — so it has no mutable access
to give. Fix the test too:

```rust,ignore
{{#include ../code/ownership/v1/src/lib.rs:test}}
```

Now everything compiles, the test passes, and our career in fintech is secured.
Notice the chain of intent, readable at every level: the *variable* is declared
mutable, the *method* declares it mutates, and anyone reading the call
`wallet.deposit(10)` can trace both. Nothing changes in Rust without a paper
trail.

## What's actually going on: ownership

The two errors above are the visible edge of a system worth understanding
properly, and this is the right moment, so set the wallet down for a page. Rust's
memory model rests on three rules:

**1. Every value has exactly one owner.** When the owner goes out of scope, the
value is freed. This is how Rust has no garbage collector and also no manual
`free()` — cleanup happens at a point the compiler can prove.

**2. Assignment moves ownership.** Try this experiment in the test module:

```rust,ignore
#[test]
fn moving_a_wallet() {
    let wallet = Wallet::default();
    let wallet2 = wallet;

    assert_eq!(wallet.balance(), 0);
    assert_eq!(wallet2.balance(), 0);
}
```

```text
error[E0382]: borrow of moved value: `wallet`
  --> src/lib.rs:21:20
   |
18 |         let wallet = Wallet::default();
   |             ------ move occurs because `wallet` has type `Wallet`, which does not implement the `Copy` trait
19 |         let wallet2 = wallet;
   |                       ------ value moved here
20 |
21 |         assert_eq!(wallet.balance(), 0);
   |                    ^^^^^^ value borrowed here after move
   |
note: if `Wallet` implemented `Clone`, you could clone the value
```

`let wallet2 = wallet` didn't copy the wallet — it *moved* it. `wallet2` is the
owner now, and the old name is dead. In Go (and most languages) that line makes
a second wallet, and which one your code mutates becomes a thing you track in
your head. In Rust there is always exactly one, and the compiler tracks it for
you.

(Why didn't `amount` move when we passed it to `deposit`, or the `u64` returned
from `balance()`? Small, plain values like the number types implement **`Copy`**
— the error message above mentions it — and are duplicated instead of moved,
because copying them is as cheap as pointing at them. Structs don't get this
automatically; note the compiler suggesting `Clone` as the explicit way to
duplicate one.)

**3. You can borrow instead of owning.** That's what references are: `&wallet`
lends read access, `&mut wallet` lends write access, and the loan has rules —
**any number of readers, or exactly one writer, never both at once**. Our
`&self` and `&mut self` receivers are these two kinds of loan. This
readers-XOR-writer rule is what makes the earlier errors principled rather than
pedantic: the compiler is guaranteeing that nothing you're reading can be
mutated behind your back — which, as a bonus, outlaws data races between threads
at compile time. That cheque gets cashed in the [Concurrency](concurrency.md) chapter.

That's the whole system: one owner, moves by default, borrows with rules. Every
mysterious Rust error you'll meet in the next hundred pages is one of these
three rules holding the line. And it's why the Go version of this chapter can't
happen here — Go quietly copied the wallet, and in Rust a wallet is either
*moved* (and the old one's gone), *borrowed* (and mutation is governed), or
*explicitly cloned*. Silently-mutating-a-copy isn't in the menu.

## Refactor

Back to the wallet. We said we were making a Bitcoin wallet, but so far it's a
`u64` wallet. `u64` works fine, but it doesn't *say* anything — and it will
happily let you deposit 10 of anything: satoshis, dollars, seconds.

Rust's tool for this is the same one Go reaches for — make a new type — with a
twist in the syntax. A **tuple struct** wraps existing types and names the
combination:

```rust,ignore
pub struct Bitcoin(pub u64);
```

This is the *newtype pattern*: `Bitcoin` is a distinct type with a `u64` inside,
accessed as `self.0` (tuple structs number their fields instead of naming them).
Construction looks like Go's conversion syntax: `Bitcoin(10)`.

Update `Wallet` so `balance` is a `Bitcoin`, update the test to
`wallet.deposit(Bitcoin(10))`, and run it:

```text
error[E0368]: binary assignment operation `+=` cannot be applied to type `Bitcoin`
 --> src/lib.rs:9:5
  |
9 |     balance += amount;
  |     -------^^^^^^^^^^
  |
note: an implementation of `AddAssign` might be missing for `Bitcoin`
```

Here's another honest divergence. In Go, `type Bitcoin int` inherits all of
`int`'s operators — you can `+=` Bitcoins immediately. In Rust, a new type starts
with **nothing**: no arithmetic, no equality, no printing. Every capability is a
trait, and you opt in.

That sounds like a chore, and then you remember what money is. Should you be able
to *multiply* Bitcoin by Bitcoin? Go's newtype says yes, ten-BTC-squared is fine.
Rust makes each operation a decision, and for domain types like money, most of
the decisions are "no".

The ones that are "yes" split into two kinds. The common traits have `derive`
macros — the compiler writes them:

```rust,ignore
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Bitcoin(pub u64);
```

One line buys printing (`Debug`), copying like a number (`Copy`, `Clone` — a
`Bitcoin` is just a `u64` in a coat, so it should behave like one when passed
around), comparison (`PartialEq`/`Eq` for `==`, which `assert_eq!` needs, and
`PartialOrd`/`Ord` for `<`, which the next chapter needs), and a zero value
(`Default`).

Arithmetic isn't derivable — the compiler won't guess what adding your types
means — so `+=` is a short impl of the `AddAssign` trait from `std::ops`:

```rust,ignore
{{#include ../code/ownership/v2/src/lib.rs:bitcoin}}
```

Operators in Rust are just traits with syntax attached: implement `AddAssign`
and `+=` works. Note what we did *not* implement: no `*`, no `/`. They aren't
meaningful for money, so they don't compile. The type system now knows more
about Bitcoin than Go's version ever will.

The wallet itself barely changes:

```rust,ignore
{{#include ../code/ownership/v2/src/lib.rs:wallet}}
```

## Display

Try deliberately breaking the test — assert we got `Bitcoin(20)` — and read the
failure:

```text
  left: Bitcoin(10)
 right: Bitcoin(20)
```

That's `Debug` formatting: fine for us, wrong for anyone showing a balance to a
user. Go's chapter implements the `Stringer` interface here; Rust's equivalent
is the `Display` trait, which powers `{}` the way `Debug` powers `{:?}`.
Without it:

```text
error[E0277]: `Bitcoin` doesn't implement `std::fmt::Display`
  |
5 |     format!("{b}")
  |              ^^^ `Bitcoin` cannot be formatted with the default formatter
```

So we say what displaying a Bitcoin means:

```rust,ignore
{{#include ../code/ownership/v3/src/lib.rs:display}}
```

The signature is a mouthful — take it on faith for now that `f` is where the
output goes and `write!` is `format!` aimed at it; rustdoc's `Display` page has
this exact template to copy. What you get: every `{}` in every `println!`,
`format!`, and assertion message now renders Bitcoin your way. Break the test
again with a custom message that uses it:

```text
assertion `left == right` failed: got 10 BTC want 20 BTC
  left: Bitcoin(10)
 right: Bitcoin(20)
```

Both faces at once — `Display` (`10 BTC`) speaking to humans in the message,
`Debug` (`Bitcoin(10)`) speaking to developers underneath.

## Withdraw

Deposit's opposite. Write the test first — a wallet that starts with 20, loses
10:

```rust,ignore
#[test]
fn withdraws_from_the_wallet() {
    let mut wallet = Wallet {
        balance: Bitcoin(20),
    };

    wallet.withdraw(Bitcoin(10));

    assert_balance(&wallet, Bitcoin(10));
}
```

You know the drill by now: `no method named withdraw`, stub it, watch `got 20
BTC want 10 BTC`, implement:

```rust,ignore
pub fn withdraw(&mut self, amount: Bitcoin) {
    self.balance -= amount;
}
```

(`-=` needs `SubAssign` — same shape as `AddAssign`, two lines, do it yourself.)

While we're here, the tests repeat the get-balance-and-compare dance, so this
chapter's version of the by-now-traditional refactor, `#[track_caller]` and all:

```rust,ignore
{{#include ../code/ownership/v3/src/lib.rs:test}}
```

Note `assert_balance(&wallet, ...)` — the helper only reads, so it *borrows*.
After this chapter, you know exactly what that `&` is doing there.

## A loose thread, on purpose

What should happen if you withdraw more than you have?

Right now: `self.balance -= amount` on a `u64` **underflows** — and if you recall
the [Integers chapter](integers.md), you know that means a panic in debug builds
and a silent wrap to roughly eighteen quintillion BTC in release. Neither is a
great look for a bank.

Our function has no way to say "no". It needs to return something that can be
either success or failure — and making callers *unable to ignore* the failure is
the subject of the [next chapter](errors-and-result.md).

## Wrapping up

### Ownership

- Every value has one owner; assignment and argument-passing **move** by default
- Number-like types are `Copy` and duplicate instead of moving; anything can opt
  in to explicit duplication with `Clone`
- Borrows lend access without moving: many `&` readers *or* one `&mut` writer
- Method receivers encode all this: `&self` reads, `&mut self` writes, `self`
  consumes
- Mutation requires agreement at both ends: a `&mut self` method *and* a `let
  mut` binding

### The newtype pattern

- `struct Bitcoin(pub u64)` makes a distinct type from an existing one
- Unlike Go's `type Bitcoin int`, it inherits *no* capabilities — you derive the
  common ones and implement the meaningful ones, and what you don't grant
  doesn't compile
- Operators are traits (`AddAssign` et al.), so your types can have exactly the
  arithmetic that makes sense

### Display

- `Display` is Go's `Stringer`: implement it once and `{}` renders your type
  properly everywhere, while `{:?}` keeps the developer view

The Go chapter ends by warning that its central lesson — check whether you were
handed a `nil` pointer before touching it — is one **the compiler won't help you
with**. Rust's references can't be null, so that warning has no translation. The
nearest thing Rust has to `nil` is `Option`, it's checked at compile time, and
it's two chapters away.
