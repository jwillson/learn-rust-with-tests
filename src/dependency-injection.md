# Dependency injection

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/dependency-injection)**

It is assumed that you have read the [structs and traits](structs-methods-and-traits.md)
chapter, as some understanding of traits will be needed for this.

There are *a lot* of misunderstandings around dependency injection in the
programming community. Hopefully this chapter will show you how:

- You don't need a framework
- It does not overcomplicate your design
- It facilitates testing
- It allows you to write great, general-purpose functions

We want to write a function that greets someone, like we did in the
[hello-world chapter](hello-world.md) — but this time we are going to be testing
the *actual printing*.

Just to recap, here is what that function could look like:

```rust,ignore
fn greet(name: &str) {
    print!("Hello, {name}");
}
```

But how can we test this? Calling `print!` writes to stdout, which is pretty hard
for us to capture with the testing framework.

What we need is to be able to **inject** (which is just a fancy word for *pass
in*) the dependency of printing. Our function doesn't need to care *where* or
*how* the printing happens, so it should accept a **trait** rather than a
concrete destination.

We saw in Hello, World that `println!` and `print!` have a sibling, `format!`,
that returns the string instead. There's a fourth sibling, and it's the hook we
need — `write!`, which takes a *destination* as its first argument:

```rust,ignore
write!(some_destination, "Hello, {name}")
```

What can be a destination? Anything that implements the standard library's
[`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) trait —
Rust's version of Go's `io.Writer`, and the same "great general-purpose
interface for *put this data somewhere*" that the Go book is teaching here. As
you write more Rust you will find this trait popping up a lot.

Stdout implements it. So does a plain `Vec<u8>` — a growable byte buffer, which
is exactly the thing a test can capture and inspect.

## Write the test first

```rust,ignore
{{#include ../code/dependency-injection/v1/src/lib.rs:test}}
```

The buffer collects raw bytes, so at the end we turn them back into text with
`String::from_utf8` — which returns a `Result` (bytes might not be valid text),
so per the [Option chapter's](option-and-pattern-matching.md) guidance, in a
test we `.unwrap()`: if our greeting isn't valid UTF-8 the test *should* die
loudly. Same for the `Result` that `greet` itself will return — writing can
fail, and `write!` will tell us about it. In tests, unwrap; the failure is the
information.

## Try and run the test

```text
error[E0425]: cannot find function `greet` in this scope
```

## Write the minimal amount of code for the test to run and check the failing test output

*Listen to the compiler* and give the test the signature it's calling — but keep
the old `print!` body:

```rust,ignore
use std::io::Write;

pub fn greet(writer: &mut Vec<u8>, name: &str) -> std::io::Result<()> {
    print!("Hello, {name}");
    Ok(())
}
```

(The `use std::io::Write;` is needed before a `Vec` can act as a writer —
trait methods only exist where the trait is in scope. Forget it and the
compiler will tell you, by name.)

Run the test and enjoy one of `cargo test`'s better party tricks:

```text
---- tests::greet_writes_the_greeting stdout ----
Hello, Chris
thread 'tests::greet_writes_the_greeting' panicked at src/lib.rs:18:9:
assertion `left == right` failed
  left: ""
 right: "Hello, Chris"
```

The test fails — the buffer is empty — but look above the panic: **there's our
greeting**, in the "stdout" section of the failure report. Rust captures each
test's stdout and replays it when the test fails, so the report is literally
showing us the bug: the text went to the screen instead of to the writer we were
handed. The Go book has this same moment; Rust frames the evidence.

## Write enough code to make it pass

Use the writer:

```rust,ignore
{{#include ../code/dependency-injection/v1/src/lib.rs:code}}
```

`write!` is `print!` aimed at a destination of the caller's choosing, and it
returns `std::io::Result<()>` because real writing can fail — a full disk, a
closed socket. Our function passes that result straight back as its return
value; deciding what a write failure *means* is the caller's business, not the
greeter's.

The test passes.

## Refactor

The compiler-appeasing signature we wrote is technically correct and not very
useful. To see why, wire `greet` into an actual application:

```rust,ignore
fn main() -> std::io::Result<()> {
    greet(&mut std::io::stdout(), "Elodie")
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:8:11
  |
8 |     greet(&mut std::io::stdout(), "Elodie")
  |     ----- ^^^^^^^^^^^^^^^^^^^^^^ expected `&mut Vec<u8>`, found `&mut Stdout`
```

Of course: we hardcoded the *test's* writer into the signature. But `Vec<u8>`
and `Stdout` both implement `Write`, and `impl Trait` from the
[structs chapter](structs-methods-and-traits.md) says "any type that
implements":

```rust,ignore
{{#include ../code/dependency-injection/v2/src/main.rs:code}}
```

Now the function accepts *any* writer — the test injects a buffer, `main`
injects stdout, and `greet` can't tell the difference, which is the whole idea.

(Note `main` returning `std::io::Result<()>` — Rust's `main` is allowed to
return a `Result`, and an `Err` becomes a nonzero exit code with the error
printed. It's the grown-up version of crashing, and you'll see it in every CLI
chapter from here on.)

## More on Write

What other places can we send data? Just how general-purpose is our `greet`?

### The Internet

The Go book fires up an HTTP server here, using nothing but its standard
library. Rust's standard library doesn't include an HTTP server (the community
builds those as crates, and a later chapter reaches for one) — but it *does*
include TCP, and a TCP connection implements `Write`:

```rust,ignore
{{#include ../code/dependency-injection/v3/src/main.rs:code}}
```

Run the program, and in another terminal:

```sh
$ nc localhost 5001
Hello, world
```

Our greeter is a network service now. Nothing about `greet` changed — a
`TcpStream` is a writer, so it's a valid destination, the same as the test's
buffer and `main`'s stdout.

(Two small novelties in `main`, both previews: `?` after the fallible calls is
the error-propagation operator the [errors chapter](errors-and-result.md)
trailered — "if this failed, return the failure" — and `listener.incoming()` is
an iterator over connections, looped over like any collection. Networking
proper is a later chapter; the point here is only that our function followed
`Write` somewhere we never planned for.)

## Wrapping up

Our first round of code was not easy to test because it wrote data somewhere we
couldn't control.

*Motivated by our tests*, we refactored the code so we could control where the
data went by **injecting a dependency**, which allowed us to:

- **Test our code.** If you can't test a function *easily*, it's usually because
  of dependencies hard-wired into it or global state. If you have a global
  database connection used by some service layer, it will be difficult to test
  and slow to run. DI motivates you to inject the database (via a trait), which
  you can then substitute with something you control in your tests.
- **Separate our concerns**, decoupling *where the data goes* from *how to
  generate it*. If you ever feel like a function has too many responsibilities
  (generating data *and* writing to a database? handling HTTP *and* domain
  logic?), DI is probably the tool you need.
- **Allow our code to be reused in different contexts.** The first "new" context
  was the test. Then stdout, then a TCP socket, and the function never knew.

### What about mocking? I hear you need that for DI, and also it's evil

Mocking is covered [in the next chapter](mocking.md) (and it's not evil). You
use mocking to replace the real things you inject with pretend versions you can
control and inspect in tests. In our case, the standard library had a
test-double ready for us: an ordinary `Vec<u8>`.

### Study the standard library's traits

Knowing the `Write` trait let us use a byte buffer as a test double and a TCP
socket as a deployment target, with the same three-line function. Its sibling
`Read` is the other half of most I/O; the [reading files](reading-files.md) chapter leans on
it. The more familiar you are with these small, ubiquitous traits — `Write`,
`Read`, `Display`, `Iterator` — the more your own code will slot into contexts
you didn't design for.

This example is heavily influenced by a chapter in
[The Go Programming Language](https://www.amazon.co.uk/Programming-Language-Addison-Wesley-Professional-Computing/dp/0134190440),
by way of the Go original — if you enjoyed this, both are worth your money.
