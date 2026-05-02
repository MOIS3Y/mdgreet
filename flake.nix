{
  description = "greetd greeter";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;

        cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
        pname = cargoToml.package.name;
        version = cargoToml.package.version;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          inherit pname version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [
            pkgs.pkg-config
          ];

          buildInputs = [
            pkgs.fontconfig
            pkgs.freetype
            pkgs.wayland
            pkgs.wayland-protocols
            pkgs.libxkbcommon
            pkgs.mesa
            pkgs.libGL
          ];

          meta = {
            description = "greetd greeter";
            license = lib.licenses.mit;
            mainProgram = pname;
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];

          packages = with pkgs; [
            # rust
            clippy
            rustfmt
            rust-analyzer
            # slint
            slint-lsp
            slint-viewer
          ];

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          shellHook = ''
            # Fix kvantum error on my device
            export QT_STYLE_OVERRIDE="Fusion";

            # Fix Runtime deps in development
            export LD_LIBRARY_PATH="${lib.makeLibraryPath [
              pkgs.wayland
              pkgs.wayland-protocols
              pkgs.fontconfig
              pkgs.freetype
              pkgs.libxkbcommon
              pkgs.mesa
              pkgs.libGL
            ]}:$LD_LIBRARY_PATH"
          '';
        };
      }
    );
}

