# OS exec

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/os-exec)**

This short chapter answers a common question: *I have a function that runs an
external command and parses its output — how do I test it without actually
running the command?*

The instinct people reach for is a "test mode" flag: `get_data(mode)` that reads
a fixture file when `mode == "test"` and runs the real command otherwise. Resist
this. Baking test-awareness into production code is a smell, and it's almost
always a sign that two concerns are tangled together. The real fix is the one
this whole book keeps returning to: **separate the concerns, and depend on an
abstraction**.

## The tangle

Imagine a function that runs a command producing XML and shouts the message
inside it. Written as one lump, it does two unrelated jobs at once:

1. **Fetching** the raw XML — running a subprocess and capturing its output.
2. **Business logic** — parsing that XML and upper-casing the message.

Only the first job touches the operating system. The second is pure data
transformation, and it's the part you actually want to test. The reason the
whole thing is hard to test is that these two are welded together — you can't
exercise the parsing without spawning a process.

## Find the seam

The seam is the *stream of bytes* between fetching and parsing. In the
[Reading files](reading-files.md) chapter we learned that depending on `Read`
rather than a concrete source makes code testable, and it's the exact same move
here. Split the business logic into a function that takes any `impl Read`:

```rust,ignore
{{#include ../code/os-exec/v1/src/lib.rs:parse}}
```

`get_message` neither knows nor cares where its bytes come from. A subprocess's
stdout, a file, or a byte literal in a test — all satisfy `Read`. So the
business logic tests instantly, with a fixed payload and no subprocess anywhere
in sight:

```rust,ignore
{{#include ../code/os-exec/v1/src/lib.rs:test}}
```

The parsing and the upper-casing — the parts with actual logic worth
verifying — are now covered by fast, hermetic unit tests.

## The OS part, isolated

What's left, `get_data`, is the thin layer that touches the operating system,
and it does *only* that: spawn the command, hand its stdout to `get_message`:

```rust,ignore
{{#include ../code/os-exec/v1/src/lib.rs:exec}}
```

`Command::new("cat").arg("msg.xml").stdout(Stdio::piped()).spawn()` starts the
process and gives us a handle; `child.stdout.take()` is a `ChildStdout` — which
implements `Read`, so it drops straight into `get_message` as a *stream*,
exactly like Go's `StdoutPipe`. We read the message, then `child.wait()` to reap
the process. This layer is so thin there's barely any logic to get wrong, which
is the point: the risky, OS-dependent part shrank to almost nothing, and the
interesting logic moved somewhere testable.

Because the fetching layer is now trivial, an integration test that runs the
real command is cheap and worthwhile — proof the wiring works end to end:

```rust,ignore
{{#include ../code/os-exec/v1/src/lib.rs:integration_test}}
```

Notice the shape of the final design: a small, well-tested pure function doing
the real work, and a thin, dumb adapter connecting it to the outside world.
That's the same architecture as the [Reading files](reading-files.md) chapter
(pure parser, thin file-system layer) and the
[Scaling acceptance tests](scaling-acceptance-tests.md) chapter (pure
specification, thin driver). Difficult-to-test code is almost always a
concerns-not-separated problem in disguise.

## Wrapping up

- **A "test mode" flag in production code is a mistake.** It couples your code
  to your tests and signals tangled concerns.
- **Separate fetching from logic, and connect them with `Read`.** The business
  logic depends on `impl Read`, so it's tested with byte literals — no
  subprocess, no fixture files, no flakiness.
- **The OS-touching layer becomes thin enough to trust** with a single
  integration test, because all the logic that could be wrong now lives in the
  unit-tested pure function.
- **"Hard to test" usually means "concerns are coupled."** Find the seam —
  often a stream of bytes — split there, and depend on the abstraction.
