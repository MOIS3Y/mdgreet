# Configuration

`mdgreet` reads its configuration from a TOML file. 

By default, it looks for the configuration file in standard locations like `/etc/greetd/mdgreet.toml`. You can also specify a custom path using the `-c` or `--config` command-line argument.

```bash
mdgreet -c /path/to/my/mdgreet.toml
```

## Full Configuration Example

Here is a comprehensive example showing all available settings:

```toml
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

[appearance.background]
path = "/usr/share/backgrounds/my-wallpaper.jpg"
blur = 15.0
color = "#1e1e2e" # Fallback color

[power]
shutdown = "systemctl poweroff"
reboot = "systemctl reboot"
sleep = "systemctl suspend"
hibernate = "systemctl hibernate"

[logging]
level = "info"
```

Navigate through the subsections to learn more about specific configuration blocks.
