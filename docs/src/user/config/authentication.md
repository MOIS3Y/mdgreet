# Authentication configuration

The `[auth.fingerprint]` section controls the fingerprint card and when mdgreet
starts authentication. The actual authentication methods and their limits are
configured in the system's greetd/PAM stack.

```toml
[auth.fingerprint]
enable = true
auto_start = false
```

| Option | Type | Default | Behaviour |
| --- | --- | --- | --- |
| `enable` | Boolean | `false` | Show or hide the fingerprint sign-in card and its start button. This does not enable or disable `pam_fprintd`. |
| `auto_start` | Boolean | `true` | Start authentication when the card opens or the selected account changes. Set to `false` to wait for an explicit action. |

Omitting the section or setting `enable = false` keeps the password-entry
interface. If greetd's PAM stack checks fingerprints before passwords, password
verification still waits for that check to finish. Setting `auto_start` has no
effect when `enable = false`.

## Automatic start

```toml
[auth.fingerprint]
enable = true
auto_start = true
```

Startup shows the clock without an active authentication attempt. Opening the
card starts authentication for the selected account. Selecting another account
cancels the previous attempt and starts one for the new account. During an active
attempt, pressing the fingerprint button shows an inline hint.

## Explicit start

```toml
[auth.fingerprint]
enable = true
auto_start = false
```

Opening the card and selecting an account leave authentication idle. Press the
fingerprint button to start. After returning to the clock and opening the card
again, another explicit action is required. Confirming a manually entered
username with **Enter** also starts authentication.

Selecting **Use password** opens and focuses the password field. It also starts
a PAM conversation if none is active. If your PAM stack checks fingerprints
first, that check still precedes password verification.

## System authentication policy

These options do not enable or disable PAM modules, change scan limits or enforce
a password-only security policy. In particular:

- `enable = false` selects the password-entry interface; PAM still decides which
  methods can authenticate the user.
- `auto_start = false` delays the request; it does not discard a fingerprint
  captured earlier by a power-button reader.
- **Use password** allows immediate input but does not interrupt an active scan.
  Verification can wait for PAM's finite fingerprint timeout.

Set `max-tries` and `timeout` in the existing `pam_fprintd` rule, not in this TOML
section. See [Fingerprint login](../fingerprint.md) for the sign-in flow and
[standard Linux](../greetd/standard.md#fingerprint-login) or
[NixOS](../greetd/nixos.md#fingerprint-login) for system setup.
