# MDGreet

> **🚧 Work In Progress 🚧**
>
> This project is currently in active development. Features, configuration formats, and architecture are subject to change.

**MDGreet** is a modern, clean, and highly customizable [greetd](https://git.sr.ht/~kennylevinsen/greetd) greeter built with Rust and the [Slint](https://slint.dev/) UI framework. It features dynamic Material Design 3 (MD3) theming, allowing it to adapt its color palette based on your wallpaper or custom seed colors.

## Features

- **Material Design 3**: Fully embraces the MD3 design language.
- **Dynamic Theming**: Automatically generates color schemes from your chosen background image.
- **Wayland Native**: Designed specifically for modern Wayland environments (runs via `cage`).
- **Configuration-Driven**: Easy to set up via a single TOML file.
- **NixOS Ready**: Includes a flake and a test VM setup for easy development and deployment.

## Building & Testing

We recommend using Nix for development to ensure all Wayland dependencies are met.

```bash
# Enter the development shell
nix develop

# Build the project
cargo build

# Test the greeter in a safe, isolated QEMU VM
nix run .#vm
```

## License

This project is licensed under the **GPL-3.0** License.
