# Refactoring checklist

Every chapter of this book ends its TDD cycle the same way: red, green, then
**refactor**. That third step is where good design actually happens, and it's
the step people skip when they're rushed or unsure. This chapter is a practical
checklist of small, safe refactoring moves — most of which you've already seen
in action — so that "refactor" becomes a reflex rather than a vague intention.

## What refactoring is (and isn't)

Refactoring has a precise meaning: **changing the internal structure of code
without changing its observable behaviour**. The crucial consequence is that
*your tests should not change*. If you find yourself editing a test while
"refactoring", you're doing something else — changing behaviour, changing a
public signature — and you should recognise that and drive it with a failing
test first.

That constraint is what makes refactoring safe and fast. You are free to:

- rename local variables,
- extract and inline functions,
- introduce private helpers, new types, and new traits,
- rearrange the internals of a public function,

...all with the tests staying green the entire time. The green bar is your
proof that behaviour hasn't drifted. Rust sharpens this: alongside the tests,
the compiler verifies that every rename and extraction is type-correct, so a
large class of refactoring mistakes can't even compile.

## The mental checklist

Run through this every cycle. Each move is small; the discipline is in doing
them constantly, not heroically.

### Extract magic values into named constants

A bare literal is a puzzle. Give it a name and the code explains itself. We did
this throughout: `TEN_SECOND_TIMEOUT` in the [Select chapter](select.md),
`SECOND_HAND_LENGTH` and `CLOCK_CENTRE` in [Maths](maths.md),
`ALL_ROMAN_NUMERALS` in [Property-based tests](property-based-tests.md). The
value didn't change; its *meaning* became legible. In Rust, a `const` at module
scope is free — computed at compile time, zero runtime cost — so there's no
excuse to leave a `90` or a `"---"` floating in a function body.

### Extract a variable to name a sub-expression

When one expression does two things, pull the inner part into a `let` with a
descriptive name. `let tail = numbers.get(1..).unwrap_or(&[]);` reads better
than inlining that slice into a `sum` call. The compiler optimises the binding
away; the reader keeps the name.

### Extract a function to name a concept

If a block of a function is doing a nameable job, extract it. `angle_to_point`,
`score_for_player`, `schedule_blind_alerts`, `read_league` — each pulled a
coherent idea out of a longer function and gave it a name. The test of a good
extraction: the *caller* now reads like a summary. `play_poker` calling
`schedule_blind_alerts` then reading input says what it does at a glance.

### Make public functions easy to scan

A reader skimming a public function wants the *what*, not every *how*. Push the
detail down into private helpers so the top-level function reads like a table of
contents. This is why `respond`, `render`, and `svg_writer` stayed short —
they delegate. Rust's default privacy helps: helpers are private unless you say
`pub`, so extracting a private helper never pollutes your public API.

### Move value creation to construction time

If a struct method recomputes something derivable from its fields on every
call, compute it once in the constructor and store it. The
[IO chapter](io-and-sorting.md)'s file store parses the league *once* in `new`
and caches it, rather than re-reading the file on every `score` call. The public
behaviour is identical; the work moved to where it belongs. In Rust this often
also *simplifies ownership* — a cached owned value is easier to hand out than
one recomputed behind a lock each time.

### Prefer the type system over comments

A comment explaining what a value means is often a type waiting to be born. The
[Errors chapter](errors-and-result.md) turned "this string might be an error"
into a `Result`; the newtype `Bitcoin(u64)` from the
[Structs chapter](structs-methods-and-traits.md) turned "this integer is
money, don't add it to a plain number" into a compile-time guarantee. A good
name and a good type remove the need for most explanatory comments — and unlike
comments, they can't go stale, because the compiler checks them. Keep comments
for the *why* that code genuinely can't express: a constraint, a workaround, a
link to a spec.

### Let clippy do the nagging

Rust hands you a refactoring coach for free. Throughout this book, `clippy`
pushed us toward the idiomatic move — `sort_by_key(|p| Reverse(...))` over a
manual comparator, `.flatten()` over `if let Ok(...)`, `sort_by_key` over
`sort_by`. Running `cargo clippy` after a green test is a cheap prompt for "is
there a cleaner way to say this?" — and because our CI runs it with
`-D warnings`, the cleaner way is the only way that ships.

## The two tools that make it possible

None of this works without a tight feedback loop, and you already have both
halves:

- **Run the tests after every small change.** The whole point of investing in
  fast unit tests is this: a few milliseconds tells you whether your last edit
  kept behaviour intact. Use that loop constantly, not once at the end.
- **Lean on source control.** Try an idea; if it's better, commit; if not,
  revert without ceremony. Refactoring should feel low-stakes and reversible.
  The reason it feels risky to people who complain they "don't have time to
  refactor" is that they don't do it in small steps with a safety net — so it
  becomes a big, scary, all-at-once change. Small steps under green tests are
  the opposite of scary.

## Don't ask permission

Refactoring isn't a separate ticket or a special event you schedule. It's part
of writing the code — the "refactor" in every red-green-refactor cycle. Leave
each piece of code a little cleaner than you found it, continuously, and the
codebase stays healthy without ever needing a dramatic "refactoring sprint".
The skill grows with practice, and the practice is cheap precisely because your
tests and the compiler catch you when you slip.

## Wrapping up

- **Refactoring changes structure, not behaviour** — so your tests stay
  unchanged and green throughout. If a test has to change, you're changing
  behaviour; drive that with a failing test instead.
- **Keep a mental checklist**: name magic values as `const`s, extract variables
  and functions to name concepts, keep public functions scannable, move
  derivable work to construction, and replace explanatory comments with better
  names and types.
- **In Rust the compiler and clippy are refactoring partners** — renames and
  extractions must type-check, privacy keeps helpers out of your API, and
  clippy suggests the idiomatic form.
- **Fast tests plus source control make it safe and cheap.** Change in small
  steps, run the tests, commit or revert. Do it constantly, don't ask
  permission, and your design skills compound.
