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

## Fingerprint login

Use NixOS's standard fprintd/PAM support and enroll a finger for your account.
No mdgreet-specific NixOS or PAM module is needed.

```nix
services.fprintd.enable = true;
```

Inspect the effective chain before changing it:

```sh
cat /etc/pam.d/{greetd,login}
```

In the standard NixOS setup, greetd delegates authentication to `login`. Enable
fingerprint there if you previously disabled it, and set finite limits on the
existing rule:

```nix
security.pam.services.login.fprintAuth = true;
security.pam.services.login.rules.auth.fprintd.settings = {
  max-tries = 2;
  timeout = 10;
};
```

The timeout is in seconds per verification attempt; two failed attempts can
therefore use close to twenty seconds of scan time. This is not an account lockout.
These settings also affect services that include the `login` authentication
stack.
If greetd uses a different stack, configure the fingerprint rule in that stack
instead. Setting `greetd.fprintAuth` alone does not change a delegated `login`
stack, and adding a second fingerprint rule can cause repeated scan waits.
Preserve your existing password, account and session rules.

In the attribute set generating mdgreet's TOML configuration, add:

```nix
auth.fingerprint = {
  enable = true;
  auto_start = false; # Start with the fingerprint button
};
```

Use `auto_start = true` to start when the card opens. See
[Fingerprint login](../fingerprint.md) for the interaction and the bounded wait
before password verification. Build a boot generation and
reboot when ready; keep an alternate login and the previous generation available.
The greeter does not apply system configuration automatically.
