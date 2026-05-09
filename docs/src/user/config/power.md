# Power Management

The `[power]` section allows you to override the default commands executed when a user interacts with the power menu on the login screen.

By default, mdgreet uses standard `systemd` (specifically `systemctl`) commands, which work out-of-the-box on most modern Linux distributions like NixOS, Arch, Fedora, and Ubuntu.

```toml
[power]
shutdown = "systemctl poweroff"
reboot = "systemctl reboot"
sleep = "systemctl suspend"
hibernate = "systemctl hibernate"
```

## Non-systemd Distributions

If you are using a distribution that does not use `systemd` as its init system (such as Void Linux or Artix), you will need to override these commands to match your system's power management utilities (e.g., `loginctl`, `zzz`, or direct `shutdown` commands).

For example, on a system using `elogind`:

```toml
[power]
shutdown = "loginctl poweroff"
reboot = "loginctl reboot"
sleep = "loginctl suspend"
hibernate = "loginctl hibernate"
```

> [!NOTE]
> Ensure that the `greeter` user has the necessary permissions to execute these commands without a password prompt.
