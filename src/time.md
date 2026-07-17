# Time

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/time)**

Our poker CLI records wins. Now the product owner wants it to run a real game:
as time passes, the *blinds* (forced bets) should increase on a schedule, and
the program should announce each rise. A tournament might start at a blind of
100 and climb to 8000 over a couple of hours.

The interesting problem here isn't the poker — it's **testing code that depends
on time**. We do not want a test that waits ten real minutes to check the
second blind. The answer is the same lesson the whole application section keeps
teaching: don't call the clock directly, *inject* the thing that schedules
against it, and in tests substitute a spy that just records what *would* have
been scheduled.

## Write the test first

We want the `Cli` to schedule a series of blind alerts when a game starts. So
we invent a `BlindAlerter` — an abstraction for "please announce this amount
after this much time" — and inject it, exactly like the `PlayerStore`:

```rust,ignore
{{#include ../code/time/v1/src/lib.rs:alerter}}
```

`Duration` is `std::time::Duration`, the standard type for a span of time —
Go's `time.Duration`. The spy implements the trait by recording each scheduled
alert instead of acting on it:

```rust,ignore
{{#include ../code/time/v1/src/lib.rs:spy}}
```

Then a table-based test states the whole schedule as data — the blind rises to
100 immediately, 200 at ten minutes, and so on up to 8000 at 100 minutes — and
asserts the spy captured exactly that:

```rust,ignore
{{#include ../code/time/v1/src/lib.rs:schedule_test}}
```

Because `ScheduledAlert` derives `PartialEq` and `Debug`, we can build the
entire expected `Vec` and compare in one `assert_eq!` — no per-row loop, and a
mismatch prints both full lists. This is the [Structs chapter](structs-methods-and-traits.md)'s
derived equality doing the heavy lifting for a table test.

## Write enough code to make it pass

The `Cli` grows an `alerter` field, and `play_poker` schedules the blinds
before reading the winner:

```rust,ignore
{{#include ../code/time/v1/src/lib.rs:cli}}
```

`schedule_blind_alerts` walks the blind amounts, calling the alerter with a
`blind_time` that starts at `Duration::ZERO` and advances ten minutes each
round (`blind_time += Duration::from_secs(10 * 60)`). The test passes
*instantly* — no clock ran, we only checked that the right schedule was handed
to the alerter. That's the entire point: by depending on the `BlindAlerter`
trait rather than on real timers, we made time-dependent behaviour testable in
microseconds.

## The real alerter

The spy proves the scheduling logic. For the actual program we need a real
`BlindAlerter` — one that genuinely waits and then prints:

```rust,ignore
{{#include ../code/time/v1/src/lib.rs:stdout_alerter}}
```

It spawns a thread that sleeps for the duration and then announces the blind —
the [Concurrency chapter](concurrency.md)'s `thread::spawn` plus
`thread::sleep`, `move`-ing the `amount` into the closure so the thread owns
what it needs. In `main` you'd inject a `StdoutBlindAlerter`; in tests, the
spy. Same `Cli::play_poker`, two very different alerters — dependency injection,
one more time.

(This is deliberately the *simplest* real alerter, a fire-and-forget thread per
blind. A production version might use tokio timers or a cancellable schedule —
and the [Cancellation chapter](cancellation.md) showed how you'd stop pending
work cleanly — but the injected-trait seam is what matters, and it doesn't
change.)

## Wrapping up

- **Never let testable logic call the clock directly.** Inject an abstraction
  — here `BlindAlerter` — and your tests substitute a spy that records
  *intended* timing instead of waiting for real time to pass. An 11-blind,
  100-minute schedule is verified in an instant.
- **`std::time::Duration`** is the currency of time spans; build them with
  `from_secs`, add them, start from `Duration::ZERO`.
- **The pattern is the same one this whole section runs on.** `PlayerStore`,
  `BufRead`, and now `BlindAlerter` are all dependencies injected behind a
  trait, spied in tests, made real in `main`. When a new requirement touches
  the outside world — storage, input, the clock — the move is always: name the
  capability as a trait, depend on the trait, and test against a double.

Next we take the game online: the same poker logic, driven through a
[WebSocket](websockets.md) so a browser can play — and we'll see the injected
abstractions pay off yet again.
