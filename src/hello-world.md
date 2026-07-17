# Hello, World

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/hello-world)**

It is traditional for your first program in a new language to be
[Hello, World](https://en.m.wikipedia.org/wiki/%22Hello,_World!%22_program).

Create a project wherever you like:

```sh
cargo new hello-world
cd hello-world
```

Cargo has already written the program for us, in `src/main.rs`. Replace it with
this — which is, near enough, what was already there:

```rust,ignore
{{#include ../code/hello-world/v1/src/main.rs}}
```

To run it, type `cargo run`.

## How it works

When you write a program in Rust, you have a `main` function, and that's where
execution starts. The `fn` keyword defines a function with a name and a body.

`println!` prints a line to stdout. That trailing `!` means it's a *macro*, not a
function. You don't need to care much about the difference yet, beyond noticing
that the `!` is not decoration — it's telling you this thing runs at compile time
and does something a function can't. In this case, checking your format string
against your arguments *before your program ever runs*. Keep an eye out for `!`;
Rust uses macros where other languages use runtime magic.

Notice there's no `import`. `println!` comes from the standard prelude, which is
in scope in every Rust file automatically.

## How to test

How do you test this? It is good to separate your "domain" code from the outside
world (side-effects). The `println!` is a side effect (printing to stdout), and
the string we send in is our domain.

So let's separate these concerns so it's easier to test.

```rust,ignore
{{#include ../code/hello-world/v2/src/main.rs:code}}
```

We've created a new function with `fn`, but this time we've added `-> String` to
the signature. That means this function returns a `String`.

### Wait — `String`, or `&str`?

Here is the first place Rust asks you a question Go doesn't. Go has one string
type. Rust has two, and you're going to meet both in the next ten minutes, so
let's get it over with:

- **`&str`** is a *borrowed* view of some text that already exists somewhere. A
  literal like `"Hello, world"` is a `&str`. It's a pointer and a length. It
  doesn't own anything, and it can't grow.
- **`String`** *owns* its text. It's heap-allocated, growable, and it's yours to
  do what you like with.

Our function builds a new string and hands it back to the caller, so it has to
return a `String` — there's no existing text to lend out a view of. That's why
`"Hello, world".to_string()` is there: it copies the literal into an owned
`String`.

The rule of thumb, which will serve you until the [Ownership and borrowing](ownership-and-borrowing.md)
chapter fills it in properly: **take `&str`, return `String`.** Accept the cheap
borrowed view, hand back the thing you made.

If this feels like a lot of ceremony for "hello world", that's fair. It is the
tax Rust charges up front, and the rest of the book is the refund.

## Writing the test

Now add a test. In Rust, unit tests live in the same file as the code they test,
at the bottom:

```rust,ignore
{{#include ../code/hello-world/v2/src/main.rs:test}}
```

Run `cargo test`. It should pass. Just to check, try deliberately breaking the
test by changing `want`.

Notice how you have not had to pick between multiple testing frameworks and then
figure out how to install them. Everything you need is built into the language and
the tooling, and the syntax is the same as the rest of the code you will write.
There's no `Cargo.toml` change, no dependency, no `#[test]` library to add.

### The rules

Writing a test is just like writing a function, with a few rules:

- It lives in a module annotated `#[cfg(test)]`. By convention that module is
  called `tests` and sits at the bottom of the file.
- The test function is annotated `#[test]`.
- It takes no arguments and returns nothing.
- It fails by panicking. That's it. No `t` to pass around.

Two of those deserve a moment.

#### `#[cfg(test)]`

`cfg` is conditional compilation. This attribute says "only compile this module
when running tests". Your test code — and anything it pulls in — is simply not
present in your release binary. Not stripped later, not dead code. Never compiled.

#### `use super::*;`

The `tests` module is a *child* module, and child modules don't automatically see
their parent's contents. `use super::*` pulls in everything from the parent so we
can call `hello()`.

This is also why the test can see private functions. `tests` is inside the module
it's testing, so it has access to its internals. Rust's answer to "how do I test
private functions" is: you're already inside.

#### `assert_eq!`

`assert_eq!(got, want)` panics if the two aren't equal, and prints both when it
does. There's no format string to write, no message to compose. You'll also see
`assert!(cond)` for booleans and `assert_ne!` for inequality.

We're using `got` and `want` as variable names to keep the test readable, and
because it's a good habit that will pay off in longer tests.

### Rust's documentation

Rust has excellent offline documentation, and you already have all of it:

```sh
rustup doc          # the whole standard library, in your browser, offline
rustup doc --std    # straight to std
cargo doc --open    # docs for *your* crate and its dependencies
```

That last one is worth remembering. It builds documentation for your project and
every crate you depend on, at exactly the versions you're using — not the latest
version some website is showing you. Online, [docs.rs](https://docs.rs) has the
same thing for every published crate.

The standard library documentation is unusually good, and nearly every entry has a
runnable example. Go and look at
[`str`](https://doc.rust-lang.org/std/primitive.str.html) now; you'll use it all
through this book.

## Hello, YOU

Now that we have a test, we can iterate on our software safely.

In the last example, we wrote the test *after* the code had been written, so that
you could get an example of how to write a test and declare a function. From this
point on, we will be *writing tests first*.

Our next requirement is to let us specify the recipient of the greeting.

Let's start by capturing these requirements in a test. This is basic test-driven
development, and it lets us make sure our test is *actually* testing what we want.
When you write tests retrospectively, you risk writing a test that keeps passing
even when the code doesn't work.

```rust,ignore
#[test]
fn saying_hello_to_people() {
    let got = hello("Chris");
    let want = "Hello, Chris";

    assert_eq!(got, want);
}
```

Run `cargo test`. You should get a compilation error:

```text
error[E0061]: this function takes 0 arguments but 1 argument was supplied
  --> src/main.rs:15:19
   |
15 |         let got = hello("Chris");
   |                   ^^^^^ ------- unexpected argument of type `&'static str`
   |
note: function defined here
  --> src/main.rs:1:4
   |
 1 | fn hello() -> String {
   |    ^^^^^
help: remove the extra argument
   |
15 -         let got = hello("Chris");
15 +         let got = hello();
   |
```

When using a statically typed language it is important to *listen to the
compiler*. The compiler understands how your code should snap together, so you
don't have to.

But look closely at that last bit, because it's a lesson worth learning early:

```text
help: remove the extra argument
```

The compiler is **wrong**. Not about the error — about what we should do. It has
no idea we're in the middle of adding a feature; it just sees a mismatch and
suggests the smaller edit. rustc's suggestions are unusually good and you should
read them, but they answer "how do I make this compile?", not "what am I trying to
build?" Only you know the second one.

We want the other fix: change `hello` to accept an argument.

```rust,ignore
fn hello(name: &str) -> String {
    "Hello, world".to_string()
}
```

Deliberately, we have *not* used `name` yet. We're making the compiler pass, not
the test. Run `cargo test` again and `main` will break, because it isn't passing an
argument:

```text
error[E0061]: this function takes 1 argument but 0 arguments were supplied
 --> src/main.rs:6:20
  |
6 |     println!("{}", hello());
  |                    ^^^^^-- argument #1 of type `&str` is missing
  |
help: provide the argument
  |
6 |     println!("{}", hello(/* &str */));
  |                          ++++++++++
```

Send in `"world"` to make it compile:

```rust,ignore
fn main() {
    println!("{}", hello("world"));
}
```

Now when you run your tests, you should see this:

```text
warning: unused variable: `name`
 --> src/main.rs:1:10
  |
1 | fn hello(name: &str) -> String {
  |          ^^^^ help: if this is intentional, prefix it with an underscore: `_name`

running 1 test
test tests::saying_hello_to_people ... FAILED

---- tests::saying_hello_to_people stdout ----
thread 'tests::saying_hello_to_people' panicked at src/main.rs:18:9:
assertion `left == right` failed
  left: "Hello, world"
 right: "Hello, Chris"
```

Two things happened, and both are useful.

The failure is the one we wanted: we have a compiling program that doesn't meet its
requirements, and the test says so, in terms we can read. `left` is what we got,
`right` is what we wanted.

But notice the **warning**. The compiler spotted that `hello` takes a `name` and
never uses it. It has, in effect, noticed that our implementation is a fake. This
is a small thing, and it is going to keep happening throughout this book: Rust
tends to notice the gap between what your code claims to do and what it does.

Let's make the test pass by actually using the argument.

```rust,ignore
fn hello(name: &str) -> String {
    format!("Hello, {name}")
}
```

`format!` builds a `String` using the same formatting machinery as `println!`, but
returns the string instead of printing it. That `{name}` inside the string
interpolates the variable directly.

When you run the tests, they should now pass.

### A note on source control

At this point, if you are using source control (which you should!) I would `commit`
the code as it is. We have working software backed by a test.

I *wouldn't* push to main though, because I plan to refactor next. It's nice to
commit here in case you get into a mess with the refactor — you can always go back
to the working version.

### Constants

There's not a lot to refactor, but we can introduce another language feature:
constants.

```rust,ignore
const ENGLISH_HELLO_PREFIX: &str = "Hello, ";
```

Constants must have their type written out — `&str` here — and by convention they
are `SCREAMING_SNAKE_CASE`. That convention is not merely cosmetic, and we'll see
why it earns its keep later in this chapter.

```rust,ignore
{{#include ../code/hello-world/v3/src/main.rs:code}}
```

After refactoring, re-run your tests to make sure you haven't broken anything.

It's worth thinking about creating constants to capture the meaning of values.

## Hello, world... again

The next requirement is that when our function is called with an empty string, it
defaults to `"Hello, World"` rather than `"Hello, "`.

Start by writing a new failing test:

```rust,ignore
#[test]
fn empty_string_defaults_to_world() {
    let got = hello("");
    let want = "Hello, World";

    assert_eq!(got, want);
}
```

If you've read the Go version of this book, this is where it reaches for subtests
with `t.Run`. Rust doesn't have subtests. The idiomatic equivalent is what we've
just done: **another `#[test]` function with a descriptive name**. Each test is
independent, they run in parallel by default, and the test name is the description.

Now let's fix the code:

```rust,ignore
{{#include ../code/hello-world/v4/src/main.rs:code}}
```

Run the tests: the new requirement is satisfied and we haven't broken the old one.

Look closely at that line, because it's doing two Rust things at once:

```rust,ignore
let name = if name.is_empty() { "World" } else { name };
```

**`if` is an expression.** It evaluates to a value, so you can assign it directly.
There's no ternary operator in Rust because there's no need for one. Note that
neither branch ends in a semicolon — that's what makes them the value of the block
rather than a statement.

**We're shadowing.** We declared a *new* `name` that hides the parameter. Rust
does this all the time and it isn't the mistake it would be elsewhere: the old
`name` is genuinely gone from this point on, so there's no chance of using the
wrong one by accident.

### A note on test helpers

The Go version of this chapter refactors its tests at this point, extracting an
`assertCorrectMessage` helper, because in Go each assertion is four lines of
`if got != want { t.Errorf(...) }`.

We don't need to. `assert_eq!(got, want)` already *is* that helper — it compares,
it fails, and it prints both values. The refactor has been done for us by the
standard library, so we'll skip it rather than invent an abstraction to have one.

When you *do* eventually write a custom assertion helper — and you will, in later
chapters — reach for this:

```rust,ignore
#[track_caller]
fn assert_greeting(got: &str, want: &str) {
    assert_eq!(got, want);
}
```

`#[track_caller]` is Rust's answer to Go's `t.Helper()`. Without it, a failure
reports the line number *inside the helper*, which is useless — every failure
points at the same line. With it, the panic reports the line that *called* the
helper, which is the line you actually want. File it away.

### Discipline

Let's go over the cycle again:

- Write a test
- Make the compiler pass
- Run the test, see that it fails, and check the error message is meaningful
- Write enough code to make the test pass
- Refactor

On the face of it this may seem tedious, but sticking to the feedback loop is
important.

Not only does it ensure you have *relevant tests*, it helps ensure *you design good
software* by refactoring with the safety of tests.

Seeing the test fail is an important check, because it lets you see what the error
message looks like. As a developer it is very hard to work with a codebase where
failing tests don't give a clear idea of what the problem is.

## Keep going! More requirements

Goodness me, we have more requirements. We now need to support a second parameter
specifying the language of the greeting. If a language is passed in that we don't
recognise, default to English.

Write a test for Spanish. Add it to the existing suite.

```rust,ignore
{{#include ../code/hello-world/v5/src/main.rs:spanish_test}}
```

Remember not to cheat! *Test first.* The compiler should complain, because you're
calling `hello` with two arguments rather than one:

```text
error[E0061]: this function takes 1 argument but 2 arguments were supplied
  --> src/main.rs:19:19
   |
19 |         let got = hello("Elodie", "Spanish");
   |                   ^^^^^           --------- unexpected argument #2 of type `&'static str`
```

Fix the compilation problem by adding another argument to `hello`:

```rust,ignore
fn hello(name: &str, language: &str) -> String {
    let name = if name.is_empty() { "World" } else { name };

    format!("{ENGLISH_HELLO_PREFIX}{name}")
}
```

Now it complains about your *other* callers:

```text
error[E0061]: this function takes 2 arguments but 1 argument was supplied
  --> src/main.rs:10:20
   |
10 |     println!("{}", hello("world"));
   |                    ^^^^^--------- argument #2 of type `&str` is missing
```

Fix them by passing empty strings. Now everything compiles and passes, apart from
our new scenario:

```text
---- tests::in_spanish stdout ----
thread 'tests::in_spanish' panicked at src/main.rs:22:9:
assertion `left == right` failed
  left: "Hello, Elodie"
 right: "Hola, Elodie"
```

We can use `if` to check the language and change the message:

```rust,ignore
{{#include ../code/hello-world/v5/src/main.rs:code}}
```

The tests should now pass. Notice we've captured the magic strings as constants
while we were there — `"Spanish"` appears once now, not once per branch.

### French

Your turn:

- Write a test asserting that if you pass in `"French"` you get `"Bonjour, "`
- See it fail, check the error message is easy to read
- Do the smallest reasonable change in the code

You should see the failure you'd expect:

```text
---- tests::in_french stdout ----
thread 'tests::in_french' panicked at src/main.rs:29:9:
assertion `left == right` failed
  left: "Hello, Lauren"
 right: "Bonjour, Lauren"
```

And you may have written something roughly like this:

```rust,ignore
{{#include ../code/hello-world/v6/src/main.rs:code}}
```

## `match`

When you have lots of `if` statements checking one value, reach for `match`.

```rust,ignore
{{#include ../code/hello-world/v7/src/main.rs:code}}
```

`match` looks like a `switch` from other languages, and for now you can read it as
one. But it is a considerably bigger idea, and this chapter is only showing you the
corner of it:

- **`match` is an expression.** It evaluates to a value, which is why we can write
  `let prefix = match language { ... }`. There's no mutable `prefix` variable being
  reassigned in branches, as there would be in Go's `switch`.
- **`match` is exhaustive.** The compiler checks that your arms cover every
  possible value, and refuses to compile if they don't. Here `language` is a `&str`,
  so there are infinite possibilities and we need the catch-all `_`. But when we
  start matching on our own types — in [Option and pattern matching](option-and-pattern-matching.md) — this
  becomes the feature that makes whole categories of bug impossible.

### A trap worth knowing about

Remember that convention about `SCREAMING_SNAKE_CASE` constants? Here's the payoff.

In a `match` arm, a bare lowercase identifier is not compared against — it's a
**binding**, which matches everything and captures the value. So if you typo a
constant's name, or use a lowercase one, the arm silently becomes a catch-all:

```rust,ignore
match language {
    SPANISH => "Hola, ",
    FRENCHH => "Bonjour, ",   // typo! this matches EVERYTHING
    _ => "Hello, ",
}
```

Rust catches this, but as a warning rather than an error:

```text
warning: unreachable pattern
 --> src/main.rs:8:9
  |
8 |         _ => "Hello, ",
  |         ^ no value can reach this
  |
note: multiple earlier patterns match some of the same values

warning: unused variable: `FRENCHH`
```

Two warnings, both pointing straight at it. This is why the CI for this book runs
`cargo clippy -- -D warnings`, turning warnings into errors: Rust's warnings are
usually not stylistic nagging, they're bugs that haven't happened yet.

## One... last... refactor?

You could argue our function is getting a little big. The simplest refactor is to
extract some of it into another function.

```rust,ignore
{{#include ../code/hello-world/v8/src/main.rs:code}}
```

A few things to note:

- **No `return`.** `greeting_prefix` is a single `match` expression, and the last
  expression in a function body is its return value. `return` exists in Rust, but
  it's for leaving *early*; using it at the end of a function marks you out as
  writing Go in Rust. (Go's answer to this — named return values — has no Rust
  equivalent, and isn't needed.)
- **`-> &'static str`.** Every prefix we return is a compile-time constant that
  lives for the entire run of the program, and `'static` is how you say that.
  Lifetimes are the third of Rust's big ideas and they get [their own chapter](lifetimes.md);
  for now, read `&'static str` as "borrowed text that will never go away".
- **Privacy.** Go marks a function public with a capital letter. Rust uses the `pub`
  keyword, and everything is private by default. `greeting_prefix` is an internal
  detail, so it stays private — which here means doing nothing at all.

Run `cargo clippy` when a refactor feels unfinished. It's a very good teacher and
it costs you nothing.

## Wrapping up

Who knew you could get so much out of `Hello, world`?

By now you should have some understanding of:

### Some of Rust's syntax around

- Writing tests: `#[cfg(test)]`, `#[test]`, `assert_eq!`
- Declaring functions, with arguments and return types
- `if` and `match` as *expressions*, `const`, shadowing
- `String` vs `&str`, and `format!`

### The TDD process, and *why* the steps are important

- *Write a failing test and see it fail* so we know we've written a *relevant* test
  and seen that it produces an *easy-to-understand description of the failure*
- Write the smallest amount of code to make it pass, so we know we have working
  software
- *Then* refactor, backed by the safety of our tests

In our case, we've gone from `hello()` to `hello("name")` to
`hello("name", "French")` in small, easy-to-understand steps.

Of course, this is trivial compared to "real-world" software. But the principles
stand. TDD is a skill that needs practice, and by breaking problems into small
testable components you'll have a much easier time writing software.

### And one thing specific to Rust

We saw the compiler catch a fake implementation (`unused variable: name`), suggest
a fix that was technically right and strategically wrong, and flag a typo'd match
arm that would have been a silent bug elsewhere.

That's the deal Rust offers, and it's the through-line of this book: you do more
work up front, and in exchange the compiler becomes a second pair of eyes that
never gets tired. TDD and Rust get along well, because they're the same bet — catch
it now, cheaply, rather than later, expensively.
