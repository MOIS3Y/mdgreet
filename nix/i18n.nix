# ==========================
# i18n Development Tools FAQ
# ==========================
#
# These tools manage the translation workflow for mdgreet.
# They are automatically available in the `nix develop` shell.
#
# Q: I added a new translatable string in Rust or Slint. What do I do?
# A: 1. Run `i18n-extract` to scan the codebase and update `po/mdgreet.pot`.
#    2. Run `i18n-update <locale>` (e.g., `i18n-update ru`) to merge the new
#       strings into the existing translation file (`po/ru.po`).
#    3. Edit `po/ru.po` to translate the new empty entries.
#
# Q: How do I add support for a completely new language?
# A: 1. Ensure the template is up to date: `i18n-extract`
#    2. Run `i18n-init <locale>` (e.g., `i18n-init fr`). This creates a new
#       file `po/fr.po`.
#    3. Edit the header in `po/fr.po` (optional but recommended) and start
#       translating.
#
# Q: How do I test my translations locally?
# A: Cargo handles compilation automatically via `build.rs`. Just run:
#    `LANG=<locale>.UTF-8 cargo run` (e.g., `LANG=ru_RU.UTF-8 cargo run`).

{ pkgs }:
let
  i18n-extract = pkgs.writeShellApplication {
    name = "i18n-extract";
    runtimeInputs = [
      pkgs.gettext
      pkgs.slint-tr-extractor
      pkgs.findutils
    ];
    text = ''
      echo "Extracting strings from Rust and Slint..."

      # Ensure po directory exists
      mkdir -p po

      # Extract from Rust
      find src -name "*.rs" -print0 | xargs -0 xgettext \
        --from-code=UTF-8 \
        --language=Rust \
        --keyword=gettext \
        -o po/rust.pot

      # Extract from Slint
      find ui -name "*.slint" -print0 | xargs -0 slint-tr-extractor \
        -o po/slint.pot

      # Merge
      msgcat po/rust.pot po/slint.pot -o po/mdgreet.pot

      # Cleanup
      rm po/rust.pot po/slint.pot

      echo "Done! Template saved to po/mdgreet.pot"
    '';
  };

  i18n-init = pkgs.writeShellApplication {
    name = "i18n-init";
    runtimeInputs = [ pkgs.gettext ];
    text = ''
      if [ -z "''${1:-}" ]; then
        echo "Usage: i18n-init <locale> (e.g., ru)"
        exit 1
      fi

      LOCALE=$1
      PO_FILE="po/''${LOCALE}.po"

      if [ ! -f po/mdgreet.pot ]; then
        echo "Error: po/mdgreet.pot not found. Run 'i18n-extract' first."
        exit 1
      fi

      if [ -f "$PO_FILE" ]; then
        echo "Error: $PO_FILE exists. Use 'i18n-update $LOCALE' instead."
        exit 1
      fi

      msginit \
        --input=po/mdgreet.pot \
        --locale="$LOCALE" \
        --output="$PO_FILE" \
        --no-translator

      echo "Initialized $PO_FILE"
    '';
  };

  i18n-update = pkgs.writeShellApplication {
    name = "i18n-update";
    runtimeInputs = [ pkgs.gettext ];
    text = ''
      if [ -z "''${1:-}" ]; then
        echo "Usage: i18n-update <locale> (e.g., ru)"
        exit 1
      fi

      LOCALE=$1
      PO_FILE="po/''${LOCALE}.po"

      if [ ! -f po/mdgreet.pot ]; then
        echo "Error: po/mdgreet.pot not found. Run 'i18n-extract' first."
        exit 1
      fi

      if [ ! -f "$PO_FILE" ]; then
        echo "Error: $PO_FILE not found. Use 'i18n-init $LOCALE' first."
        exit 1
      fi

      msgmerge --update "$PO_FILE" po/mdgreet.pot
      echo "Updated $PO_FILE"
    '';
  };
in
[
  i18n-extract
  i18n-init
  i18n-update
]
