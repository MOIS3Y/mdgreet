# Background Configuration

The background of your greeter is controlled under the `[appearance.background]` block.

## Wallpaper

You can set an image to be used as the background. mdgreet supports standard formats like JPEG and PNG.

```toml
[appearance.background]
path = "/usr/share/backgrounds/landscape.jpg"
```

## Blur Effect

To ensure the login card and clock remain legible, mdgreet can apply a Gaussian blur to your wallpaper.

```toml
[appearance.background]
# Sigma value for the Gaussian blur. 
# Set to 0.0 to disable the blur entirely.
blur = 10.0
```

> [!TIP]
> **Performance Note:** When a blur value is set, mdgreet calculates the blur on the first launch and caches the resulting image to disk. While values up to `10.0` process almost instantly (even on 4K images), setting excessively high values might cause a slight delay during the very first boot. Subsequent logins will load instantly from the cache.

## Fallback Color

If the image path is missing or the file cannot be loaded, mdgreet will fall back to a solid color. You can define this explicitly, or omit it to let the current theme decide the best background color.

```toml
[appearance.background]
color = "#11111b"
```
