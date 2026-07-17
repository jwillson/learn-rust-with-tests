# Iteration

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/iteration)**

To do stuff repeatedly in Rust you have three tools: `loop`, `while`, and `for`.
We'll meet all three, but you'll spend nearly all your time with `for` — and Rust's
`for` is a different animal from the C-style loop you may be picturing. There is no
`for i := 0; i < 5; i++` in Rust. We'll see what there is instead.

Let's write a test for a function that repeats a character 5 times.

There's nothing new so far, so try and write it yourself for practice.

## Write the test first

```rust,ignore
{{#include ../code/iteration/v1/src/lib.rs:test}}
```

## Try and run the test

```text
error[E0425]: cannot find function `repeat` in this scope
 --> src/lib.rs:7:24
  |
7 |         let repeated = repeat("a");
  |                        ^^^^^^ not found in this scope
```

## Write the minimal amount of code for the test to run and check the failing test output

*Keep the discipline!* You don't need to know anything new right now to make the
test fail properly.

```rust,ignore
pub fn repeat(character: &str) -> String {
    String::new()
}
```

`String::new()` gives us an empty owned string — the least code that satisfies the
compiler. Isn't it nice to know you already know enough Rust to write tests for
some basic problems? This means you can now play with the production code as much
as you like and know it's behaving as you'd hope.

```text
---- tests::repeats_the_character stdout ----
thread 'tests::repeats_the_character' panicked at src/lib.rs:14:9:
assertion `left == right` failed
  left: ""
 right: "aaaaa"
```

## Write enough code to make it pass

Here is the natural first attempt — build up a string in a loop:

```rust,ignore
pub fn repeat(character: &str) -> String {
    let repeated = String::new();
    for _ in 0..5 {
        repeated.push_str(character);
    }
    repeated
}
```

Run it:

```text
error[E0596]: cannot borrow `repeated` as mutable, as it is not declared as mutable
 --> src/lib.rs:4:9
  |
4 |         repeated.push_str(character);
  |         ^^^^^^^^ cannot borrow as mutable
  |
help: consider changing this to be mutable
  |
2 |     let mut repeated = String::new();
  |         +++
```

Welcome to one of Rust's defining decisions: **variables are immutable by
default**. If you want to change something after creating it, you must say so, out
loud, with `mut`. This isn't the borrow checker being difficult — it's a
declaration of intent. When you read Rust, every variable that can change is
marked, so everything unmarked is guaranteed to be exactly what it was when it was
made.

The compiler's suggestion is exactly right this time. Take it:

```rust,ignore
{{#include ../code/iteration/v1/src/lib.rs:code}}
```

Run the test and it should pass. Some things worth noticing:

- **`0..5` is a range** — the values 0, 1, 2, 3, 4 (5 excluded; `0..=5` would
  include it). Rust's `for` doesn't count with a mutable index variable; it walks
  over anything iterable, and a range is iterable.
- **`_` is deliberate.** We don't use the loop counter, so we don't name it. If
  you'd written `for i in 0..5` without using `i`, the compiler would have warned
  you — the same "your code claims more than it does" instinct we saw in
  [Hello, World](hello-world.md).
- `push_str` appends a `&str` to the end of a `String`, which is exactly the
  take-`&str`, build-`String` division of labour from the last two chapters.

For completeness, the other two loops — `while` works as you'd expect, and `loop`
loops forever until you `break`:

```rust,ignore
while still_hungry {
    eat();
}

loop {
    poll();               // run until break
    if done { break; }
}
```

`loop` looks redundant (`while true` exists, after all) but it's actually the more
honest construct when you mean "forever": the compiler knows a `loop` without a
`break` never exits, and types the code after it accordingly. You'll see it again
in the concurrency chapters.

## Refactor

Time to refactor, and to introduce the `+=` operator, which works on `String` just
as you'd hope:

```rust,ignore
{{#include ../code/iteration/v2/src/lib.rs:code}}
```

`+=`, the *add-and-assign* operator, appends the right operand to the left. It
works with other types too, like integers.

Notice `REPEAT_COUNT` is a `usize`. When Rust counts things — lengths, indexes,
sizes — it uses `usize`, the pointer-sized unsigned integer. A range built from a
`usize` produces `usize` values. You'll get comfortable with this quickly, because
every `.len()` in the standard library returns one.

### Benchmarking

Here's where this chapter and the Go original part ways, in an instructive fashion.

The Go book uses this function to introduce benchmarks, and its punchline is that
string concatenation in a loop is slow — Go strings are immutable, so every `+=`
copies the whole string — and you should refactor to `strings.Builder`, for a
roughly 5× win.

**That problem does not exist in Rust.** A `String` you own and have marked `mut`
appends in place, growing its buffer as needed. Rust's `String` *is* the builder;
that's why it exists as a separate type from `&str` at all. The naive code you'd
naturally write is already the efficient code — the `mut` the compiler insisted on
earlier wasn't just ceremony, it's what makes in-place appending possible to
express.

But "trust me, it's fast" is exactly the kind of claim this book doesn't make, so
let's benchmark it. That story also diverges: Rust's built-in benchmark harness has
sat unstable on nightly for years, and the community standard is the
[criterion](https://docs.rs/criterion) crate. This is our first dependency — add it
to `Cargo.toml`:

```toml
[dev-dependencies]
criterion = "0.8"

[[bench]]
name = "repeat"
harness = false
```

`dev-dependencies` are for tests and benchmarks only — like `#[cfg(test)]` code,
they never touch your release binary or burden anyone who depends on your crate.
The `[[bench]]` section registers a benchmark target; `harness = false` says
criterion is driving, not the built-in harness.

Benchmarks live in a `benches/` directory next to `src/`. Create
`benches/repeat.rs`:

```rust,ignore
{{#include ../code/iteration/v3/benches/repeat.rs:bench}}
```

Note the `use iteration_v3::repeat;` — like a doctest, a benchmark lives *outside*
your crate and sees only its public API.

The shape inside is the same as Go's `b.Loop()`: you hand criterion a closure —
`|b| ...` is Rust's lambda syntax, of which much more in a later chapter — and it
decides how many times to run it to get statistically meaningful numbers. How many
times it runs shouldn't matter to you.

Run `cargo bench`:

```text
repeat                  time:   [31.448 ns 31.886 ns 32.395 ns]
```

Three numbers, not one: the bounds of criterion's confidence interval — read the
middle one as "about 32 nanoseconds per call". Criterion runs your code millions of
times, does real statistics, and warns you about outliers. It will even generate
HTML reports with plots under `target/criterion/` if you have gnuplot installed.

### The standard library already did it

One of this book's recurring lessons: before writing a function, have a look
through the standard library docs — `rustup doc --std` — and see if it's already
there. String repetition is:

```rust,ignore
{{#include ../code/iteration/v4/src/lib.rs:code}}
```

The tests still pass. Now run `cargo bench` again, and this is where criterion
quietly outclasses most benchmarking tools — **it remembers the previous run and
compares**:

```text
repeat                  time:   [23.089 ns 23.322 ns 23.567 ns]
                        change: [−27.616% −26.525% −25.380%] (p = 0.00 < 0.05)
                        Performance has improved.
```

About 26% faster, with a p-value telling you it's not noise. (`str::repeat` can
allocate the exact final size up front instead of growing as it goes.)

Notice the *shape* of the result, though: 32ns to 23ns. A nice win, but nothing
like the 5× the Go book gets from `strings.Builder` — because the loop version was
never broken. Measure before you optimise; your intuitions from other languages
may not survive contact with this one. And keep the benchmark workflow in mind:
run, change the code, run again, and let criterion tell you whether it mattered.

## Practice exercises

- Change the test so a caller can specify how many times the character is repeated,
  then fix the code. (What type should the count parameter be? You've met it in
  this chapter.)
- Add a doctest to `repeat` — you know how from [Integers](integers.md) — and check
  it shows up in `cargo doc --open`.
- Have a look through the docs for [`str`](https://doc.rust-lang.org/std/primitive.str.html)
  and its methods. Find functions you think could be useful and experiment with
  them by writing tests like we have here. Investing time learning the standard
  library will really pay off.

## Wrapping up

- More TDD practice
- `for`, ranges, and a nod to `while` and `loop`
- **`mut`**: variables are immutable by default, and changing one is something you
  declare, not something you just do
- How to write benchmarks with criterion, our first dependency — and why
  `dev-dependencies` cost your users nothing
- A performance lesson inverted: Rust's `String` appends in place, so the naive
  loop was already fast, and the benchmark proved it instead of us assuming it
