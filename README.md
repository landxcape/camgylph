# camgylph

Real-time CLI camera renderer that converts webcam frames into colored ASCII.

Current stable milestone: `v1.3.0`

Scope tracking:

- `docs/SCOPES.md`
- Completed: `docs/scopes/v1.3.0-interactive-tui.md`
- Next plan: `docs/scopes/v1.4.0-release-mode-plan.md`

## Prerequisites

- Rust toolchain (`cargo`, `rustc`)
- `ffmpeg` available in `PATH`

## Platform setup

### macOS

1. Install dependencies:

```bash
brew install ffmpeg rust
```

2. Grant camera permission to your terminal app in:
`System Settings -> Privacy & Security -> Camera`

3. Build release binary:

```bash
cargo build --release
```

4. List devices:

```bash
target/release/camgylph --list-devices
```

### Linux

1. Install dependencies (example for Debian/Ubuntu):

```bash
sudo apt-get update
sudo apt-get install -y ffmpeg build-essential pkg-config libudev-dev
```

2. Ensure your user can read camera device (usually `/dev/video0`).

3. Build release binary:

```bash
cargo build --release
```

4. Optional device format listing:

```bash
target/release/camgylph --list-devices --device /dev/video0
```

### Windows

1. Install Rust and FFmpeg (add FFmpeg `bin` to `PATH`).
2. Build release binary:

```powershell
cargo build --release
```

3. Use the camera name from the device list:

```powershell
target\release\camgylph.exe --list-devices
```

4. Run with explicit device:

```powershell
target\release\camgylph.exe --device "Integrated Camera"
```

## Run

Default run:

```bash
target/release/camgylph --device "0:none"
```

Development run (debug build):

```bash
cargo run -- --device "0:none"
```

Rendering mode:

- Full-terminal `cover` (fills available terminal area)
- Aspect ratio preserved with centered crop
- Footer shortcuts are shown by default; `h` toggles the footer line
- `m` toggles metrics details inside the footer

Common options:

- `--width <n>` / `--height <n>`
- `--fps <n>`
- `--ramp standard|detailed`
- `--truecolor` / `--ansi256` / `--no-color`
- `--show-metrics` / `--hide-metrics`
- `--gamma <value>` / `--contrast <value>`
- `--render-fps <n>` (`0` = follow camera FPS)
- `--log-metrics-ms <n>`
- `--max-cols <n>` / `--max-rows <n>`
- `--fast` (performance preset)
- `--max-failures <n>` / `--backoff-ms <n>`

## Controls

- `q` or `Esc`: quit
- `c`: cycle color mode
- `r`: cycle character ramp
- `m`: toggle metrics details
- `h`: show/hide footer shortcuts line
- `v`: toggle horizontal mirror
- `+` / `-`: increase / decrease gamma
- `]` / `[`: increase / decrease contrast

## Release build

```bash
cargo build --release
```

Binary path:

- macOS/Linux: `target/release/camgylph`
- Windows: `target\release\camgylph.exe`

## Smoke-test checklist

1. `target/release/camgylph --list-devices` prints device information.
2. `target/release/camgylph --device <valid-device>` renders ASCII frames.
3. `q` exits and terminal restores cleanly.
4. `c`, `r`, `m`, `h`, and `v` controls work during runtime.
5. Disconnecting camera does not panic and follows restart policy.
6. `+/-` and `[]` adjust output tone in real time.

## Known limitations

- Linux device enumeration via ffmpeg is format-focused for a chosen v4l2 node, not a full camera list.
- Renderer currently expects `rgb24` output path.

## License

MIT. See `LICENSE`.
