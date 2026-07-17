# Revisiting HTTP handlers

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/http-handlers-revisited)**

We built HTTP handlers back in the [HTTP server](http-server.md) chapter. This
chapter steps back to answer a design question people wrestle with constantly:
*what should an HTTP handler actually do, and how do you keep it testable?* The
answer is a shape you can apply to almost any handler you'll ever write.

## What a handler should do

No matter the language or framework, a good HTTP handler has the same three
responsibilities, and *only* these:

1. **Accept the request, parse and validate it.**
2. **Call some service** to do the important business logic with that data.
3. **Send an appropriate response** based on what the service returned.

The trap is a handler that also connects to a database, runs queries, applies
business rules, formats results — all inline. That handler is miserable to test
(you need a real database to exercise any of it) and impossible to reuse (the
logic is welded to HTTP). Separate the three concerns and everything gets
easier: the handler tests trivially with a fake service, and the business logic
tests with no HTTP in sight — the [dependency injection](dependency-injection.md)
lesson, applied to the web layer.

## The shape in Rust

Here's a "register a user" endpoint. The service — the "important business
logic" — is a trait, and everything the handler needs from the outside world
arrives as a typed extractor:

```rust,ignore
{{#include ../code/http-handlers-revisited/v1/src/lib.rs:types}}
```

```rust,ignore
{{#include ../code/http-handlers-revisited/v1/src/lib.rs:handler}}
```

Look at how directly the handler maps onto the three responsibilities:

1. **Parse and validate** — `Json(new_user): Json<NewUser>`. axum's `Json`
   extractor decodes the request body into `NewUser`, and here Rust hands us
   something the Go version has to do by hand: **if the body isn't valid JSON,
   the extractor rejects the request with `400 Bad Request` before our handler
   ever runs.** Parsing and its error path are handled by the type system and
   the framework; we write zero code for it.
2. **Call the service** — `service.register(new_user)`. The handler depends on
   the `UserService` *trait*, not on a database, so it neither knows nor cares
   how registration actually happens.
3. **Respond by mapping the result** — a `match` on the service's `Result`
   turns each outcome into the right status: `Ok(id)` becomes `201 Created`
   with the id, `AlreadyExists` becomes `409 Conflict`, an infrastructure
   failure becomes `500`. This is the [error-types chapter](error-types.md)'s
   enum paying off: because the service returns a *typed* error, the handler can
   give each failure the HTTP status it deserves rather than a blanket 500.

## The tests are a breeze

Because the handler only talks to a trait, testing it means passing a fake
service and asserting on the HTTP response. No database, no fixtures:

```rust,ignore
{{#include ../code/http-handlers-revisited/v1/src/lib.rs:mock}}
```

The happy path — valid body in, `201` and the id out, and the service saw the
right user:

```rust,ignore
{{#include ../code/http-handlers-revisited/v1/src/lib.rs:happy_test}}
```

And the failure paths, each one a focused check on a single responsibility:

```rust,ignore
{{#include ../code/http-handlers-revisited/v1/src/lib.rs:sad_tests}}
```

Notice the malformed-payload test asserts two things: the response is `400`,
*and the service was never called*. That's the separation working — a bad
request is rejected at the parsing boundary and never reaches the business
logic. And the duplicate-user test drives a `409` purely by having the fake
service return `AlreadyExists`; the handler's job is just to translate that
outcome, and that's all the test checks.

Meanwhile — and this is the real prize — the *actual* `UserService`
implementation is tested entirely separately, with no HTTP anywhere, because
nothing about registering a user depends on HTTP. The concern that's hard to
test (the business logic) and the concern that's fiddly to set up (the web
layer) never contaminate each other.

## Wrapping up

- **An HTTP handler has three jobs**: parse/validate the request, call a
  service, respond based on the result. Resist letting it do anything else.
- **Depend on a service trait, not an implementation.** The handler becomes
  trivial to test with a fake, and the business logic becomes testable with no
  HTTP at all — and reusable outside the web layer.
- **Let the framework and types do the boring parts.** axum's `Json` extractor
  parses *and* returns `400` on bad input for free; a typed error enum from the
  service lets the handler map each failure to a precise status code instead of
  a catch-all `500`.
- This shape scales. When registration grows more complex, the handler doesn't
  change — only the service does, behind its unchanged trait.
