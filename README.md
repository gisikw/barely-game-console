# barely-game-console

RFID-powered retro game console launcher for NixOS kiosk displays.

Tap an RFID card to launch a game. Each card maps to a ROM + emulator core (via RetroArch) or an arbitrary command. A physical power button returns to the menu.

## Features

- RFID card detection via USB HID reader (evdev)
- Power button listener for returning to the launcher menu
- ROM preview UI with artwork display
- RetroArch integration for emulation
- Generic command support for non-emulator apps
- Automatic evdev recovery after child process exit

## Configuration

Games and commands are configured in `config.toml`:

```toml
[rfid_cards."0001234567"]
rom_path = "/opt/roms/snes/Game.zip"
emulator = "/path/to/libretro/core.so"
artwork = "assets/game-art.jpg"

[rfid_cards."0009876543"]
command = ["/usr/bin/some-app", "--fullscreen"]
working_dir = "/opt/apps"
artwork = "assets/app-art.png"
```

## Building

```bash
nix develop     # enter dev shell
just build      # iterative build
just check      # fmt + test
just ship       # commit, push — CI handles deployment
```

## Deployment

Builds are deployed as a Nix overlay via Forgejo CI. The binary lands at `/run/overlays/bin/barely-game-console` on subscribed hosts.
