# Install Rust

The official installation guide is at [rust-lang.org/tools/install][install].
Rust moves fast enough that it's the source of truth; this page just covers what
you need for this book.

[install]: https://www.rust-lang.org/tools/install

## rustup

Rust is installed through `rustup`, which manages toolchains and keeps them
updated. On Linux and macOS:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Linux, your distribution likely packages `rustup` too — `pacman -S rustup` on
Arch, for instance — which is often tidier than the script. On Windows, use the
installer from the link above.

Then check it worked:

```sh
rustc --version
cargo --version
```

You want a stable toolchain from 1.85 or later, since this book uses the 2024
edition. Newer is fine.

## Cargo

`cargo` is Rust's build tool, test runner, and package manager in one. You'll use
three commands constantly:

```sh
cargo new my-project   # create a project
cargo test             # run the tests
cargo run              # build and run it
```

There's no separate test framework to choose or install. Testing is part of the
language and the tooling, which is a large part of why Rust suits this book.

## An editor

Get [rust-analyzer][ra]. It's the language server, it works in VS Code, Neovim,
Helix, JetBrains, and most other editors, and it will show you type errors as you
type rather than when you compile.

This matters more in Rust than elsewhere. Much of what you'll learn in the early
chapters is a conversation with the compiler, and rust-analyzer moves that
conversation from your terminal into your editor, where it's faster.

[ra]: https://rust-analyzer.github.io/

## Two more tools

Both ship with rustup and both are used by this book's CI:

```sh
cargo fmt      # format your code, no arguments about style
cargo clippy   # lints; it will teach you idiomatic Rust for free
```

Run `cargo clippy` when a chapter's refactor step feels unfinished. It is a very
good teacher.

## Now

On to [Hello, World](hello-world.md).
