# Installation

The recommended and easiest way to install mdgreet is via Nix.

## Nix (Flakes)

If you are using NixOS with Flakes enabled, you can run mdgreet directly or add it to your system configuration.

To test mdgreet without installing it, you can run:

```bash
nix run github:MOIS3Y/mdgreet
```

To include it in your NixOS configuration as a package:

```nix
{
  inputs.mdgreet.url = "github:MOIS3Y/mdgreet";

  outputs = { self, nixpkgs, mdgreet, ... }: {
    nixosConfigurations.myHost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        # ... your configuration ...
        ({ pkgs, ... }: {
          environment.systemPackages = [ mdgreet.packages.x86_64-linux.default ];
        })
      ];
    };
  };
}
```

## Building from Source

If you are not using Nix, you can build mdgreet using Cargo. Ensure you have Rust and the necessary dependencies (Wayland, Fontconfig, etc.) installed.

```bash
git clone https://github.com/MOIS3Y/mdgreet.git
cd mdgreet
cargo build --release
```

The compiled binary will be located at `target/release/mdgreet`. You will need to manually place it in a location accessible to your system (e.g., `/usr/local/bin/`).
