# Appearance Configuration

The `[appearance]` section of your configuration file controls the general layout and typography of mdgreet.

## Greeting Message

You can customize the text displayed on the login card just above the user selection.

```toml
[appearance]
greeting = "Welcome to NixOS!"
```

## Opacity

Controls the transparency of the login card and the power menu. Accepts a float value between `0.0` (fully transparent) and `1.0` (fully opaque).

```toml
[appearance]
opacity = 0.75
```

## Typography

You can change the global font family used by the application, as well as specific settings for the large clock displayed on the screen.

```toml
[appearance]
# Uses the system's "Noto Sans" font for buttons, inputs, etc.
font_family = "Noto Sans"

[appearance.clock]
# Use a custom font just for the clock
font_family = "JetBrains Mono"
font_size = 200
font_weight = 600
```

> [!NOTE]
> If no font is specified, mdgreet uses its bundled font specifically for the clock. The rest of the interface will automatically fall back to your system's default sans-serif font.
