## [v0.1.0] - 2026-05-15

### Initial Release

The first official release of `mdgreet`, a clean Material Design 3 greeter for `greetd`.

#### Features
- **Material Design 3**: Full implementation of M3 design language with dynamic color schemes.
- **Dynamic Theming**: Smart theme generation based on the background image with caching.
- **Session Management**: Automatic discovery of Wayland and X11 sessions (Hyprland, Niri, Sway, etc.).
- **User Discovery**: Integration with `AccountsService` for user listing and LRU-based persistence for the last logged-in user.
- **Power Management**: Built-in menu for Shutdown, Reboot, and Suspend actions.
- **Internationalization**: Support for multiple languages (EN, RU, DE, ES, FR).
- **Security**: Full `greetd` IPC integration for secure authentication.
- **Documentation**: Comprehensive user and development guides in mdBook format.

#### Deployment
- Native Nix/NixOS support with Flake and VM testing infrastructure.
- Lightweight and fast async architecture built with Rust and Slint.
