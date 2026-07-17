# HashMaps

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/hashmaps)**

In [arrays and slices](arrays-and-slices.md), you saw how to store values in
order. Now we'll look at a way to store items by a *key* and look them up
quickly.

Maps allow you to store items like a dictionary: the key is the word, the value
is the definition. And what better way to learn about maps than to build our own
dictionary?

Rust's map is `HashMap<K, V>`, from `std::collections` — the one part of the
standard library that isn't in scope automatically, so files that use it start
with:

```rust,ignore
use std::collections::HashMap;
```

One note before we start, the equivalent of the Go book's aside that map keys
must be "comparable": a `HashMap` key must implement the `Hash` and `Eq` traits
— hashable, and equality-comparable, which is what a hash table needs to do its
job. You know from the [ownership chapter](ownership-and-borrowing.md) what that
means in practice: `String` and the number types qualify, and your own types can
opt in with `#[derive(Hash, PartialEq, Eq)]`. The value type can be anything,
even another map.

## Write the test first

The first requirement: given a dictionary that already contains a word, `search`
returns its definition. And after the [last chapter](option-and-pattern-matching.md),
we can write a more honest test than "search returns a string" — because what
should searching for a *missing* word return? We know the type for that answer
now. Both cases, up front:

```rust,ignore
{{#include ../code/hashmaps/v1/src/lib.rs:test}}
```

This is worth a pause, because it's a whole section of the Go original
evaporating. The Go chapter starts with `Search` returning a bare string, then
discovers that a missing word silently returns `""`, then upgrades to the
`(value, ok)` comma-ok idiom, then wraps it in an error. That whole arc exists
because Go map lookups *default to pretending the key exists*, returning a zero
value. Rust's `HashMap::get` returns `Option` — the "might not be there" is in
the signature on day one, and our test simply writes down both possibilities.

## Try to run the test

```text
error[E0433]: failed to resolve: use of undeclared type `Dictionary`
```

## Write the minimal amount of code for the test to run and check the output

Like the Go book, we don't want to pass raw maps around — we want a `Dictionary`
type of our own. Go wraps with `type Dictionary map[string]string`; our tool is
the newtype pattern from the [ownership chapter](ownership-and-borrowing.md):

```rust,ignore
#[derive(Default)]
pub struct Dictionary(HashMap<String, String>);

impl Dictionary {
    pub fn search(&self, word: &str) -> Option<&str> {
        None
    }
}
```

(The Go version starts with a free function `Search(dictionary, word)` and
refactors to a method later. Rust nudges us to the method immediately — try the
free-function form and you get:

```text
error[E0106]: missing lifetime specifier
 --> src/lib.rs:3:75
  |
3 | pub fn search(dictionary: &HashMap<String, String>, word: &str) -> Option<&str> {
  |                           ------------------------        ----            ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature
          does not say whether it is borrowed from `dictionary` or `word`
```

Read the help text — it's a fair question! We return borrowed text, and there
are two things it could be borrowed from. The [Lifetimes](lifetimes.md) chapter teaches the
annotation that answers it; a *method* sidesteps the question, because "borrowed
from `self`" is the obvious answer and Rust assumes it. Method it is.)

The stub fails the right way:

```text
assertion `left == right` failed
  left: None
 right: Some("this is just a test")
```

## Write enough code to make it pass

```rust,ignore
{{#include ../code/hashmaps/v1/src/lib.rs:code}}
```

Both tests pass — the unknown-word case came free, which tells you the type was
right. Two details in that one line:

- `self.0.get(word)` returns `Option<&String>` — a borrowed view into the map,
  no copying. Our signature promises `Option<&str>`, so we `.map` the borrowed
  `String` down to a borrowed `str` — the `map` combinator from last chapter,
  three pages old and already earning its keep.
- We stored `String` keys but searched with a `&str` — `HashMap` accepts a
  borrowed form of the key for lookups, so callers don't have to allocate to
  ask a question.

## Add

We have a great way to search, but no way to add new words.

The Go book's `Add` arc has a twist worth spoiling: its first version silently
*overwrites* existing words, and the chapter then patches it with an
`ErrWordExists`. We've been burned by "compiles but lies" before, so let's write
both tests up front — adding a new word works; adding an existing word refuses
and *leaves the original alone*:

```rust,ignore
{{#include ../code/hashmaps/v2/src/lib.rs:test}}
```

Failure needs a type. Last chapter's lesson: when there's more than one way to
fail — and `WordDoesNotExist` is visibly coming once we write `update` — that's
an enum growing a variant per failure mode:

```rust,ignore
{{#include ../code/hashmaps/v2/src/lib.rs:error}}
```

Stub `add` with an unconditional insert and `Ok(())`, and the second test
catches the overwrite bug the Go version shipped:

```text
assertion `left == right` failed
  left: Ok(())
 right: Err(WordExists)
```

Make it pass:

```rust,ignore
{{#include ../code/hashmaps/v2/src/lib.rs:code}}
```

Note `add` takes `&mut self` while `search` takes `&self` — by now that's not
syntax, it's documentation. Which brings up the Go chapter's strangest section,
the one titled "Pointers, copies, et al": it has to explain that Go maps mutate
through *value* receivers because a map is secretly "a pointer to a
runtime.hmap structure", and then warn that a `nil` map panics on write, so
never declare one uninitialised. None of that survives translation. A
`HashMap` is a plain value; mutating it requires `&mut` like everything else;
and there is no nil map to step on — `Dictionary::default()` gives you an
empty, writable one. The whole section's job is done by the receiver.

## Refactor

The implementation works but does its work twice: `contains_key` hashes and
finds the slot, then `insert` hashes and finds it again. `HashMap` has an API
for "find the slot once, then decide" — `entry`:

```rust,ignore
{{#include ../code/hashmaps/v3/src/lib.rs:add}}
```

And look at what `entry` returns: an **enum**, `Occupied` or `Vacant`, matched
exhaustively. The standard library keeps handing us the same shapes this book
has been teaching — the "is it there or not?" question, answered by a type you
can't half-read. (`use std::collections::hash_map::Entry;` brings the variant
names in.)

## Update

Next, updating the definition of an existing word — and its mirror-image rule:
updating a word that *doesn't* exist is an error, not a sneaky insert. The Go
book argues for a distinct error here, and the argument is good — a web app
might redirect on not-found but show a message on can't-update — so the enum
grows its second variant:

```rust,ignore
#[derive(Debug, PartialEq, Eq)]
pub enum DictionaryError {
    WordExists,
    WordDoesNotExist,
}
```

Tests:

```rust,ignore
{{#include ../code/hashmaps/v3/src/lib.rs:update_test}}
```

Implementation — one new tool:

```rust,ignore
{{#include ../code/hashmaps/v3/src/lib.rs:update}}
```

`get_mut` is `get`'s writable sibling: `Option<&mut String>`, a *mutable* borrow
into the map's own storage. `*existing = ...` writes straight through it —
replacing the value in place, no remove-and-reinsert. The borrow rules from the
[ownership chapter](ownership-and-borrowing.md) are what make this safe to
offer: while `existing` lives, the compiler guarantees nobody else is reading
the map.

## Delete

You know the moves by now. Tests first — deleting a word removes it; deleting an
unknown word is `WordDoesNotExist`:

```rust,ignore
{{#include ../code/hashmaps/v4/src/lib.rs:delete_test}}
```

Where Go's `delete(map, key)` returns nothing — so the Go version has to
`Search` first, then delete — `HashMap::remove` returns `Option<V>`: the value
that was there, if it was. One call asks and acts:

```rust,ignore
{{#include ../code/hashmaps/v4/src/lib.rs:delete}}
```

`Some(_)` — it existed, don't care what it said, `Ok`. `None` — nothing to
delete, error. The test suite is green: a full CRUD API.

## Refactor: last coat of paint

`DictionaryError` crosses our API boundary, so it gets the
[errors chapter](errors-and-result.md) treatment — `Display` with a message per
variant (one `match`, exhaustive, so adding a failure mode someday *forces* a
human-readable message for it), and the `Error` marker:

```rust,ignore
{{#include ../code/hashmaps/v4/src/lib.rs:error}}
```

## Wrapping up

We made a full CRUD (Create, Read, Update, Delete) API for our dictionary, and
learned:

- **`HashMap<K, V>`**: creation (`from`, `default`), `get`, `insert`,
  `contains_key`, `get_mut`, `remove` — and that keys are anything `Hash + Eq`
- Lookups return **`Option<&V>`** — borrowed views into the map, with absence in
  the type; removal returns the evicted value the same way
- The **`entry` API** for check-then-act in a single lookup, driven by matching
  on an enum
- **`get_mut`** and writing through a mutable borrow with `*`
- One **error enum** with a variant per failure mode, `Display`ed exhaustively —
  where the Go chapter needed three sentinel values and a custom string type
- `&self` vs `&mut self` doing the documentation work that Go's chapter needed
  a blog-post digression about map internals to do

A pattern to notice across these last three chapters: Go's runtime conventions —
comma-ok, nil maps, sentinel error values — keep turning into Rust types. The
convention becomes an enum; checking it becomes a `match`; forgetting it becomes
a compile error. That's not three coincidences. It's the design philosophy, and
you now know enough of it to be dangerous.
