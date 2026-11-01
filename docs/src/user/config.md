# Configuration

`mdgreet` reads `/etc/greetd/mdgreet.toml` by default. Use `-c` or `--config`
to select another TOML file. Only the settings you want to change are needed.

```bash
mdgreet -c /path/to/my/mdgreet.toml
```

## Configuration example

This example covers the main configuration sections. The pages below describe
individual options and their defaults.

```toml
[auth.fingerprint]
# Offer fingerprint login (requires system PAM setup)
enable = false
# Start when the card opens; false requires the fingerprint button
auto_start = true

[appearance]
# The greeting message displayed above the avatar on the login card
greeting = "Welcome Back!"

# Opacity of the login card and power menu (0.0 to 1.0)
opacity = 0.85

# Global font family for UI elements
font_family = "Inter"

[appearance.clock]
font_family = "FlexRounded"
font_size = 220
font_weight = 700

[appearance.theme]
# "default", "slint", "auto", "seed", or "custom"
name = "auto"
mode = "dark"
seed_color = "#1e66f5" # Used only if name = "seed"
# path = "/etc/greetd/my-mdgreet-theme.json" # Used only if name = "custom"

[appearance.background]
path = "/usr/share/backgrounds/my-wallpaper.jpg"
blur = 15.0
color = "#1e1e2e" # Fallback color

[power]
shutdown = "systemctl poweroff"
reboot = "systemctl reboot"
sleep = "systemctl suspend"
hibernate = "systemctl hibernate"

# --- Advanced / Technical Settings ---
# These settings are primarily for debugging or environments with non-standard paths.
# Most users can safely omit these blocks.

[logging]
level = "info"
# path = "/var/log/mdgreet/mdgreet.log"

[cache]
# path = "/var/cache/mdgreet"
```

## Configuration reference

- [Authentication](config/authentication.md): fingerprint login, automatic or
  explicit start and defaults.
- [Appearance](config/appearance.md): greeting, opacity, typography and clock.
- [Theming](config/themes.md): theme sources, colours and mode.
- [Background](config/background.md): wallpaper, blur and fallback colour.
- [Power management](config/power.md): commands used by the power menu.

To enable fingerprint login, configure both the interface and system
[authentication setup](fingerprint.md):

```toml
[auth.fingerprint]
enable = true
auto_start = false
```

`enable` defaults to `false`; `auto_start` defaults to `true`. See the
[authentication reference](config/authentication.md) for the full behaviour.
