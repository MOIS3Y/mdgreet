# Architecture Overview

`mdgreet` separates UI layout (Slint) from business logic (Rust) while using Tokio for asynchronous operations like inter-process communication (IPC) with `greetd`.

## Module Structure

The project is structured into clear responsibilities:

- **`src/main.rs`**: Acts as the pure orchestrator. It parses CLI arguments, initializes logging, sets up the Slint UI instance, and delegates specific features to the `app::` modules.
- **`src/app/`**: Contains the core UI logic and event handlers.
  - **`appearance.rs`**: Resolves themes (builtin, dynamic, custom), parses colors, handles background blur, and configures fonts.
  - **`auth.rs`**: Manages the list of valid system users.
  - **`login.rs`**: Registers authentication callbacks and owns the current attempt.
    `login/attempt.rs` owns the connection and session startup; `login/view.rs`
    applies generation-checked UI updates.
  - **`session.rs`**: Discovers available Wayland/X11 compositors on the system.
  - **`state.rs`**: Manages UI state persistence, such as remembering the last selected user and their preferred compositor.
  - **`power.rs`**: Executes system commands for shutdown, reboot, etc.
- **`src/utils/`**: Helper utilities.
  - **`client.rs`**: The `GreetdClient` for Unix socket IPC. `client/conversation.rs`
    schedules standard greetd requests and PAM replies; `client/demo.rs` provides
    an isolated peer for demos and tests.
  - **`cache.rs`**: Simple LRU-style disk caching for UI state.

## Concurrency Model

Slint runs its own event loop on the main thread. We use `tokio::spawn` within UI callbacks to run blocking or network tasks asynchronously.

When an async task needs to update the UI (e.g., showing an error message after a failed login), we use `ui_weak.upgrade_in_event_loop(...)` to safely push the update back to the main UI thread without causing deadlocks.

## Authentication lifecycle

The clock screen has no active authentication attempt. With fingerprint enabled,
opening the card starts one only when `auto_start = true`; explicit mode waits
for a user request. The UI setting does not select a method inside PAM.

Each attempt owns its input and cancellation channels and snapshots the account
and selected session. A submitted password can wait in memory while a sequential
PAM stack checks a fingerprint. It is consumed once by a matching Secret prompt;
Info/Error messages are acknowledged without sending credentials. Repeated
prompts require fresh input. Authentication succeeds only on greetd Success,
and session startup must also be acknowledged before the successful selection
is persisted. The cache stores account/session preferences, not credentials or
authorization results.

Cancellation invalidates the UI generation so stale events cannot update the
card or authorize session startup. Tasks share a gate held through connection
cleanup, keeping consecutive conversations from overlapping. A cancelled pending
read preserves any partial frame while waiting for a bounded protocol boundary.
PAM's own finite fingerprint timeout remains necessary for predictable cleanup.
A terminal attempt's still-open input channel cannot consume a new retry.
