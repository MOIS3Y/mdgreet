# NixOS Configuration

If you are using NixOS, configuring `greetd` to use `mdgreet` is straightforward. You can define everything declaratively in your configuration files.

## Recommended: Using `niri` (Multi-monitor friendly)

While `cage` is a common choice for greeters, it has limitations with multi-monitor setups (e.g., determining which exact monitor displays the greeter). 

Using [niri](https://github.com/niri-wm/niri) is highly recommended. It allows you to strictly define the output monitor, force fullscreen mode, and disable keybindings so users cannot accidentally close the greeter or interact with the compositor.

Here is an example configuration using `niri`:

```nix
{ config, pkgs, lib, ... }:

let
  mdgreetPkg = pkgs.mdgreet;
  
  # 1. Define your mdgreet configuration
  mdgreetConfig = {
    appearance = {
      greeting = "Welcome to NixOS!";
      theme.mode = "dark";
      theme.seed_color = "#89b4fa"; # Catppuccin Mocha Blue
    };
  };

  # 2. Define the niri configuration specifically for the greeter
  niriConfig = pkgs.writeText "niri-greet.kdl" ''
    // Specify the target monitor (replace DP-1 with your actual monitor name)
    // You can find monitor names by running `niri msg outputs` in a normal session.
    output "DP-1" {
      focus-at-startup
      layout {
        // Match the background color to your mdgreet theme to avoid flashes
        background-color "#1e1e2e"
      }
    }

    // Disable all hotkeys so the user cannot close the greeter or interact with niri
    hotkey-overlay { skip-at-startup; }

    window-rule {
      match at-startup=true
      open-fullscreen true
    }

    // Launch mdgreet and exit niri when mdgreet is done
    spawn-sh-at-startup "${lib.getExe mdgreetPkg} ; ${lib.getExe pkgs.niri} msg action quit --skip-confirmation"
  '';
in
{
  # Generate the TOML configuration file
  environment.etc."greetd/mdgreet.toml".source =
    (pkgs.formats.toml {}).generate "mdgreet.toml" mdgreetConfig;

  # Make sure greetd has the necessary directories
  systemd.tmpfiles.settings."10-mdgreet" = {
    "/var/cache/mdgreet".d = { mode = "0755"; user = "greeter"; group = "greeter"; };
    "/var/log/mdgreet".d =   { mode = "0755"; user = "greeter"; group = "greeter"; };
  };

  # Configure the greetd service
  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        # Launch niri with our custom config
        command = "${lib.getExe pkgs.niri} --config ${niriConfig}";
        user = "greeter";
      };
    };
  };
  
  # Ensure necessary packages are available
  environment.systemPackages = with pkgs; [ niri dbus ];
}
```

## Alternative: Using `cage` (Single monitor)

If you have a single monitor or don't mind the greeter appearing on the default screen, you can use `cage`. It's a kiosk compositor designed to run a single full-screen application.

```nix
{ config, pkgs, ... }:

let
  mdgreetPkg = pkgs.mdgreet;
  mdgreetConfig = {
    appearance = { greeting = "Welcome!"; }
  };
in
{
  environment.etc."greetd/mdgreet.toml".source =
    (pkgs.formats.toml {}).generate "mdgreet.toml" mdgreetConfig;

  systemd.tmpfiles.settings."10-mdgreet" = {
    "/var/cache/mdgreet".d = { mode = "0755"; user = "greeter"; group = "greeter"; };
    "/var/log/mdgreet".d =   { mode = "0755"; user = "greeter"; group = "greeter"; };
  };

  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        # -s ensures it runs smoothly via systemd
        command = "${pkgs.cage}/bin/cage -s -- ${mdgreetPkg}/bin/mdgreet";
        user = "greeter";
      };
    };
  };

  environment.systemPackages = with pkgs; [ cage dbus ];
}
```
