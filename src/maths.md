# Maths

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/maths)**

For all the power of modern computers to perform huge sums at lightning
speed, the average developer rarely uses any mathematics to do their job.
But not today! Today we'll use trigonometry and vectors to solve a *real*
problem: drawing an SVG of an analogue clock — hands pointing in the right
directions for a given time.

SVGs are a great format to generate programmatically because they're just
shapes described in XML. A clock is a circle (the bezel) and three lines,
each starting at the centre `(150, 150)` and ending wherever the maths says
the tip of that hand should be:

```xml
<circle cx="150" cy="150" r="100" style="fill:#fff;stroke:#000;stroke-width:5px;"/>
<line x1="150" y1="150" x2="150" y2="60" style="fill:none;stroke:#f00;stroke-width:3px;"/>
```

That's the second hand at midnight: straight up, 90 units long. One SVG
gotcha to hold onto: the origin `(0,0)` is the **top left** corner, so "up"
means *smaller* y.

Two housekeeping notes before the fun starts. Go's chapter takes a `time.Time`;
Rust's standard library deliberately has no calendar/clock-of-day type (that's
the [chrono](https://docs.rs/chrono) crate's job in real applications), and
a clockface needs only three numbers, so we'll say what we mean with a struct:

```rust,ignore
{{#include ../code/maths/v1/src/lib.rs:types}}
```

## An acceptance test

This chapter also introduces an idea the whole final act of this book is
built on. Ask yourself: *what does finished look like?* TDD answers "when
the test passes" — but so far our tests have been about functions. An
**acceptance test** is a test written at the level of the *feature*: it
describes the whole outcome (here: "the SVG contains a second hand pointing
the right way for this time"), and then unit-tested pieces are built up
underneath until it passes. High-level test telling you *whether* you're
done; low-level tests telling you *what to do next* — two nested feedback
loops.

We'll write that acceptance test soon — testing actual SVG output. But the
honest first step is smaller, because before we can draw anything we need to
answer: *what angle is the second hand at?* Park the acceptance test;
descend a level.

## Write the test first

Angles for computers mean **radians**: a full turn is 2π rather than 360°,
and Rust's trig methods (`f64::sin`, `f64::cos`) expect them. 30 seconds is
half a turn — π radians:

```rust,ignore
#[test]
fn converts_seconds_to_an_angle_in_radians() {
    let thirty_seconds = simple_time(0, 0, 30);

    assert_eq!(seconds_in_radians(&thirty_seconds), PI);
}
```

`PI` is `std::f64::consts::PI`, and `simple_time` is a one-line test helper
constructing a `Time`. The familiar rhythm: undefined function, stub
returning `0.0`, failure `left: 0.0, right: 3.141592653589793`, return `PI`,
green, extend the table:

```rust,ignore
let cases = [
    (simple_time(0, 0, 0), 0.0),
    (simple_time(0, 0, 30), PI),
    (simple_time(0, 0, 45), (PI / 2.0) * 3.0),
    (simple_time(0, 0, 7), (PI / 30.0) * 7.0),
];
```

One second is 2π/60 = π/30 radians, so the implementation is a division:

```rust,ignore
{{#include ../code/maths/v1/src/lib.rs:radians}}
```

(Written as `π / (30/s)` rather than `s * (π/30)` — mathematically
identical; remember that.) Run the tests:

```text
thread 'tests::converts_seconds_to_an_angle_in_radians' panicked:
assertion `left == right` failed: at 7 seconds
  left: 0.7330382858376184
 right: 0.7330382858376183
```

Wait, what?

### Floats are horrible

Look closely: the two numbers differ in the **sixteenth decimal place**. Our
implementation computes `π / (30/7)`; our test expectation computes
`(π/30) * 7`; and although algebra says they're the same number, floating
point arithmetic rounds after every operation, and the two routes round
differently by exactly one bit. Floating point is [notoriously like
this](https://0.30000000000000004.com/) — computers really handle integers;
decimals are an approximation with error baked in.

(Amusingly, the Go book hits this same wall one test earlier, at 30 seconds
— because Go evaluates *constant* expressions like `math.Pi / 30` with extra
precision, so its two forms disagree in different places than ours. Same
disease, different symptom — which is itself the lesson: where float error
bites is not portable intuition.)

Two ways out: rearrange the arithmetic until the bits happen to agree — a
truce, not a treaty, broken by the next refactor — or admit that *exact
equality is the wrong question for floats* and test "close enough". The
[Structs chapter](structs-methods-and-traits.md) promised this moment would
come, and here is the payoff:

```rust,ignore
const EQUALITY_THRESHOLD: f64 = 1e-7;

fn roughly_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EQUALITY_THRESHOLD
}
```

Within a ten-millionth is overwhelming precision for drawing a clock (we're
not landing on the moon). Swap the table's `assert_eq!` for
`assert!(roughly_equal(got, want), ...)` and we're green — honestly.
(In production code you'd likely reach for the
[approx](https://docs.rs/approx) crate's `assert_relative_eq!`; ours is the
same idea, small enough to own.)

## From angles to vectors

Now, where is the *tip* of the hand? Trigonometry on the unit circle: a ray
at angle `a` from twelve o'clock ends at `x = sin(a)`, `y = cos(a)`. Test
first, on a circle of radius 1 to keep the maths swallowable:

```rust,ignore
{{#include ../code/maths/v1/src/lib.rs:point_test}}
```

And the floats curse us instantly — `sin(π)` comes back as
`0.00000000000000012246...`, an infinitesimal echo of the true zero — which
is why that test was written with `roughly_equal_point` from the start.
The implementation is exactly the formula:

```rust,ignore
{{#include ../code/maths/v1/src/lib.rs:point_fn}}
```

## The other two hands

The minute hand is the same idea with a twist: it creeps as the seconds
pass, so its angle is the minutes' angle *plus* the seconds' angle scaled
down by 60. The hour hand likewise trails the minutes, scaled by 12 (and
`hours % 12`, because clockfaces don't know about afternoons):

```rust,ignore
{{#include ../code/maths/v2/src/lib.rs:radians}}
```

Each of these was test-driven the same way — table of known times against
fraction-of-π expectations, `roughly_equal` throughout (the compounding
fractions make exactness even more hopeless) — and the point functions
collapse into one shared helper:

```rust,ignore
{{#include ../code/maths/v2/src/lib.rs:points}}
```

A curiosity worth flagging: at zero minutes, `30.0 / 0.0` divides by zero.
In Rust (as in Go's runtime), float division by zero isn't an error — it's
`inf`, and `π / inf` is `0`, which happens to be exactly the angle we want.
Spooky, but IEEE 754 knew what it was doing. (Integer division by zero, by
contrast, panics — floats are the permissive ones.)

## Drawing the SVG — and the acceptance test comes back

Time to cash the acceptance-test promise. What we *ship* is SVG text, so the
test should assert on SVG. The tempting cheap version is
`svg.contains("<line x1=\"150\" ...")` — but that's testing a *data
structure* (XML) by its accidental spelling as a string: too fragile (an
innocent extra space breaks it) and too forgiving (it would pass even if the
output weren't valid XML at all). The Go book makes exactly this mistake on
purpose and then corrects itself with `encoding/xml`. We'll go straight to
the correction: parse the output as XML — dev-dependency number three,
[roxmltree](https://docs.rs/roxmltree) — pull out every `<line>`, and check
the hand we expect is among them, floats compared roughly, of course:

```rust,ignore
{{#include ../code/maths/v3/src/lib.rs:xml_helpers}}
```

The acceptance tests then read like the requirement:

```rust,ignore
{{#include ../code/maths/v3/src/lib.rs:acceptance}}
```

Making them pass is the last piece of geometry. Our unit vector is a maths
creature: length 1, y pointing up. The SVG needs it **scaled** to the
hand's length, **flipped** (SVG's y grows downward), and **translated** to
the clock's centre — in exactly that order:

```rust,ignore
{{#include ../code/maths/v3/src/lib.rs:svg}}
```

`svg_writer` takes `&mut impl Write` — the same DI seam as every writer in
this book — which is what let the tests render into a `Vec<u8>`. All four
acceptance tests green.

## Ship it

A `src/main.rs` beside the library turns it into a program. The standard
library can't tell us the wall-clock time of day directly, but it can tell
us the seconds since 1970, and a clockface only needs the remainder
arithmetic (this yields UTC — timezones are chrono's department):

```rust,ignore
{{#include ../code/maths/v3/src/main.rs:main}}
```

```sh
cargo run > clock.svg
```

Open `clock.svg` in a browser: a clock, hands pointing at now. There's
nothing quite like making something you can *look at*.

## Wrapping up

- **Acceptance tests vs unit tests**: one high-level test defining "done"
  (ours parsed real SVG output), with unit-level TDD filling in the maths
  beneath it. The Testing Fundamentals section of this book runs on this
  idea.
- **Never test floats with `==`.** Two algebraically identical expressions
  produced different bits in this very chapter. Define (or import)
  approximate equality with a tolerance your domain justifies, and use it
  everywhere floats flow — including inside other comparisons, like our
  XML line matcher.
- **Test structured output as structure.** String-matching XML makes tests
  simultaneously brittle and blind; parsing it (roxmltree) makes them
  honest.
- **Programs are libraries plus a `main.rs`** — the maths stayed testable
  because writing SVG happens through `impl Write`, and the binary is
  a dozen lines of plumbing.
- And some actual trigonometry: radians, `sin`/`cos` on the unit circle,
  then scale → flip → translate to land in screen coordinates. That last
  pipeline is the seed of every 2D graphics system you'll ever meet.
