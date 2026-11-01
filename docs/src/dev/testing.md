# Testing & VM Integration

Testing a greeter locally can be tricky since it requires a Wayland compositor and the `greetd` daemon running as root. 

mdgreet solves this by providing a fully configured, containerized NixOS Virtual Machine.

## Running the Test VM

If you have Nix installed, you can spin up the test VM directly from the flake. This VM includes dummy users, a compositor (`cage`), and is configured to run your local build of mdgreet.

```bash
nix run .#vm
```

This command will:
1. Compile mdgreet.
2. Build a minimal NixOS qcow2 image.
3. Launch QEMU with graphical support.

You can log in using any of the test users (e.g., `alice`, `bob`) with the password `password`.

## Running Unit Tests

Cargo tests cover business logic, framed greetd IPC and a headless Slint login
controller.

```bash
cargo test
```

## Fingerprint demo

Demo mode uses a local, simulated greetd peer. It does not contact PAM or fprintd,
start a user session or persist a successful login. Create `fingerprint-demo.toml`:

```toml
[auth.fingerprint]
enable = true
auto_start = true
```

Run the default scenario:

```sh
cargo run -- --demo --config fingerprint-demo.toml
```

Set `auto_start = false` to check the start button. The available scenarios are:

| `MDGREET_DEMO_AUTH` | Simulated behaviour |
| --- | --- |
| Unset or `fallback` | Wait one second, then request a password. Any dummy response succeeds. |
| `fingerprint` | Authenticate after one second without requesting a password. |
| `mismatch` | Report two unmatched scans, then request a password. |
| `error` | Request a password after the scan wait; reject `incorrect-demo` and accept another dummy response. |

For example:

```sh
MDGREET_DEMO_AUTH=mismatch cargo run -- --demo --config fingerprint-demo.toml
```

A submitted password waits for the simulated PAM prompt in the same conversation.
Fingerprint success can complete the conversation before that prompt, leaving
the queued password unused. Demo timing and results do not establish hardware
behaviour or the installed system's scan/lockout policy. Use dummy responses.

## Automated authentication checks

The framed IPC tests cover Info/Error acknowledgement, queued input, matching
secret and visible prompts, fresh input for repeated requests, success, rejection,
disconnection, partial frames and bounded cancellation. They also check that
cleanup failures preserve the original credential error and that a successful
fingerprint does not send an unused password.

The headless Slint tests cover:

- Clock-first startup and automatic or explicit fingerprint start.
- No implicit start on account/card changes when `auto_start = false`.
- Password focus on opening, account changes, retries and return from the clock.
- Manual username confirmation and preservation of focus in the user picker.
- Password submission during scanning, duplicate submission prevention and retry
  after rejection, including a terminal task whose channel is still open.
- Cancellation on leaving the card or changing accounts, and a fixed primary
  button position across modes and inline errors.

Configuration tests check defaults and the fingerprint options table.

To render the actual Slint layouts with dummy accounts and responses:

```sh
MDGREET_TEST_RENDER_DIR=/tmp/mdgreet-preview cargo test app::login::tests
```

The output includes `fingerprint.png`, `fingerprint-hint.png`,
`fingerprint-error.png`, `password.png` and `password-error.png`. These are
software-rendered previews, not recordings of hardware authentication.

## Manual fingerprint verification

Use an enrolled test account and a working password fallback. Inspect the PAM
stack used by greetd, including its `include`/`substack` dependencies. Configure
finite scan limits and timeouts as described in the
[system setup guide](../user/fingerprint.md#before-you-start).

Check each configuration independently:

| mdgreet configuration | Expected initial behaviour after the clock |
| --- | --- |
| `enable = false` | Password-entry interface; PAM policy still owns the methods. |
| `enable = true`, `auto_start = true` | Start authentication for the selected account. |
| `enable = true`, `auto_start = false` | Wait for the fingerprint button or another explicit authentication action. |

Verify correct and incorrect fingerprints, timeout without touching the reader,
password submission before and after the password prompt, and an incorrect
password followed by a correct one. In the sequential stack, submitted passwords
can wait for fingerprint timeout, and a new conversation can repeat that wait.
A correct password must not require touching the reader to advance the stack.

Return to the clock with **Esc**, change users or sessions during a pending
attempt, and confirm that the old conversation is cleaned up before another
starts. Check that the reader remains usable and the UI stays responsive during
system delays. In explicit mode, verify that simply reopening the card or
selecting an account does not start another attempt.

If the reader is integrated into the power button, distinguish a new touch at
the card from a fingerprint captured when powering on. Compare startup with an
enrolled and an unenrolled finger; explicit start does not guarantee a fresh
hardware sample.

Missing devices, missing enrollment and unavailable services can be simulated
without deleting enrolled fingers. Check real hardware separately when practical.
Record configuration, observed messages, scanner/indicator behaviour and timing;
keep automatic results separate from hardware results. Maintain an alternate
login and a way to restore the previous configuration during interactive tests.
Do not restart greetd or terminate an active user session as part of unit tests.

## System policy

`pam_fprintd` limits attempts within a PAM conversation. Its `timeout` is per
verification attempt: no touch times out the first attempt, while an unmatched
scan can start another attempt and timeout. This does not provide a persistent
biometric lockout across new conversations.

Password delay and account lockout must be tested against the installed system
policy. A UI counter or timer is not a security boundary. Ensure fingerprint
short-circuit rules do not bypass required policy; see the
[Linux-PAM faillock documentation](https://github.com/linux-pam/linux-pam/tree/master/modules/pam_faillock).

Disabling the fingerprint interface is not proof that PAM requires a password.
Likewise, greetd's generic Success response does not identify which module
accepted authentication. Do not infer the method from UI state, a queued password
or translated PAM message text.
