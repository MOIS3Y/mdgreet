{
  pkgs,
  wlLibs,
  package,
}:
let
  i18nTools = pkgs.callPackage ./i18n.nix { };
in
pkgs.mkShell {
  inputsFrom = [ package ];

  packages = with pkgs; [
    # rust
    clippy
    rustfmt
    rust-analyzer
    
    # slint
    slint-lsp
    slint-viewer

    # docs
    mdbook
    git-cliff
  ] ++ i18nTools;

  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

  shellHook = ''
    # Fix kvantum error on my device
    export QT_STYLE_OVERRIDE="Fusion"

    # Fix Runtime deps in development
    export LD_LIBRARY_PATH="${wlLibs}:$LD_LIBRARY_PATH"
  '';
}
