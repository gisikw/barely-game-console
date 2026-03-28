# INVARIANTS — barely-game-console

Violations of these contracts are bugs.

## Runtime Environment

- **Wayland-only.** Runs inside cage (a Wayland kiosk compositor) via greetd. No X11 support needed or tested.
- **Config at runtime.** `config.toml` is loaded from the working directory at startup — not baked into the binary.
- **Device discovery by name.** RFID reader (`HID 413d:2107`) and power button (`Power Button`) are discovered by device name via evdev, not by hardcoded `/dev/input/eventN` paths.

## Process Lifecycle

- **Evdev readers must survive child exit.** RetroArch (or any launched command) may disrupt evdev device state. Readers must recover after the child process exits rather than dying silently. See `6323fd8`.
- **Child processes are reaped.** The launcher is responsible for waiting on spawned processes and returning to the menu state on exit.

## Deployment

- **Overlay-delivered.** The binary is built by Forgejo CI, cached in Attic, and registered with the overlay-registry. Hosts subscribe to the overlay in their fort-nix manifest.
- **No pinned derivation in fort-nix.** The binary is not vendored or pinned in fort-nix. The overlay system handles versioning.
- **No automatic service restart.** Updating the overlay binary does not restart greetd or the kiosk session. A manual restart or reboot is required to pick up new versions.
