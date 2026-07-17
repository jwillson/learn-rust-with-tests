# A bytecode virtual machine

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/interpreter)**

Our tree-walking interpreter works, but it does redundant work: every time it
evaluates a program it re-matches every AST node, chasing `Box` pointers all
over the heap. Real language implementations — CPython, the JVM, Lua, V8 —
don't walk the tree. They **compile** it once into a flat list of simple
instructions, *bytecode*, and run that on a **virtual machine**.

This chapter builds exactly that, and it's the payoff of the whole arc: we'll
compile the same AST into bytecode, execute it on a stack machine, and — the
satisfying part — verify it against the *identical specification* the
interpreter passed. Two completely different engines, proven to compute the same
answers by one shared test suite.

## The instruction set

A stack machine is beautifully simple. It has a stack of values and a list of
instructions; each instruction pushes to, or pops from, the stack. Our
instruction set is a small enum:

```rust,ignore
{{#include ../code/interpreter/v4/src/lib.rs:opcode}}
```

`Push(n)` puts a number on the stack. `Add` pops two values and pushes their
sum. `Negate` pops one and pushes its negation. That's the whole machine.
Notice this is *another* enum-and-`match` design — the AST modelled *structure*,
the bytecode models *operations*, and both are natural Rust enums.

## Compiling the tree to bytecode

Compilation is a recursion over the AST, just like evaluation was — but instead
of *computing* a value at each node, we *emit instructions* that will compute it
later:

```rust,ignore
{{#include ../code/interpreter/v4/src/lib.rs:compile}}
```

The key insight is the *order*. For a binary operation, we emit the code for the
left operand, then the right, then the operator. This is **post-order**
traversal, and it's exactly what a stack machine wants: by the time the `Add`
instruction runs, its two operands are already sitting on the stack, pushed by
the code that ran before it. A test makes this concrete:

```rust,ignore
{{#include ../code/interpreter/v4/src/lib.rs:bytecode_test}}
```

`1 + 2 * 3` compiles to `push 1, push 2, push 3, multiply, add`. Read it as a
stack program: push the three numbers, `multiply` pops 2 and 3 and pushes 6,
`add` pops 1 and 6 and pushes 7. The tree's structure became a linear sequence,
and the precedence — still — is preserved automatically, because it was baked
into the tree the compiler walks.

## The virtual machine

The VM is a loop over the instructions with a `Vec<i64>` as its stack:

```rust,ignore
{{#include ../code/interpreter/v4/src/lib.rs:vm}}
```

Each instruction is a `match` arm. `Push` pushes; `Negate` pops one and pushes
the negation; the binary operators pop two (**right first** — it was pushed
last — then left) and push the result. The `binary` helper factors out that
pop-pop-compute-push dance, taking a closure for the actual operation — the
[closures from the Iterators chapter](iterators-and-closures.md), and it's where
division checks for zero and returns our `RuntimeError`. When the program
finishes, the answer is the one value left on the stack.

One honest note about those `.expect("stack underflow")` calls. A hand-written
bytecode program could underflow the stack, but *ours can't*: our compiler only
ever emits balanced code — every operator has its operands pushed first. The
compiler and VM form a closed system with an invariant the compiler guarantees,
so the pops are safe. In a VM that loaded bytecode from *untrusted* sources
you'd validate instead of `expect`, but for a compiler-plus-VM pair, an
`expect` documenting the invariant is the honest choice.

`run` now has an extra stage — lex, parse, **compile**, execute — and thanks to
the `From`-based `?` conversion from last chapter, adding it was a one-line
change:

```rust,ignore
{{#include ../code/interpreter/v4/src/lib.rs:run}}
```

## The payoff: one specification, two engines

Here's why we made `PROGRAMS` public two chapters ago. This crate depends on the
interpreter crate *only in its tests*, imports that exact list of programs, and
runs every one of them through the **bytecode VM**:

```rust,ignore
{{#include ../code/interpreter/v4/src/lib.rs:shared_test}}
```

The VM passes the interpreter's entire specification, division-by-zero and all.
This is the [specification/driver pattern](scaling-acceptance-tests.md) in its
most convincing form: one set of "this program means this value" facts, verified
against two implementations that share *nothing* but the front-end parser — a
recursive tree-walker and a compile-to-bytecode stack machine. If they ever
disagreed, a test would fail instantly, pointing at the exact program. That's
how you refactor a language's execution engine — or swap in a faster one —
without fear: the specification is the contract, and both engines must honour
it.

## Wrapping up

- **A bytecode VM compiles the AST once, then executes flat instructions** — the
  architecture real language runtimes use, and it's still just enums and
  `match`: one enum for instructions, a `Vec` for the stack, a loop for the
  machine.
- **Compilation is post-order traversal**: emit operands before operators, and
  the stack machine finds its arguments already in place. Precedence, as ever,
  rides along in the tree.
- **A compiler and its VM form a closed system with invariants** — our compiler
  only emits balanced bytecode, so the VM's stack operations are provably safe.
- **The shared specification proves the two engines agree.** Making the
  behaviour a public, reusable list of cases meant a wholly different execution
  strategy could be verified against the original with a single imported test —
  the strongest possible statement that "it still means the same thing."

You've now built a real, if tiny, language: text in, tokens, a tree, an
interpreter, *and* a bytecode compiler and VM — every stage test-driven, and the
two backends held to one specification. The remaining two project chapters
switch gears entirely: an LRU cache to wrestle with ownership, and a parallel
data processor to cash in Rust's fearless concurrency with a real speedup.
