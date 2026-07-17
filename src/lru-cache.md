# An LRU cache

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/lru)**

An LRU (least-recently-used) cache holds a fixed number of entries. You `get`
and `put` by key, and when the cache is full and you insert something new, it
evicts the entry that hasn't been touched for the longest — *both* `get` and
`put` count as "touching". It's a classic interview question and a real
workhorse (CPU caches, page caches, HTTP caches all use the idea).

It's also the single best project for meeting the Rust borrow checker head-on,
because the textbook efficient implementation is a **doubly-linked list**, and a
doubly-linked list is precisely the shape Rust makes you rethink. We'll build it
twice: a simple version to nail the behaviour, then the idiomatic efficient
version, learning *why* Rust pushes you toward it.

## First, the behaviour

Before worrying about efficiency, let's pin down what the cache *does* with a
dead-simple implementation: a `Vec` of entries kept in recency order, oldest at
the front.

```rust,ignore
{{#include ../code/lru/v1/src/lib.rs:code}}
```

`get` finds the key, removes it, and pushes it to the back (now most-recent).
`put` removes any old copy or evicts the front when full, then pushes to the
back. It's `O(n)` per operation because of the linear search and the `remove`
shuffle — but it's obviously correct, and the tests capture every rule of an LRU
cache:

```rust,ignore
{{#include ../code/lru/v1/src/lib.rs:test}}
```

The recency tests are the interesting ones: getting `"a"` must *refresh* it so a
later insert evicts `"b"` instead, and updating an existing key must refresh it
too. With green tests locking in the behaviour, we're free to make it fast.

## The efficient design, and the wall

The `O(n)` cost comes from two places: finding a key (linear scan) and moving an
entry (shifting the `Vec`). The classic fix pairs a **hash map** (key → node,
for `O(1)` lookup) with a **doubly-linked list** (for `O(1)` move-to-front and
eviction). Every language textbook draws it with nodes pointing at each other:

```rust,ignore
struct Node {
    prev: &mut Node,  // or Box, or a pointer...
    next: &mut Node,
}
```

And in Rust, this is where you hit the wall. A doubly-linked list is
*aliased and mutable* — the same node is pointed at by its neighbour on both
sides, and you need to mutate through those pointers. That's exactly what Rust's
"one mutable borrow at a time" rule forbids. Even if you store nodes in a `Vec`
and link them by index, the moment you try to reach two nodes at once to relink
them:

```rust,ignore
fn unlink(&mut self, i: usize) {
    let node = &mut self.nodes[i];
    let prev = &mut self.nodes[node.prev];  // second &mut into the same Vec
    prev.next = node.next;
}
```

```text
error[E0499]: cannot borrow `self.nodes` as mutable more than once at a time
  --> src/lib.rs:15:25
   |
14 |         let node = &mut self.nodes[i];
   |                         ---------- first mutable borrow occurs here
15 |         let prev = &mut self.nodes[node.prev];
   |                         ^^^^^^^^^^ second mutable borrow occurs here
```

In C you'd relink with raw pointers and hope you got it right; in Rust that whole
class of use-after-free and aliasing bug is a compile error. But it means the
naive translation simply doesn't build. You have two ways forward:

- **`Rc<RefCell<Node>>`** — reference-counted shared ownership with runtime
  borrow checking (the [Mocking chapter](mocking.md)'s `RefCell`). This
  *compiles*, but a doubly-linked list creates reference cycles that leak
  memory, and every access pays a runtime borrow check that can *panic*. It's
  the tool for genuine shared ownership, but it's the wrong fit here.
- **An index arena** — the idiomatic Rust answer, and the one we'll use.

## The index arena

The insight is to stop using *references* as links and use **indices** instead.
Keep all the nodes in one `Vec` (the "arena"), and let each node store the
`usize` index of its neighbours rather than a pointer:

```rust,ignore
{{#include ../code/lru/v2/src/lib.rs:node}}
```

An index is a plain `usize` — `Copy`, owns nothing, borrows nothing. So the
borrow checker has no objection to a node "pointing at" another via an index,
and the cycle that made `Rc<RefCell>` leak is just two integers. The `E0499`
above disappears the moment we relink by *copying indices out first* and
touching one slot at a time:

```rust,ignore
{{#include ../code/lru/v2/src/lib.rs:links}}
```

`unlink` reads the node's `prev`/`next` indices into local `Copy` values (ending
the borrow), then mends each neighbour in a *separate* statement — never two
`&mut` into the arena at once. `push_front` and `move_to_front` follow the same
discipline. This is the technique the compiler was nudging us toward: with
indices, the arena stays a plain `Vec` you mutate one element at a time, and all
the pointer-juggling correctness the borrow checker was worried about is now
trivially safe.

`get` and `put` sit on top of these list operations plus the hash map:

```rust,ignore
{{#include ../code/lru/v2/src/lib.rs:get}}
```

```rust,ignore
{{#include ../code/lru/v2/src/lib.rs:put}}
```

`get` is a hash lookup, a move-to-front, and a borrow of the value — all `O(1)`.
`put` updates-and-refreshes an existing key, or for a new key either grows into
a fresh slot or, when full, **reuses the evicted least-recently-used slot** in
place. That slot reuse is a small bonus of the arena: because the capacity is
fixed, we never allocate a node beyond `capacity`, we just recycle the tail's
slot for the newcomer.

The behaviour tests are identical to the naive version — same rules, same
assertions — plus a longer sequence to exercise real churn, and they all pass.
Two very different implementations, one specification: the same lesson the
[interpreter arc](bytecode-virtual-machine.md) just taught, now for a data
structure.

## Wrapping up

- **The borrow checker makes pointer-linked structures rethink themselves.** A
  doubly-linked list is aliased-and-mutable, exactly what Rust forbids, so the
  naive `&mut`-linked translation won't compile (`E0499`).
- **`Rc<RefCell<T>>` is the shared-ownership escape hatch**, but for a linked
  list it leaks (cycles) and pays runtime borrow-check costs — usually the wrong
  tool.
- **The index arena is the idiomatic answer**: store nodes in a `Vec`, link them
  by `usize` index, and relink by copying indices out and mutating one slot at a
  time. Indices own and borrow nothing, so the aliasing that troubled the
  compiler evaporates — and you get `O(1)` operations with cache-friendly,
  leak-free storage.
- **Build the simple correct version first.** The naive `Vec` cache locked in
  the behaviour with tests, so the efficient rewrite was a refactor under a green
  bar, not a leap of faith.

The arena/handle pattern you learned here isn't just for caches — it's how
idiomatic Rust builds graphs, trees with parent pointers, entity systems in
games, and any structure where things point at each other. When references fight
you, reach for indices.

One project left: we take a slow, sequential computation and make it *fast* with
threads — the fearless-concurrency payoff, measured with real benchmarks.
