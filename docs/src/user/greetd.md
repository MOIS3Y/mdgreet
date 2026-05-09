# Setup with greetd

`mdgreet` is designed to be run by `greetd`, a minimal and flexible login manager daemon.

To use `mdgreet`, you need to configure `greetd` to launch a Wayland compositor (like `cage` or `sway`), which in turn runs `mdgreet`.

## NixOS Configuration

If you are using NixOS, configuring `greetd` to use `mdgreet` is straightforward. Here is an example configuration using `cage` as the Wayland compositor for the greeter:

```nix
{ config, pkgs, ... }:

let
  # Assuming mdgreet is available in your pkgs, e.g., via an overlay or flake input
  mdgreetPkg = pkgs.mdgreet; 
in
{
  # Make sure greetd has the necessary directories
  systemd.tmpfiles.settings."10-mdgreet" = {
    "/var/cache/mdgreet".d = {
      mode = "0755";
      user = "greeter";
      group = "greeter";
    };
    "/var/log/mdgreet".d = {
      mode = "0755";
      user = "greeter";
      group = "greeter";
    };
  };

  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        # Launch cage, which then launches mdgreet
        command = "${pkgs.cage}/bin/cage -s -- ${mdgreetPkg}/bin/mdgreet";
        user = "greeter";
      };
    };
  };

  # Required for greetd and Wayland
  environment.systemPackages = with pkgs; [ cage dbus ];
}
```

## Standard Linux Configuration

If you are configuring `greetd` manually on a non-NixOS distribution, edit your `/etc/greetd/config.toml`:

```toml
[terminal]
# The VT to run the greeter on. Can be "next", "current" or a number
# designating the VT.
vt = 1

[default_session]
# The command to start. Launch a Wayland compositor like cage running mdgreet.
command = "cage -s -- mdgreet"

# The user to run the command as. The privileges this user must have depends
# on the greeter. A graphical greeter may for example require the user to be
# in the `video` group.
user = "greeter"
```

Make sure the `greeter` user has permission to access the display server and read the `mdgreet` configuration file (usually placed at `/etc/greetd/mdgreet.toml`).
