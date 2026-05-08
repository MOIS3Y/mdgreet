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
            pkgs.gettext
            pkgs.slint-tr-extractor
          ];

          buildInputs = waylandDependencies;

          postInstall = ''
            # Install translations
            mkdir -p $out/share
            cp -r $(find target -name locales -type d | head -n 1) $out/share/locale

            wrapProgram $out/bin/${pname} \
              --prefix LD_LIBRARY_PATH : "${wlLibs}" \
              --set MDGREET_LOCALES_DIR "$out/share/locale"
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

        devShells.default = import ./nix/shell.nix {
          inherit pkgs wlLibs;
          package = self.packages.${system}.default;
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
        modules = [ ./nix/vm.nix ];
      };
    };
}
