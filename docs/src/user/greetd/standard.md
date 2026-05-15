# Standard Linux Configuration

If you are configuring `greetd` manually on a non-NixOS distribution (e.g., Arch Linux, Debian, Fedora), you will need to edit the configuration files manually.

## 1. Create mdgreet Configuration

Create the configuration file at `/etc/greetd/mdgreet.toml`:

```toml
[appearance]
greeting = "Welcome!"
theme.mode = "dark"
theme.seed_color = "#89b4fa" # Catppuccin Mocha Blue
```

*Ensure the `greeter` user has permission to read this file.*

---

## 2. Configure the Compositor and greetd

You need a Wayland compositor to run `mdgreet`. We strongly recommend [niri](https://github.com/niri-wm/niri) for its superior multi-monitor handling, but `cage` is a simpler alternative for single-monitor setups.

### Recommended: Using `niri` (Multi-monitor friendly)

`niri` allows you to strictly define the output monitor, force fullscreen mode, and disable keybindings so users cannot accidentally close the greeter.

**Step 1:** Install `niri` via your package manager.

**Step 2:** Create a special niri config file for the greeter, for example at `/etc/greetd/niri.kdl`:

```kdl
// Specify the target monitor (replace DP-1 with your actual monitor name).
// You can find monitor names by running `niri msg outputs` in a normal session.
output "DP-1" {
  focus-at-startup
  layout {
    // Set a background color that matches your theme
    background-color "#1e1e2e"
  }
}

// Disable all hotkeys so the user cannot close the greeter
hotkey-overlay { skip-at-startup; }

// Force the greeter to open in full screen
window-rule {
  match at-startup=true
  open-fullscreen true
}

// Launch mdgreet. When mdgreet exits (e.g., after login),
// tell niri to quit, which hands control back to greetd.
spawn-sh-at-startup "mdgreet ; niri msg action quit --skip-confirmation"
```

**Step 3:** Edit your `/etc/greetd/config.toml` to use `niri`:

```toml
[terminal]
vt = 1

[default_session]
# Launch niri with the specific config file we created
command = "niri --config /etc/greetd/niri.kdl"
user = "greeter"
```

---

### Alternative: Using `cage` (Single monitor)

If you have a single monitor, you can use `cage` (a simple kiosk compositor).

**Step 1:** Install `cage` via your package manager.

**Step 2:** Edit your `/etc/greetd/config.toml`:

```toml
[terminal]
vt = 1

[default_session]
# Launch cage. The -s flag is recommended when running under a display manager.
command = "cage -s -- mdgreet"
user = "greeter"
```
