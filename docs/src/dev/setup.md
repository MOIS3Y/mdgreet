# Setting up the Environment

mdgreet is primarily developed within a Nix environment, ensuring reproducible and isolated dependencies.

## Prerequisites

- Nix with Flakes enabled.

## Entering the Development Shell

Simply navigate to the project root and run:

```bash
nix develop
```

This will drop you into a shell equipped with:
- The Rust toolchain (`cargo`, `rustc`, `clippy`, `rustfmt`, `rust-analyzer`).
- Slint dependencies (`slint-lsp`, `slint-viewer`).
- Wayland development libraries.
- i18n tools (`gettext`).
- Documentation tools (`mdbook`).

## Running the Application Locally

You can run the application directly using Cargo. Note that without a Wayland compositor or `greetd` running, you must use the `--demo` flag to simulate a login environment.

```bash
cargo run -- --demo
```
