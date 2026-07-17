# Why unit tests and how to make them work for you

You've written a lot of tests over the course of this book. This chapter steps
back to ask *why* — because understanding the reasons is what turns testing from
a chore you tolerate into a tool you reach for.

## Software is soft

The word is a giveaway. Software is meant to be *soft* — changeable — as opposed
to hardware. A program isn't a thing you build once and ship; it's a living
artifact that will be changed, extended, and fixed for as long as anyone uses
it. The value of a codebase isn't the code sitting there today, it's your
ability to keep changing it cheaply tomorrow.

Two observations about long-lived software, first made by Manny Lehman decades
ago, still hold:

- **Software under use must continually change**, or it becomes progressively
  less useful. Requirements shift, dependencies move, the world around your
  program doesn't sit still.
- **As software changes, its complexity increases** unless active work is done
  to reduce it. Left alone, every codebase drifts toward tangle.

Put together, these are a warning: a program you can't confidently change is a
program that is dying. And the thing that lets you change code confidently — the
thing that fights the rising tide of complexity — is refactoring under tests.

## Tests exist to enable change

This is the core idea, and it reframes everything. Unit tests are not primarily
about *catching bugs*, though they do. They exist to make your software
**changeable**. A good test suite is a machine that answers one question,
instantly and repeatedly: *did I just break anything?* When the answer is "no,
still green" after every small edit, you can refactor fearlessly — rename,
extract, restructure, improve — knowing the moment you break behaviour, a test
will tell you exactly where.

You felt this throughout the book. Every "Refactor" step rested on green tests:
`sum_all_tails` became an iterator pipeline, the file store started caching in
memory, handlers got broken into helpers — and each time, the tests staying
green *were the proof* that behaviour hadn't drifted. Without them, every one of
those improvements would have been a gamble. With them, they were routine.

And in Rust you have a second safety net working alongside the tests: the
compiler. Renames must type-check, extracted functions must satisfy the borrow
checker, a changed trait forces every implementer to update. The compiler
catches the structural mistakes; the tests catch the behavioural ones. Together
they make refactoring in Rust unusually safe — which is exactly why this book
leaned on the red-green-**refactor** cycle so hard.

## The discipline: don't change behaviour while refactoring

Refactoring has a strict definition worth repeating: **changing structure
without changing behaviour**. The discipline that makes it safe is a rule you've
seen in every chapter — *when you refactor, your tests do not change*. If you
find yourself editing a test mid-refactor, stop: you're changing behaviour, and
that's a different activity, one that should start with a failing test.

Keeping the two activities separate is what keeps the safety net intact. If you
change code *and* tests at the same time, neither can vouch for the other. Move
in small steps, run the tests constantly, and lean on source control so any
experiment is one `git revert` away from undone. This is why the book committed
in batches and ran `cargo test --workspace` after every change: the tight loop
is the whole point.

## Why do people resist writing tests, then?

If tests are this valuable, why the widespread reluctance? Usually it traces to
tests done *badly* — and bad tests are genuinely worse than no tests, because
they impose cost without the benefit:

- **Tests coupled to implementation detail.** If your test asserts on *how* the
  code works rather than *what* it does, then every valid refactor breaks it.
  Now the safety net has become a straitjacket — the tests fight the very change
  they should enable. The [Working without mocks](working-without-mocks.md)
  chapter's warning about over-using spies is exactly this: assert on outcomes
  and observable behaviour, not on internal call sequences, and your tests
  survive refactoring.
- **Slow tests.** The value of a unit test is the *milliseconds* it takes to
  answer "did I break anything?" A suite that takes minutes can't be run after
  every small edit, so it stops being a feedback loop and becomes a gate you
  run once and dread. Keep the base of the pyramid fast; push the slow,
  end-to-end checks to the smaller [acceptance-test](intro-to-acceptance-tests.md)
  layer.
- **Tests written after the fact, to hit a coverage number.** These tend to
  assert whatever the code already does, bugs included, and teach you nothing
  about the design. Test-*first* is different: writing the test before the code
  forces you to use your own interface as a consumer would, which is the fastest
  way to notice a design is awkward — a lesson that recurred every time a test
  was painful to write in this book, and the pain pointed at a coupling problem
  rather than a testing one.

The fix for "tests are a burden" is almost never "write fewer tests" — it's
"write better tests": fast, behaviour-focused, and driven out first.

## Wrapping up

- **Software's value is in its changeability**, and long-lived software *must*
  change while resisting a natural drift toward complexity.
- **Unit tests exist to enable change.** Their headline benefit isn't catching
  bugs — it's letting you refactor fearlessly, because green tests prove
  behaviour is intact. In Rust, the compiler is a second net catching the
  structural mistakes.
- **Refactor without changing behaviour**, which means without changing tests;
  keep the two activities separate so each can vouch for the other.
- **Resistance to testing usually means bad tests.** Coupled, slow, or
  after-the-fact tests impose cost without benefit. Fast, behaviour-focused,
  test-first unit tests are the ones that pay for themselves many times over —
  and they're the ones this whole book set out to teach you to write.

One chapter remains: a catalogue of TDD **anti-patterns** — the specific
mistakes that produce those brittle, burdensome tests, and how to avoid them.
