# Working without mocks, stubs and spies

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/working-without-mocks)**

This book has leaned hard on test doubles — the stubs and spies behind
`StubPlayerStore`, `SpyBlindAlerter`, and friends. They've served us well. But
lean on them too heavily and cracks appear, so this chapter introduces two more
tools that scale better: **fakes** and **contracts**.

## A quick taxonomy

The test doubles we've used so far come in flavours:

- A **stub** returns canned answers. `score()` always returns `Some(20)`.
- A **spy** records how it was called so you can assert on the interaction —
  "`record_win` was called with `Ruth`".
- A **mock** is a spy with expectations baked in, failing if the interaction
  doesn't match.

These are all **interaction-based**: they test that your code *talked to* its
dependency in a particular way. That's often exactly right — the Mocking chapter
needed to prove the countdown *printed* in the correct order, and only a spy can
see that. But interaction-based tests have a failure mode: they couple your test
to *how* the code achieves its goal, so a valid refactor that reaches the same
result by a different route breaks tests that should have stayed green. Assert
on too many interactions and your tests calcify your implementation.

## Fakes: real behaviour, in memory

A **fake** is different. Instead of canned answers or call logs, a fake is a
genuine — if simplified — implementation of the dependency, usually holding
state in memory. Where a stub *pretends* to be a recipe store, a fake *is* one:

```rust,ignore
{{#include ../code/working-without-mocks/v1/src/lib.rs:trait}}
```

```rust,ignore
{{#include ../code/working-without-mocks/v1/src/lib.rs:fake}}
```

`InMemoryRecipeStore` actually stores recipes and gives them back. That changes
how you write tests: instead of spying on interactions, you exercise your code
and then **assert on the final state**, exactly as you'd verify a real system by
querying its database rather than inspecting its request logs.

```rust,ignore
{{#include ../code/working-without-mocks/v1/src/lib.rs:consumer}}
```

```rust,ignore
{{#include ../code/working-without-mocks/v1/src/lib.rs:state_test}}
```

Read that test: plan a menu, then ask the store what it holds. There's no
"was `add` called twice with the right arguments" — there's "the store ends up
with three recipes, two of them Ramen". That's a more natural, more robust
assertion. It doesn't care *how* the planner recorded the recipes, only that the
world ended up correct — so you can refactor the planner freely.

Fakes bring other benefits too: because a fake behaves like the real thing, you
can wire your *whole application* to fakes and run it locally — fast, offline,
no database to spin up — and it genuinely works.

## The catch, and the fix: contracts

Fakes have a cost the other doubles don't. A stub is trivially correct — it
returns what you told it to. A fake *encodes behaviour*, and if that behaviour
drifts from the real implementation, you get the worst outcome in testing: green
tests over broken software. Your fake says adding a recipe then reading it back
works; maybe the real database-backed store has a bug where it doesn't.

The fix is a **contract**: a reusable set of assertions describing what *any*
implementation of the interface must do, which you run against *both* the fake
and the real thing.

```rust,ignore
{{#include ../code/working-without-mocks/v1/src/lib.rs:contract}}
```

This is the [specification pattern from the previous chapter](scaling-acceptance-tests.md)
wearing a different hat: a behaviour expressed against a trait, coupled to no
implementation. Here's a second, deliberately-different store — one that
persists to a JSON document instead of a plain `Vec`, standing in for a
database-backed implementation:

```rust,ignore
{{#include ../code/working-without-mocks/v1/src/lib.rs:json_store}}
```

Its internals are completely different, but the *same contract* holds it to the
same behaviour as the fake:

```rust,ignore
{{#include ../code/working-without-mocks/v1/src/lib.rs:contract_tests}}
```

Now the fake is *validated*. As long as the contract passes for both, you can
trust the fake as a stand-in for the real store — in unit tests, in integration
tests, and in local development. If someone changes the real store and breaks
the behaviour, the contract catches it. If the fake drifts, the contract catches
that too. The assumption that "these two are interchangeable" stops being a hope
in a Slack thread and becomes an executable, always-checked fact.

Rust makes this pattern especially tidy: the contract is a plain function
generic over `impl RecipeStore`, and each implementation gets a one-line test
that hands itself to the contract. No framework, no code generation — just a
trait and a function.

## When to use which

None of this retires stubs and spies — it adds tools for jobs they do badly:

- **Spy** when the *interaction is the behaviour* — the Mocking chapter's
  print-in-order, or "we definitely called the payment API exactly once".
- **Stub** for a simple canned answer when state and behaviour don't matter.
- **Fake** when you want **state-based tests** and realistic local runs — a
  store, a queue, an external API you model as a stateful thing.
- **Contract** whenever you have a fake *and* a real implementation, to prove
  they behave alike and keep them from drifting.

## Wrapping up

- **Interaction-based doubles (mocks, spies) test *how* your code calls its
  dependencies**; over-used, they couple tests to implementation and break on
  valid refactors.
- **Fakes are real, in-memory implementations** that let you write **state-based
  tests** — exercise the code, then assert on the resulting state, the way
  you'd check a real system. They also let you run the whole app locally on
  fakes.
- **Contracts keep fakes honest.** Express the interface's required behaviour
  once as a trait-generic function, and run it against both the fake and the
  real implementation so they can't drift apart. It's the specification pattern
  again — stability where you want it.

The next chapter is a practical one: a checklist of refactoring moves — many of
which we've used throughout the book — for keeping code clean once your tests
give you the safety to change it.
