# Theming

mdgreet features a powerful theming engine based on Material Design 3. It can generate full color palettes dynamically or use predefined themes.

All theme settings go under the `[appearance.theme]` block.

## Mode

Themes can run in either light or dark mode.

```toml
[appearance.theme]
mode = "dark" # or "light"
```

## Theme Types

The `name` property defines how the theme is generated.

### 1. Built-in Themes

Use standard, predefined color schemes. Available options are `"default"` and `"slint"`.

```toml
[appearance.theme]
name = "default"
```

### 2. Auto (Material You)

Generates a theme dynamically by extracting the dominant colors from your current background image. This provides a deeply integrated, personalized look.

```toml
[appearance.theme]
name = "auto"

[appearance.background]
path = "/path/to/my/wallpaper.jpg"
```

### 3. Seed Color

If you want a specific brand color without relying on a wallpaper, use the `"seed"` theme and provide a HEX color.

```toml
[appearance.theme]
name = "seed"
seed_color = "#1e66f5"
```

### 4. Custom JSON

For total control over every Material Design color token, you can provide a custom JSON file.

```toml
[appearance.theme]
name = "custom"
path = "/etc/greetd/my-mdgreet-theme.json"
```

> **Note:** The JSON file must follow the structure expected by mdgreet's internal `MaterialScheme` struct.
