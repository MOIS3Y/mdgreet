# Setup with greetd

`mdgreet` is designed to be run by [greetd](https://sr.ht/~kennylevinsen/greetd/), a minimal and flexible login manager daemon.

To use `mdgreet`, you need to configure `greetd` to launch a Wayland compositor (like `cage`, `sway`, or `niri`), which in turn runs `mdgreet`. 

> [!NOTE]
> The `mdgreet.toml` configuration file is entirely optional. If not provided, mdgreet will use sensible defaults. However, creating it is highly recommended to customize the appearance to your liking.

Please choose your distribution type below:

- [NixOS Configuration](./greetd/nixos.md)
- [Standard Linux Configuration](./greetd/standard.md)
