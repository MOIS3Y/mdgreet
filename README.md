# mdgreet

**A clean, fast, and visually appealing Material Design 3 greeter for
greetd, built with Rust and Slint.**

mdgreet is designed to provide a modern login experience for Linux
environments. It leverages the Material Design 3 guidelines to offer a
polished interface that can dynamically adapt its color palette to match
your wallpaper.

<div align="center">

[![Docs](https://img.shields.io/badge/docs-latest-blue?style=for-the-badge&labelColor=101418)](https://mois3y.github.io/mdgreet/)
![Rust](https://img.shields.io/badge/Rust-ea4a31?style=for-the-badge&logo=rust&logoColor=white&labelColor=101418)
![Slint](https://img.shields.io/badge/Slint-2391f7?style=for-the-badge&logo=rust&logoColor=white&labelColor=101418)
![NixOS](https://img.shields.io/badge/NixOS-5277C3?style=for-the-badge&logo=nixos&logoColor=white&labelColor=101418)
[![License](https://img.shields.io/badge/License-GPLv3-blue.svg?style=for-the-badge&labelColor=101418)](./LICENSE)

<br>

<img src="docs/src/assets/main-screen.png" width="48%" alt="Main Screen">
<img src="docs/src/assets/login-screen.png" width="48%" alt="Login Card">

</div>

## Key Features

- **Material Design 3:** Adheres strictly to MD3 principles for a familiar
  and modern aesthetic.
- **Dynamic Theming:** Automatically generates comprehensive color schemes
  from your wallpaper or a specific seed color (Material You).
- **Smooth Transitions:** Featuring graceful background blur and interface
  animations.
- **Built for Wayland:** Designed for modern Wayland environments, with
  native support for starting both Wayland and X11 user sessions.
- **Fast & Lightweight:** Written in Rust to ensure minimal resource
  overhead and near-instant startup.
- **Nix-First:** Native support for Nix Flakes, including an integrated
  QEMU test VM for development.

## Usage Guide

Comprehensive guides on installation, configuration, and development are
available in the official **[Documentation](https://mois3y.github.io/mdgreet/)**.

## Inspirations & Related Projects

- **[ReGreet](https://github.com/rharish101/ReGreet):** A clean GTK4
  greeter for greetd. I've learned a lot from this project regarding
  `greetd` IPC communication and LRU caching logic.
- **[pixie-sddm](https://github.com/xCaptaiN09/pixie-sddm):** A beautiful
  Material Design SDDM theme. This project served as a major inspiration
  for mdgreet's visual design and layout.

## License

This project is licensed under the **GPL-3.0** License.
