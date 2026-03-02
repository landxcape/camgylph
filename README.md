# camgylph

Real-time CLI camera renderer that converts webcam frames into colored ASCII.

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

3. List devices:

```bash
cargo run -- --list-devices
```

### Linux

1. Install dependencies (example for Debian/Ubuntu):

```bash
sudo apt-get update
sudo apt-get install -y ffmpeg build-essential pkg-config libudev-dev
```

2. Ensure your user can read camera device (usually `/dev/video0`).

3. Optional device format listing:

```bash
cargo run -- --list-devices --device /dev/video0
```

### Windows

1. Install Rust and FFmpeg (add FFmpeg `bin` to `PATH`).
2. Use the camera name from the device list:

```powershell
cargo run -- --list-devices
```

3. Run with explicit device:

```powershell
cargo run -- --device "Integrated Camera"
```

## Run

Default run:

```bash
cargo run -- --device "0:none"
```

Common options:

- `--width <n>` / `--height <n>`
- `--fps <n>`
- `--ramp standard|detailed`
- `--truecolor` / `--ansi256` / `--no-color`
- `--show-metrics` / `--hide-metrics`
- `--gamma <value>` / `--contrast <value>`
- `--render-fps <n>`
- `--log-metrics-ms <n>`
- `--max-failures <n>` / `--backoff-ms <n>`
- `--cell-aspect <ratio>`

## Controls

- `q` or `Esc`: quit
- `c`: cycle color mode
- `r`: cycle character ramp
- `m`: toggle metrics line
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

1. `cargo run -- --list-devices` prints device information.
2. `cargo run -- --device <valid-device>` renders ASCII frames.
3. `q` exits and terminal restores cleanly.
4. `c`, `r`, and `m` controls work during runtime.
5. Disconnecting camera does not panic and follows restart policy.
6. `+/-` and `[]` adjust output tone in real time.

## Known limitations

- Linux device enumeration via ffmpeg is format-focused for a chosen v4l2 node, not a full camera list.
- Renderer currently expects `rgb24` output path.
