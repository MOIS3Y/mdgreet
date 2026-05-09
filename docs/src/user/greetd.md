# Setup with greetd

`mdgreet` is designed to be run by `greetd`, a minimal and flexible login manager daemon.

To use `mdgreet`, you need to configure `greetd` to launch a Wayland compositor (like `cage` or `sway`), which in turn runs `mdgreet`.

> [!NOTE]
> The `mdgreet.toml` configuration file is entirely optional. If not provided, mdgreet will use sensible defaults. However, creating it is highly recommended to customize the appearance to your liking.

## NixOS Configuration

If you are using NixOS, configuring `greetd` to use `mdgreet` is straightforward. Here is an example configuration using `cage` as the Wayland compositor for the greeter:

```nix
{ config, pkgs, ... }:

let
  # Assuming mdgreet is available in your pkgs
  mdgreetPkg = pkgs.mdgreet;

  # Define your mdgreet configuration as a Nix attribute set
  mdgreetConfig = {
    appearance = {
      greeting = "Welcome to NixOS!";
      theme.mode = "dark";
      theme.name = "auto";
      background.blur = 15.0;
    };
  };
in
{
  # 1. Generate the TOML configuration file declaratively
  environment.etc."greetd/mdgreet.toml".source =
    (pkgs.formats.toml {}).generate "mdgreet.toml" mdgreetConfig;

  # 2. Make sure greetd has the necessary directories
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

  # 3. Configure the greetd service
  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        command = "${pkgs.cage}/bin/cage -s -- ${mdgreetPkg}/bin/mdgreet";
        user = "greeter";
      };
    };
  };

  environment.systemPackages = with pkgs; [ cage dbus ];
}
```

## Standard Linux Configuration

If you are configuring `greetd` manually on a non-NixOS distribution:

### 1. Configure greetd

Edit your `/etc/greetd/config.toml`:

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

### 2. Create mdgreet Configuration

Create the configuration file at `/etc/greetd/mdgreet.toml`:

```toml
[appearance]
greeting = "Welcome!"
```

Make sure the `greeter` user has permission to read this file.
