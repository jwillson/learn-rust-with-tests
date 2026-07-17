# Cancellation-aware Reader

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/cancellation-aware-reader)**

The [Cancellation chapter](cancellation.md) showed how to make long-running work
stoppable by having it check a shared token. This short chapter applies that idea
to one of the most fundamental abstractions in the standard library — the `Read`
trait — and in doing so demonstrates the **decorator pattern**: wrapping an
existing thing to add behaviour without changing it.

## The idea

Reading can be slow. Pulling a large body off a socket, decoding a huge file —
these can take a long time, and if the reason for reading has gone away (the user
navigated off, the request timed out), we'd like to stop mid-read rather than
grind to the end. What we want is a `Read` that behaves exactly like whatever
reader it wraps, until a cancellation token flips, at which point reads fail.

Because it should be a drop-in for any reader, it must itself implement `Read`.
That's the decorator: same interface in, same interface out, extra behaviour in
the middle.

## Write the test first

First, that our wrapper is transparent — with no cancellation, it reads exactly
what the inner reader would:

```rust,ignore
{{#include ../code/cancellation-aware-reader/v1/src/lib.rs:normal_test}}
```

Note the inner reader is a `&[u8]` — the in-memory `Read` we've used since the
[Reading files](reading-files.md) chapter — and `read_to_string` is a method
that comes *free* on our wrapper the moment it implements `Read`, because
`read_to_string` is a default method built on `read`. Implement one method,
inherit the rest — the same dividend the [Iterators chapter](iterators-and-closures.md)
got from implementing `Iterator::next`.

Then the interesting case — reads work until we cancel, then fail:

```rust,ignore
{{#include ../code/cancellation-aware-reader/v1/src/lib.rs:cancel_test}}
```

## Write enough code to make it pass

We reuse the `CancellationToken` from the Cancellation chapter — `Arc<AtomicBool>`,
cloneable so the reader and the canceller share one signal:

```rust,ignore
{{#include ../code/cancellation-aware-reader/v1/src/lib.rs:token}}
```

And the decorator itself is small: hold the inner reader and the token,
implement `Read` by checking the token and then delegating:

```rust,ignore
{{#include ../code/cancellation-aware-reader/v1/src/lib.rs:reader}}
```

The whole design is in the `read` method. If the token is cancelled, return an
error — `ErrorKind::Interrupted`, the standard library's "this operation was
deliberately stopped" kind, so callers can distinguish cancellation from a real
I/O failure or from clean end-of-input. Otherwise, hand off to
`self.reader.read(buf)` and behave exactly as the wrapped reader does. That
delegation is the essence of a decorator: transparent by default, with just the
one added behaviour.

`CancellableReader<R>` is generic over the reader it wraps (`R: Read`), so it
decorates *anything* readable — a file, a socket, a `&[u8]`, even another
decorator — with no runtime cost, the monomorphization from the
[Traits and generics](traits-and-generics.md) chapter.

## Why a decorator over a `Read` is so powerful

The reason this is worth a chapter is the reach it gives you. Vast amounts of
code — JSON decoders, `read_to_string`, HTTP body parsers, decompressors — accept
*any* `impl Read` and don't care what's underneath. Slot a `CancellableReader`
in front of the real source and every one of them becomes cancellable, without a
single change to their code. You've added a cross-cutting capability by wrapping,
not by editing. That composability is exactly why programming to the standard
library's small traits (`Read`, `Write`, `Iterator`) pays off again and again.

## Wrapping up

- **The decorator pattern wraps a type in another that shares its interface**,
  adding behaviour transparently. A `CancellableReader<R>` is a `Read` that
  wraps a `Read`.
- **Implement one method, inherit the rest.** Providing `read` gave us
  `read_to_string`, `read_to_end`, and every other `Read` default method for
  free.
- **Cancellation is cooperative, still.** The reader checks a shared
  `CancellationToken` and returns `ErrorKind::Interrupted` when it's set —
  a distinct signal from EOF or a real error.
- **Decorating a standard trait composes everywhere.** Any code that consumes
  `impl Read` becomes cancellable by wrapping its source, with no changes to
  that code and no runtime overhead.
