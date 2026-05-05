{
  description = "A clean MD3 greetd greeter in Rust/Slint with dynamic colors";

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
        # nixpkgs aliases
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;

        # package meta
        cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
        pname = cargoToml.package.name;
        version = cargoToml.package.version;

        # runtime deps
        waylandDependencies = with pkgs; [
          wayland
          wayland-protocols
          libxkbcommon
          mesa
          libGL
          fontconfig
          freetype
        ];

        # LD lib paths
        wlLibs = lib.makeLibraryPath waylandDependencies;

        # dev test VM
        vmConfig = self.nixosConfigurations.vm.config;
        vmBuildDir = vmConfig.system.build.vm;
        vmHostName = vmConfig.networking.hostName;
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
            pkgs.makeWrapper
          ];

          buildInputs = waylandDependencies;

          postInstall = ''
            wrapProgram $out/bin/${pname} --prefix LD_LIBRARY_PATH : "${wlLibs}"
          '';

          meta = {
            description = "A clean MD3 greetd greeter in Rust/Slint";
            license = lib.licenses.gpl3Plus;
            mainProgram = pname;
          };
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = self.packages.${system}.default;
          };
          vm = {
            type = "app";
            program = "${vmBuildDir}/bin/run-${vmHostName}-vm";
          };
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
            export QT_STYLE_OVERRIDE="Fusion"

            # Fix Runtime deps in development
            export LD_LIBRARY_PATH="${wlLibs}:$LD_LIBRARY_PATH"
          '';
        };
      }
    )
    // {
      # NixOS configurations must be outside of eachDefaultSystem
      nixosConfigurations.vm = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        specialArgs = {
          inherit self;
          system = "x86_64-linux";
        };
        modules = [ ./vm.nix ];
      };
    };
}
