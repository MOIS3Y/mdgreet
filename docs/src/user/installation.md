# Installation

The recommended way to install mdgreet is via Nix, but it can also be built and installed manually on any standard Linux distribution.

## Nix (Flakes)

If you are using NixOS with Flakes, there are several ways to integrate mdgreet. The most common approach is to add it as a flake input and then pass it to your system configuration.

### 1. Add to your Flake Inputs

In your `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    mdgreet.url = "github:MOIS3Y/mdgreet";
  };

  outputs = { self, nixpkgs, mdgreet, ... }: {
    nixosConfigurations.myHost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      # Pass the 'mdgreet' input to your modules
      specialArgs = { inherit mdgreet; };
      modules = [ ./configuration.nix ];
    };
  };
}
```

### 2. Include as a System Package

Then, in your `configuration.nix`, you can access the package and add it to your system:

```nix
{ pkgs, mdgreet, ... }: {
  environment.systemPackages = [ 
    mdgreet.packages.${pkgs.stdenv.hostPlatform.system}.default 
  ];
}
```

### Alternative: Using Overlays

For a more seamless integration, you can add mdgreet as an overlay. This allows you to use `pkgs.mdgreet` anywhere in your configuration:

```nix
{
  nixpkgs.overlays = [
    (final: prev: {
      mdgreet = mdgreet.packages.${prev.stdenv.hostPlatform.system}.default;
    })
  ];

  environment.systemPackages = [ pkgs.mdgreet ];
}
```

## Building from Source

To build mdgreet manually, you need the Rust toolchain and several development libraries.

### Prerequisites

Ensure the following dependencies are installed on your system:

- **Build-time**: `cargo`, `rustc`, `pkg-config`, `gettext`.
- **Runtime**: `wayland`, `libxkbcommon`, `fontconfig`, `mesa` (OpenGL), `freetype`.

### 1. Compilation

Clone the repository and build the release binary:

```bash
git clone https://github.com/MOIS3Y/mdgreet.git
cd mdgreet
cargo build --release
```

The binary will be generated at `target/release/mdgreet`. Copy it to your system path:

```bash
sudo cp target/release/mdgreet /usr/local/bin/
```

### 2. Install Translations

mdgreet uses `gettext` for internationalization. Compiled locales must be placed in the standard system locale directory.

```bash
# Locate and copy compiled locales
LOCALES_DIR=$(find target -name locales -type d | head -n 1)
sudo cp -r "$LOCALES_DIR"/* /usr/share/locale/
```

### 3. Setup System Directories

mdgreet requires specific directories for caching dynamic themes and storing logs. These must be owned by the user running the greeter (usually `greeter`).

```bash
# Create directories
sudo mkdir -p /var/cache/mdgreet
sudo mkdir -p /var/log/mdgreet

# Set permissions
sudo chown -R greeter:greeter /var/cache/mdgreet
sudo chown -R greeter:greeter /var/log/mdgreet
```
