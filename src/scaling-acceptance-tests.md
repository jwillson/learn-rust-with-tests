# Scaling acceptance tests

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/scaling-acceptance)**

The [previous chapter](intro-to-acceptance-tests.md) sold you on acceptance
tests. This one is about the trap they set as they multiply. Write enough of
them the naive way — each one spinning up a server, poking raw HTTP, asserting
on response bodies — and you get a suite that is slow, repetitive, and, worst
of all, *coupled to how the system is built rather than what it does*. Change a
URL or a response shape and dozens of tests break for reasons that have nothing
to do with behaviour changing.

The fix, drawn from Dave Farley's work on acceptance testing, is a separation
of concerns into three layers:

- **Specification** — *what* the system does, in domain terms, coupled to
  nothing. "Greeting someone returns a hello."
- **Driver** — *how* you talk to one particular incarnation of the system: over
  HTTP, through a browser, by calling a function directly.
- **Test** — glue that manages a real system's lifecycle and plugs a driver
  into a specification.

Get this right and specifications change *only when behaviour changes*, while
implementation churn is absorbed by drivers. That stability is what lets an
acceptance suite grow without becoming a millstone.

## The specification is a trait

Here's the whole idea in miniature. The domain is trivial — a greeter — so the
mechanics stand out. The specification is a reusable function that expresses the
behaviour against an *interface*, knowing nothing about how greeting actually
happens:

```rust,ignore
{{#include ../code/scaling-acceptance/v1/src/lib.rs:specification}}
```

`Greeter` is the specification's contract; `greet_specification` is the
behaviour. Note what it *doesn't* mention: no HTTP, no URLs, no server. It just
says "a greeter greets with `Hello, world`". Rust's traits are a natural fit
here — the specification depends on the trait, and anything that implements the
trait can be checked against it.

(One Rust wrinkle worth naming: the trait uses `async fn`, and the compiler's
`async_fn_in_trait` lint warns that a public async-trait method can't promise
its future is `Send`. For a specification driven by static dispatch in tests,
that's irrelevant, so we `#[allow]` it with a comment. It's a good example of a
lint that's *usually* right flagging a case where you genuinely know better.)

## Two drivers, one specification

Now the payoff. The domain logic — the *essential complexity* — is a plain
function, and an in-process driver wraps it:

```rust,ignore
{{#include ../code/scaling-acceptance/v1/src/lib.rs:domain}}
```

The HTTP incarnation is an axum route serving that same function:

```rust,ignore
{{#include ../code/scaling-acceptance/v1/src/lib.rs:server}}
```

And an HTTP driver knows how to *speak to* that server — it translates the
specification's `greet()` into a real HTTP request and pulls the greeting out of
the response:

```rust,ignore
{{#include ../code/scaling-acceptance/v1/src/lib.rs:driver}}
```

The driver is the *only* place that knows about sockets, request lines, and
where the body starts. If the API's shape changes, this one obvious file
changes — not every test.

Now the same specification verifies both incarnations. The in-process test is
essentially a unit test — instant feedback that the essential complexity is
correct:

```rust,ignore
{{#include ../code/scaling-acceptance/v1/src/lib.rs:domain_test}}
```

The HTTP test is a full acceptance test — spin up the real server on a
throwaway port, point the driver at it, run the *identical* specification:

```rust,ignore
{{#include ../code/scaling-acceptance/v1/src/lib.rs:http_test}}
```

One specification, checked against the domain directly *and* over real HTTP.
Add a browser front end tomorrow and you'd write a third driver (a headless
browser) and reuse the same specification a third time.

## Why this scales

- **Specifications change for the right reason only** — a change in behaviour.
  Implementation detail (a URL, a header, a response format) lives in drivers,
  so it changes in one place.
- **Drivers are reusable.** As the suite grows, tests share drivers; a change
  to how you talk to the system is a single edit.
- **The same specification spans the test pyramid.** Run it in-process for
  fast unit-level feedback on your domain, over HTTP for acceptance-level
  confidence, and — because a driver can be made configurable — even against a
  staging or production environment to catch environment-specific problems.
- **It's a way to *start* work, not just verify it.** Expressing the behaviour
  you want as a specification *before* building anything focuses your intent,
  the essence of behaviour-driven development. You write the "what", then build
  drivers and implementation until it passes.

## Wrapping up

- **Separate specification, driver, and test.** The specification (a trait plus
  a behaviour function) says *what*; the driver says *how to talk to* one
  incarnation; the test wires them together and manages lifecycle.
- **Rust traits are the specification interface.** Implement the trait once per
  incarnation — in-process domain object, HTTP client, browser — and reuse the
  behaviour function across all of them.
- **This is what makes acceptance tests scale.** Coupling them to
  implementation detail is the number-one reason acceptance suites become
  expensive; pushing that detail into swappable drivers keeps your
  specifications stable and your suite maintainable as the system grows.

Next we look at what happens when we lean *too hard* on the test doubles this
book has used throughout — and how to test some kinds of code without mocks,
stubs, or spies at all.
