# Architecture Overview

`mdgreet` separates UI layout (Slint) from business logic (Rust) while using Tokio for asynchronous operations like inter-process communication (IPC) with `greetd`.

## Module Structure

The project is structured into clear responsibilities:

- **`src/main.rs`**: Acts as the pure orchestrator. It parses CLI arguments, initializes logging, sets up the Slint UI instance, and delegates specific features to the `app::` modules.
- **`src/app/`**: Contains the core UI logic and event handlers.
  - **`appearance.rs`**: Resolves themes (builtin, dynamic, custom), parses colors, handles background blur, and configures fonts.
  - **`auth.rs`**: Manages the list of valid system users.
  - **`login.rs`**: Orchestrates the authentication flow. It handles the `on_login` callback, bridging the synchronous Slint UI with asynchronous Tokio tasks that talk to `greetd`.
  - **`session.rs`**: Discovers available Wayland/X11 compositors on the system.
  - **`state.rs`**: Manages UI state persistence, such as remembering the last selected user and their preferred compositor.
  - **`power.rs`**: Executes system commands for shutdown, reboot, etc.
- **`src/utils/`**: Helper utilities.
  - **`client.rs`**: The `GreetdClient` which handles Unix socket IPC using the `greetd_ipc` protocol.
  - **`cache.rs`**: Simple LRU-style disk caching for UI state.

## Concurrency Model

Slint runs its own event loop on the main thread. We use `tokio::spawn` within UI callbacks to run blocking or network tasks asynchronously.

When an async task needs to update the UI (e.g., showing an error message after a failed login), we use `ui_weak.upgrade_in_event_loop(...)` to safely push the update back to the main UI thread without causing deadlocks.
