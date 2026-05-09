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

We use standard Cargo tests for non-UI business logic (like theme parsing).

```bash
cargo test
```
