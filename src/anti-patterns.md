# Anti-patterns

The [previous chapter](why-unit-tests.md) argued that resistance to testing
almost always comes from tests done *badly*. This final chapter names the
specific ways tests go bad — the anti-patterns — so you can recognise and avoid
them. Most of these are the flip side of habits the book has been building all
along.

## Not doing TDD at all

The most common one. Writing code first and tests afterward means your tests
document *whatever the code happens to do*, bugs included, rather than what it
*should* do. You lose the design feedback that comes from being your interface's
first consumer, and you tend to write only the tests that are easy to write
rather than the ones that matter. Test-first isn't a moral position; it's the
mechanism that makes the other benefits appear.

## Misunderstanding the refactor step

Refactoring means changing structure *without changing behaviour* — so **your
tests must not change while you refactor**. If you're editing tests and
production code in the same breath, you've lost your safety net: neither can
vouch for the other. When a "refactor" genuinely requires a test change, that's
your signal you're changing behaviour, and behaviour changes start with a
failing test. Keep the two activities strictly separate.

## Evergreen tests

A test that cannot fail is worse than no test, because it radiates false
confidence. The classic cause is never watching your test go red. This is why
every chapter in this book ran the test at the *failing* state and looked at the
output before making it pass — the red step isn't ceremony, it's how you verify
the test actually tests something. If you skip straight to green, you may have
written a test that would pass no matter what the code did.

## Useless assertions

`assert!(true)`, or asserting a value equals itself, or checking that a function
returned *something* without checking *what*. These pad a coverage number while
proving nothing. Every assertion should be capable of failing if the behaviour
you care about is wrong — if you can't imagine a code change that would make an
assertion fail, delete it.

## Asserting on irrelevant detail

This is the big one, the root of most brittle suites. When a test asserts on
things it doesn't actually care about, it breaks for reasons unrelated to its
purpose. The [Error types](error-types.md) chapter's whole lesson was here:
asserting on an exact error *string* couples the test to wording nobody cares
about, so a harmless message tweak fails the test. Assert on the error's *type
and data* instead. Likewise, the [JSON chapter](json-routing-embedding.md)
parsed responses into typed values rather than string-matching JSON, because the
data is what matters, not its byte-for-byte encoding.

The rule: **assert on what the behaviour promises, and nothing more.** Every
incidental detail you pin down is a future false failure.

## Too many assertions crammed into one scenario

A single test that checks a dozen unrelated things fails opaquely — you see one
red test and have to dig to find which of the twelve things broke. Prefer
several focused tests, each with a clear name stating the one behaviour it
checks. Throughout this book the tests were split into named `#[test]` functions
(`returns_404_on_missing_players`, `a_duplicate_user_is_a_409`) precisely so a
failure names itself. Rust's per-function test model makes this cheap; use it.

## Not listening to your tests

A painful test is *information*. When a test needs elaborate setup, a tangle of
test doubles, or knowledge of internals to write, the test is telling you the
*design* is off — not that testing is hard. This is the most valuable feedback
TDD gives, and ignoring it wastes the whole exercise.

Concretely, some things tests complain about:

- **Excessive setup and too many doubles.** If exercising one behaviour requires
  stubbing five collaborators, the unit under test is doing too much or depends
  on too much. Consolidate dependencies, or split responsibilities. Every time a
  test in this book got simple, it was because a concern had been separated out
  behind a trait.
- **Over-mocking.** Leaning on spies to assert exact interaction sequences
  couples tests to implementation, so valid refactors break them. Reach for
  [fakes and state-based assertions](working-without-mocks.md) where you can,
  and save interaction-based spying for when the interaction genuinely *is* the
  behaviour.
- **Leaky or polluted interfaces.** If a trait exposes more than a consumer
  needs — or a struct makes internals public just so a test can poke them —
  every consumer (and every test) couples to detail it shouldn't. Rust's default
  privacy helps here: keep fields and helpers private, and define traits with
  the *smallest* surface a consumer needs (`impl Read`, not "the whole file
  system").

## Violating encapsulation to test

Making a function or field `pub` *solely* so a test can reach it is a smell.
It's the test asking to check internals, which is the "asserting on irrelevant
detail" anti-pattern wearing a disguise — now the internal is part of your
public API, and callers can couple to it. Test through the public surface, the
way a real consumer would. (Rust's `#[cfg(test)] mod tests` lives *inside* the
module, so it can already reach private items for genuine white-box unit tests
without exposing them to the outside world — use that rather than widening
visibility.)

## Over-complicated table tests

Table-driven tests are excellent — this book used them for Roman numerals, clock
angles, and blind schedules — but they turn bad when the table grows conditional
logic: rows that mean different things, `if` statements inside the loop, optional
fields that only apply to some cases. At that point the test has its own
complexity worth testing, which is absurd. Keep each table homogeneous: every
row the same shape, exercising the same behaviour with different data. If some
cases need different handling, they belong in their own test.

## Summary

Almost every anti-pattern here reduces to one of two failures:

1. **Testing implementation instead of behaviour** — asserting on strings,
   internals, exact interactions, and incidental detail. These tests break when
   you refactor, turning your safety net into a straitjacket. The cure is to
   assert on *what the code promises*, through its public surface.
2. **Not listening to the feedback tests give you** — pushing through painful
   setup and heavy mocking instead of treating that pain as a design signal. The
   cure is to let a hard-to-test unit push you toward better separation of
   concerns.

Get these right and tests become what they're meant to be: the thing that lets
you change your software forever, without fear.

## The end (and the beginning)

That's the book. You've gone from `assert_eq!` in a Hello, World test to a
persistent, concurrent, HTTP-and-WebSocket poker application, an SVG clock, a
property-tested Roman numeral converter, and a blog engine — every line of it
driven out with tests, and every external dependency injected behind a trait so
the logic stayed testable in isolation.

More importantly, you've built the *habits*: red, green, refactor; listen to
your tests; separate concerns; depend on abstractions; let the compiler and the
tests carry the risk so you can change code without fear. Those habits transfer
to whatever you build next. The Rust-specific machinery — ownership, traits,
`Result` and `Option`, the borrow checker as an always-on correctness assistant
— turned out not to be obstacles to TDD but amplifiers of it: a great many bugs
this book would otherwise have caught with a test, Rust caught at compile time
instead, leaving your tests free to verify the behaviour that really needs
verifying.

Go build something, test-first. And keep it green.
