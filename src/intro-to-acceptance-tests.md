# Introduction to acceptance tests

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/acceptance-tests)**

Almost every test in this book so far has been a **unit test**: fast, focused,
exercising a small piece in isolation, often with an injected test double. Unit
tests are the engine of fearless refactoring and good design, and we'll never
stop writing them. But they share a blind spot: by construction, none of them
proves that the *whole assembled system* actually works when it runs for real.

This chapter introduces the other end of the spectrum — **acceptance tests** —
using a problem that is genuinely hard to unit-test: shutting a server down
gracefully.

## The problem: graceful shutdown

When an orchestrator (Kubernetes, systemd, a deploy script) wants to stop your
server, it sends a signal — `SIGTERM` or `SIGINT`. A naive server dies
instantly, dropping any requests it was in the middle of answering. Users see
errors during every deploy. A *graceful* server does better: on the signal it
stops accepting new connections but lets the in-flight ones finish, then exits.

Here's the whole thing in axum:

```rust,ignore
{{#include ../code/acceptance-tests/v1/src/lib.rs:code}}
```

`slow_handler` sleeps before responding — standing in for real work, and giving
us a window in which to trigger a shutdown mid-request. The important line is
`.with_graceful_shutdown(shutdown)`: axum takes a *future*, and when that future
completes, it begins graceful shutdown — new connections refused, existing ones
drained. What that future *is* — a signal, a channel, a timer — is left to the
caller, which is exactly the injectable seam that makes this testable.

The production wiring waits on OS signals:

```rust,ignore
{{#include ../code/acceptance-tests/v1/src/main.rs:main}}
```

`tokio::select!` races the two signal futures — Ctrl+C and SIGTERM — and
completes as soon as *either* fires (the [Select chapter](select.md)'s idea,
now on futures). That completed future is what tells axum to drain.

## Why not just unit-test it?

You could unit-test pieces of this, but the behaviour that *matters* — "a
request already being served survives a shutdown signal" — is an emergent
property of the real server, real connections, and real signal handling all
working together. A unit test with mocks could pass while the assembled program
still drops requests. To be sure, we have to test the system *as a user's
orchestrator would experience it*.

## What acceptance tests are

Acceptance tests are **black-box tests**: the test has no access to the
system's internals, only its public surface — for a server, that means real HTTP
over a real socket. Because the test can't reach inside, it can't cheat; if it
passes, the system genuinely behaves as observed. The upsides:

- When they pass, you know the *whole* system works, not just its parts.
- No mocking — it's all real, so there's no risk of a double diverging from
  reality.
- Written well, they double as trustworthy, executable documentation.

And the honest downsides, which are why they're a complement to unit tests, not
a replacement:

- Slower to run and more expensive to write.
- When they fail they point at "something's broken" rather than the exact line.
- They say nothing about *internal* quality — you can pass an acceptance test
  with a horrible design underneath.

The standard mental model is the **test pyramid**: many fast unit tests at the
base, fewer acceptance tests at the top. Rely only on acceptance tests and your
suite is slow and your failures are vague; rely only on unit tests and you can't
prove the system works end to end.

## Writing the acceptance test

The manual check we'd otherwise do — start the server, curl it, hit Ctrl+C
mid-request, see if the response arrives — is exactly what we automate. In Rust
this fits in a `#[tokio::test]` because tokio lets us run the real server and a
real client in the same process, on a throwaway port. First a tiny raw-HTTP
client (raw TCP, so there's no framework between us and the wire — genuinely
black-box):

```rust,ignore
{{#include ../code/acceptance-tests/v1/src/lib.rs:helpers}}
```

Then the test itself follows the manual steps precisely:

```rust,ignore
{{#include ../code/acceptance-tests/v1/src/lib.rs:test}}
```

Read it as the story it tells:

1. Start the real server on an OS-assigned free port, its shutdown wired to a
   `oneshot` channel (our injectable "signal").
2. Fire off a request — the slow handler is now mid-flight.
3. *While it's in flight*, send the shutdown signal.
4. Assert the in-flight request still gets its complete `200 OK` /
   `hello, world` response — grace confirmed.
5. Assert the server task then returns cleanly (it drained and exited).
6. Assert a *new* connection is now refused — it really did stop.

No mocks, no internals — a real server, a real socket, a real shutdown,
observed from outside. If graceful shutdown regressed, this test would catch
what no unit test could.

## Wrapping up

- **Acceptance tests exercise the whole running system as a user would**,
  black-box, over its real public interface. They prove the assembled program
  works — the one thing unit tests structurally cannot.
- **They complement, not replace, unit tests.** Follow the test pyramid: lots
  of fast unit tests, a smaller number of acceptance tests for the critical
  end-to-end behaviours.
- **Some behaviour is only observable end-to-end** — graceful shutdown being a
  perfect example — and for those, an acceptance test is the *only* honest
  proof.
- **tokio makes server acceptance tests cheap in Rust**: run the real server
  and a real client in one `#[tokio::test]` on port 0, and injectable seams
  (a `shutdown` future here) let the test drive scenarios a signal normally
  would.

The next chapter builds on this: as acceptance tests multiply, their setup and
boilerplate can overwhelm you, so we'll look at how to *scale* them without
drowning in glue code.
