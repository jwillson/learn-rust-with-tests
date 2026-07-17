# WebSockets

**[You can find all the code for this chapter here](https://github.com/jwillson/learn-rust-with-tests/tree/main/code/websockets)**

To finish the application section, we'll let people play poker in a browser. A
web page opens a **WebSocket** — a persistent, two-way connection — to our
server; the player sends the number of players and, later, the winner, and the
server drives the game across that live connection instead of a one-shot
request/response.

The lesson here is twofold: WebSockets in axum, and — more importantly — that
the *game logic shouldn't care how it's delivered*. The CLI and the WebSocket
server want the exact same thing: start a game with N players, finish it with a
winner. So we hoist that into a `Game` abstraction, and both front ends drive
it.

## The `Game` abstraction

Both delivery mechanisms need to "start" and "finish" a game. That's a trait:

```rust,ignore
{{#include ../code/websockets/v1/src/lib.rs:game}}
```

The real implementation, `TexasHoldem`, holds the store and alerter we built in
the [Time chapter](time.md), and does the actual work — scheduling blinds on
start (now the timing *stretches with the number of players*, a real poker
detail), recording the winner on finish:

```rust,ignore
{{#include ../code/websockets/v1/src/lib.rs:texas}}
```

And because `Game` is a trait, its behaviour is unit-testable with the same spy
approach as everything else — no sockets, no browser, just checking `start`
schedules the right blinds and `finish` records the winner:

```rust,ignore
{{#include ../code/websockets/v1/src/lib.rs:test}}
```

This is the payoff of the whole section stated plainly: the game's rules are
tested in microseconds against spies, entirely separate from *how* a player
reaches them. The WebSocket is just plumbing.

## Write the test first

Now the plumbing. We want a server that, over a WebSocket, reads the player
count, starts the game, reads the winner, and finishes it. The honest way to
test a WebSocket is to *be a WebSocket client*: stand up a real server on a
free port, connect to it, exchange real messages, and assert on a spy `Game`.

```rust,ignore
{{#include ../code/websockets/v2/src/lib.rs:test}}
```

Walking through it: bind a `TcpListener` on port 0 (the OS picks a free port,
the trick from the [Select chapter](select.md)), serve the app on a background
task with `tokio::spawn`, then connect a real client with
[`tokio-tungstenite`](https://docs.rs/tokio-tungstenite). We send two text
messages — `"3"` and `"Ruth"` — and, after a beat for the server to process
them, assert the spy `Game` was `start`ed with 3 and `finish`ed with `"Ruth"`.

The spy is our now-familiar shape — record what it was called with, behind a
`Mutex` because the server touches it from its own task:

```rust,ignore
{{#include ../code/websockets/v2/src/lib.rs:spy}}
```

## Write enough code to make it pass

axum has first-class WebSocket support (behind the `ws` feature). A handler
takes a `WebSocketUpgrade` extractor and returns `ws.on_upgrade(...)`, handing
control to an async function once the connection is established:

```rust,ignore
{{#include ../code/websockets/v2/src/lib.rs:server}}
```

The pieces:

- **`ws_handler`** extracts the injected `Game` (via `State`, as ever) and the
  `WebSocketUpgrade`. `on_upgrade` completes the WebSocket handshake and then
  runs `play_game` with the live socket.
- **`play_game`** is the protocol: read a text message and parse it as the
  player count to `start` the game, then read another as the winner to
  `finish` it. `socket.recv().await` waits for the next message from the
  client — the `.await` yielding to the runtime while nothing has arrived.
- **The `if let ... && let ...` chain** is Rust's *let-chains* (stabilised in
  the 2024 edition): read the message *and* successfully parse it, or do
  nothing. It's the same "only act when everything matched" idea as the CLI's
  `if let Some(name)`, extended to two conditions.

The test passes: a genuine WebSocket handshake, two genuine frames, and the
game driven exactly as the browser would drive it. Wire a `TexasHoldem` in
place of the spy in `main`, serve a little HTML page with a few lines of
JavaScript to open the socket, and you can play poker in a browser — the same
`Game` the CLI runs, reached a different way.

## Wrapping up

- **WebSockets give you a persistent, two-way channel.** In axum, a
  `WebSocketUpgrade` extractor plus `on_upgrade` hands you a `WebSocket` you
  `recv` from and `send` to across the life of the connection — a different
  shape from request/response, same handler ergonomics.
- **Test WebSockets by being a client.** Bind on port 0, serve on a background
  task, connect with `tokio-tungstenite`, exchange real frames. No mocking the
  protocol — exercise the real thing, fast, on a throwaway port.
- **Deliver-mechanism-independent logic is the win.** By putting the rules
  behind a `Game` trait, the CLI and the WebSocket server share one tested
  core. Adding a third front end — a different protocol, a message queue —
  would mean writing new plumbing against the same `Game`, not rewriting the
  game.

That completes our application: a poker league that persists to disk, serves a
JSON API and a browser game over WebSockets, runs from the command line, and
schedules blinds on a timer — every piece of it driven out with tests, and
every external dependency injected behind a trait so the logic could be tested
in isolation. The next section steps back from building features to think about
*how we test* — acceptance tests, working without mocks, and the habits that
keep a growing codebase honest.
