# Templating

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/templating)**

We're continuing the blog software from the [Reading files](reading-files.md)
chapter. There we turned files into `Post`s; now we turn `Post`s into HTML —
a page per post, and an index linking them together.

Go's book uses `html/template`, which parses templates and checks their field
references *at runtime*. Rust's ecosystem offers both styles, and we're going
to reach for the one that fits this book's whole argument:
[askama](https://docs.rs/askama), which compiles your templates into Rust code
*at build time*. A template that refers to a field your data doesn't have
isn't a blank space in production — it's a compiler error, right next to your
type mismatches and borrow errors. We'll prove that at the end; keep it in
mind as we go.

The `Post` type carries over unchanged:

```rust,ignore
{{#include ../code/templating/v1/src/lib.rs:post}}
```

## Write the test first

Think smallest-useful-step: viewing a single post matters more than an index
(you can share direct links before you have a listing). And even a post is
a lot — heading, description, tags, a markdown body. So the very first slice
is just: render the title as an `<h1>`. Our writer-based design, familiar from
every I/O chapter, makes the test trivially inspectable — render into a
`Vec<u8>` and read it back:

```rust,ignore
{{#include ../code/templating/v1/src/lib.rs:test}}
```

## Write enough code to make it pass

After the compiler walks us through the missing `render` (stub it returning
`Ok(())`, watch it fail with `left: "", right: "<h1>hello world</h1>"`), here's
the real thing:

```rust,ignore
{{#include ../code/templating/v1/src/lib.rs:code}}
```

There's a fair bit of machinery here for one heading, so let's unpack it,
because it's the shape of everything to come:

- **The template lives in a file**, `templates/blog.html`, containing just
  `<h1>{{ title }}</h1>`. Askama looks in a `templates/` directory next to
  your crate by default.
- **`#[derive(Template)]` with `#[template(path = "blog.html")]`** is where
  the magic happens: at compile time, askama reads that file and *generates*
  the rendering code as a method on `BlogTemplate`. The struct's fields are
  the data the template can see — here, `title`.
- **`write_into`** streams the rendered output into any `std::io::Write` — the
  same dependency-injection seam as the Maths and Reading files chapters,
  which is exactly why the test could hand it a `Vec<u8>`.

Unlike Go, there's no "parse the template" step that can fail at runtime with
a bad template — the parsing happened when we compiled. `render` can still
return `askama::Result` because *writing* can fail (a broken pipe, a full
disk), but a malformed template never reaches production.

## Fill out the post, and meet auto-escaping

Now the whole post. The template gets fields for description and a loop over
tags — askama's control flow uses `{% %}`, close cousin of Jinja and Go's own
template syntax:

```html
<h1>{{ post.title }}</h1>

<p>{{ post.description }}</p>

Tags: <ul>{% for tag in post.tags %}<li>{{ tag }}</li>{% endfor %}</ul>
```

The template now takes the whole `Post` by reference, and we add a second
template and function for the index we're about to build:

```rust,ignore
{{#include ../code/templating/v2/src/lib.rs:code}}
```

Now for something Go's chapter makes a point of, and askama gives us for free.
Because the template file ends in `.html`, askama turns on **HTML
auto-escaping**: any value substituted into it has its `<`, `>`, `&` and
friends encoded, so untrusted post content can't inject markup. We can pin
that down with a test:

```rust,ignore
{{#include ../code/templating/v2/src/lib.rs:escape_test}}
```

A title of `<script>alert('xss')</script>` comes out as
`&#60;script&#62;...`, inert. This is the same protection Go's `html/template`
provides over `text/template`, chosen automatically from the file extension.
Security that's on by default is security you can't forget to turn on.

## The index, and a custom filter

The index lists posts as links. The wrinkle — the same one the Go book hits —
is the URL. We want `Hello World` displayed to the reader, but
`/post/hello-world` in the `href`: spaces lowercased to hyphens. We can't
mangle the titles in memory (we still want the spaces on screen), so we
transform *only where it's used in the template*, with a **filter**:

```html
<ol>{% for post in posts %}<li><a href="/post/{{ post.title|slug }}">{{ post.title }}</a></li>{% endfor %}</ol>
```

`{{ post.title|slug }}` pipes the title through our `slug` filter, which is an
ordinary Rust function askama finds in a `filters` module:

```rust,ignore
{{#include ../code/templating/v2/src/lib.rs:filters}}
```

The `#[askama::filter_fn]` attribute registers it; the `&dyn askama::Values`
parameter is filter plumbing we don't need here. The test confirms the two
halves — hyphenated slug in the link, original title in the text:

```rust,ignore
{{#include ../code/templating/v2/src/lib.rs:test}}
```

Go solves this identically — a `sanitiseTitle` function passed into the
template — because it's the right shape: display data stays display data, and
the URL transformation happens at the point of use. (This index test uses
exact string matching, which is fine for one tidy line. For the sprawling
multi-line post HTML, exact matching gets brittle fast — the Rust ecosystem's
answer to Go's "approval tests" is snapshot testing with the
[insta](https://docs.rs/insta) crate, which stores a reviewed `.snap` file and
diffs against it. Worth knowing; we've kept to focused `contains` assertions
here to avoid another dependency.)

## Rendering the markdown body

The body is markdown and needs to become HTML. We reach for
[pulldown-cmark](https://docs.rs/pulldown-cmark), the standard Rust markdown
parser — our fourth and final dependency of the chapter:

```rust,ignore
{{#include ../code/templating/v3/src/lib.rs:code}}
```

Two things collaborate here. `markdown_to_html` runs the body through
pulldown-cmark to produce an HTML fragment. Then — crucially — the template
renders it with the `|safe` filter:

```html
{{ html_body|safe }}
```

Without `|safe`, auto-escaping would kick in and display the literal text
`<em>...</em>` instead of emphasising it — the very feature that protected us
from injection now working against us, because *this* HTML we actually trust.
`|safe` is askama's escape hatch, the exact analogue of Go wrapping trusted
content in `template.HTML`. And the danger is the same in both languages: only
mark HTML safe when you genuinely trust its source, because you're switching
off the guard. Our markdown comes from our own post files, so we trust it.

The test confirms markdown became real HTML:

```rust,ignore
{{#include ../code/templating/v3/src/lib.rs:test}}
```

Note that `html_body` is a computed field on the template struct — the
rendering happens in Rust, and the template just places the result. Go builds
an unexported `postViewModel` for the same reason; ours is the `BlogTemplate`
struct itself, carrying both the borrowed `post` and the owned `html_body`.

## The payoff: templates checked at compile time

Here's the promise from the top of the chapter. In Go, if you fat-finger a
field name in a template — `{{ .Titel }}` — it compiles fine and fails, or
silently renders nothing, at *runtime*, when a user hits the page. Try the
equivalent typo in an askama template, `<h1>{{ titel }}</h1>` against a struct
whose field is `title`:

```text
error[E0609]: no field `titel` on type `&BlogTemplate<'a>`
 --> src/lib.rs:3:10
  |
3 | #[derive(Template)]
  |          ^^^^^^^^ unknown field
  |
  = note: this error originates in the derive macro `Template`
```

**It doesn't compile.** The template's reference to `titel` became Rust code
reaching for a `.titel` field that isn't there, and the borrow checker's
colleague — the type checker — caught it, before the program could run, before
a test even had to. A whole category of Go's runtime template bugs simply
cannot survive `cargo build` here. This is the same trade we've seen since the
[Concurrency](concurrency.md) chapter, now applied to HTML: work the compiler
does up front is work your users never trip over.

## Wrapping up

- **askama compiles templates to Rust at build time** via `#[derive(Template)]`.
  Template files live in `templates/`; struct fields are the template's data;
  `write_into` streams to any `io::Write`, our usual DI seam.
- **Auto-escaping is on by default for `.html` templates** — the injection
  protection you can't forget — and **`|safe`** deliberately switches it off
  for HTML you trust, exactly like Go's `template.HTML`.
- **Filters are plain Rust functions** (`#[askama::filter_fn]`), the clean way
  to transform a value at its point of use — like slugging a title for a URL
  while leaving the displayed text alone.
- **A template that references a missing field is a compile error**, not a
  runtime surprise. That is the whole reason to prefer a typed, compile-time
  templating engine, and it's the most Rust thing in this chapter.

Combine this with Reading files and you have a well-tested static site
generator: read a folder of markdown, render it to HTML, serve it. Generating
HTML on the server from data — a file system, a database, an API — is a simple,
durable technique. The next section of the book builds exactly that kind of
server.
