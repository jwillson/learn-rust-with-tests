# Property-based tests

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/roman-numerals)**

Some companies will ask you to do the [Roman Numeral
Kata](http://codingdojo.org/kata/RomanNumerals/) as part of the interview
process. This chapter will show you how to tackle it with TDD — and then use
it to introduce a new weapon: **property-based tests**, the payoff this
book's [Integers chapter](integers.md) promised long ago.

We are going to write a function which converts an Arabic number to a Roman
Numeral: 1 is `I`, 2 is `II`, 4 is `IV` (one-before-five), 1984 is
`MCMLXXXIV`. Number systems are a nice reminder that representing values in
convenient ways is an old, hard problem.

## Write the test first

By now the early gears of TDD should be muscle memory, so this chapter will
move quickly through them — the interesting scenery is later. Start small:

```rust,ignore
#[test]
fn converts_arabic_numbers_to_roman_numerals() {
    assert_eq!(convert_to_roman(1), "I");
}
```

Stub `pub fn convert_to_roman(arabic: u16) -> String` returning
`String::new()`, watch it fail with `left: "", right: "I"`, return `"I"`,
green. (Why `u16`? A Roman numeral can't be negative, which rules out the
signed types, and the [Integers chapter](integers.md) taught us to say what
we mean. Even `u16` is more than we need — hold that thought, it becomes
a plot point.)

Then `2` wants `"II"`, `3` wants `"III"`; the dumb-but-green implementation
grows `if` statements; the refactor spots the pattern —
`"I".repeat(arabic as usize)` — and then **4** arrives and breaks the whole
idea, because 4 is `IV`, and Roman numerals are not a tally, they're
a *subtractive* system. A few honest rounds of red-green later (5 is `V`,
9 is `IX`, 10 is `X`, 40 is `XL`... each new symbol first appearing as
a special case, each refactor step merging special cases into the loop) the
code converges on the insight the kata is designed to teach: the algorithm
is *data*. All that matters is the table of symbols, ordered
biggest-first:

```rust,ignore
{{#include ../code/roman-numerals/v1/src/lib.rs:code}}
```

Work through the number greedily: while it still contains a `1000`, append
`M` and subtract; then `900`/`CM`, and so on down. The subtractive forms
(`CM`, `XL`, `IV`...) are just *rows in the table*, not special cases in
code. Note `String` being the string-builder again, as the
[Iteration](iteration.md) chapter proved it is, and `mut arabic` — the
parameter is *ours* (pass-by-value), so consuming it as a countdown is
private arithmetic, invisible to the caller.

The tests that drove all this collapse nicely into a table too. Go uses
subtests generated in a loop; our Rust equivalent is a `const` table and
one loop, with the extra assert message naming the failing case:

```rust,ignore
{{#include ../code/roman-numerals/v1/src/lib.rs:test}}
```

Table-based cases like this cost one line each, so adding edge cases you're
nervous about is nearly free — note `3999`, the largest classic Roman
numeral. Remember that too.

## And back again: parsing

We're not done. Next we write the function that converts *from* a Roman
numeral to a number — and the test is almost too easy to write, because the
`CASES` table serves both directions:

```rust,ignore
{{#include ../code/roman-numerals/v2/src/lib.rs:test}}
```

The TDD path (take it yourself before reading on — dumb returns, then
noticing "count the characters" works right up until `IV` forces the real
algorithm) lands on `convert_to_roman` run in reverse: instead of
subtracting values while appending symbols, strip symbols while adding
values:

```rust,ignore
{{#include ../code/roman-numerals/v2/src/lib.rs:code}}
```

Where Go reaches for `strings.HasPrefix` + `strings.TrimPrefix`, Rust has
both in one method: `strip_prefix` returns `Option<&str>` — `Some(rest)` if
the prefix matched, with the prefix already removed. The `while let` loop
runs as long as the pattern matches, a construct you can now read as
"`match`, repeatedly". Twenty cases, both directions, all green.

## An intro to property-based tests

Step back and look at what we know about this domain that our tests *don't
say*:

- You can never have more than 3 consecutive identical symbols.
- Only I, X and C can be "subtractors".
- Converting a number to Roman and back again should give the original
  number.

Our table tests are **example-based**: they pin down twenty points in
a space of thousands. Property-based tests instead state the *rules* and
let the machine hurl randomised inputs at them. People fixate on the random
data, but that's not the hard part — the hard part, and the real value, is
being forced to understand your domain well enough to state its rules.

Go's standard library has `testing/quick`; Rust's standard library has
nothing built-in, and the community-standard crate is
[proptest](https://docs.rs/proptest) — our second dev-dependency after
criterion:

```toml
[dev-dependencies]
proptest = "1"
```

Let's state the "never four in a row" rule. Inside `proptest!`, a test
function's arguments become *generated inputs*:

```rust,ignore
proptest! {
    #[test]
    fn never_more_than_three_consecutive_identical_symbols(arabic: u16) {
        let roman = convert_to_roman(arabic);

        for symbol in ["I", "V", "X", "L", "C", "D", "M"] {
            let four_in_a_row = symbol.repeat(4);
            prop_assert!(!roman.contains(&four_in_a_row),
                "found {} in {}", four_in_a_row, roman);
        }
    }
}
```

`arabic: u16` means "any `u16` you like, proptest" — it will run the test
body against 256 random values. Run it:

```text
test tests::never_more_than_three_consecutive_identical_symbols ... FAILED

Test failed: found MMMM in MMMM at src/lib.rs:102.
minimal failing input: arabic = 4000
```

Two things deserve applause here. First, the test *found a real hole in our
domain thinking*, exactly as this technique promises: Roman numerals stop
at 3999 (`MMMCMXCIX`) — there is no standard way to write 4000 without
piling up `M`s — and nothing in our code knows that. Our choice of `u16`
was better than Go's `int` (their version of this test drowns in negative
numbers and quadrillion-character strings — their chapter carries an actual
warning that it may freeze your machine), but `u16` still reaches 65535,
and the type system can't express "at most 3999" for us.

Second, look at the failing input: **4000 exactly**. The random value
proptest first tripped over was almost certainly some ugly number like
`23941` — what it *reported* is the result of **shrinking**: when a case
fails, proptest automatically searches for the smallest input that still
fails, and hands you the boundary of your bug, not a random specimen of
it. (It also saves every failure to a `proptest-regressions` file — commit
those files, and past failures become permanent regression tests.)

### Encoding the domain in the generator

So what do we do about 4000? The kata's traditional answer is that the
domain of Roman numerals *is* 1 to 3999, and our job is to say so.
Proptest lets the generator carry that knowledge — replace the bare type
with a range strategy, and state the roundtrip property while we're at it:

```rust,ignore
{{#include ../code/roman-numerals/v3/src/lib.rs:properties}}
```

`arabic in 1..=3999u16` reads as "generate values in the domain" — note
`1`, not `0`: the Romans famously had no zero. Both properties green, 256
random values each, every run a different 256.

The roundtrip property is the one to remember. It ties `convert_to_roman`
and `convert_to_arabic` together: the only way it passes is if both are
correct — or share the *same* bug, which is possible but unlikely. One
property, four lines, exercising both functions against each other across
the entire domain. Compare the twenty rows of our example table. (Keep the
table anyway! Examples document *specific* truths — that 1984 is
`MCMLXXXIV` — and fail with friendlier messages. The two styles are
teammates, not rivals.)

If we wanted to go further, a `Result` from `convert_to_roman` for
out-of-domain inputs — or a newtype whose constructor enforces 1..=3999 —
would move the rule from the tests into the API, and after the
[Errors](errors-and-result.md) chapter you know exactly how. The kata
traditionally stops here, and so shall we.

## Wrapping up

More TDD practice, first — did you notice how *iterating on working code*
carried us? Each new case (4! 9! 40!) broke the current theory of the
problem; each refactor found a better theory; and at no point were we more
than one small test away from green. That rhythm is the kata's real
content. The Roman-numeral-specific insight — put the rules in data, run
the same table forwards and backwards — is a bonus.

On properties:

- **Property-based tests state domain rules; example tests state known
  points.** You want both. Writing properties forces the domain
  understanding that examples let you postpone.
- **proptest** generates randomised inputs (256 per run by default),
  **shrinks** failures to minimal inputs, and **records regressions** so
  a bug found once is checked forever.
- **Generators encode the domain**: `1..=3999u16` is a statement about
  Roman numerals, checkable and readable, living right in the test's
  signature.
- And the discovery workflow we lived through is the normal one: run
  a property over a *wider* space than you think valid, let it find the
  boundary, then decide — narrow the generator, or widen the code.
