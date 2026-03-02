# camgylph

Real-time CLI camera renderer that converts webcam frames into colored ASCII.

Current stable release: `v1.3.1`

## Install

### Homebrew (recommended)

```bash
brew tap landxcape/tap
brew install landxcape/tap/camgylph
```

## Quick Start

1. List camera devices:

```bash
camgylph --list-devices
```

2. Run camera renderer:

```bash
camgylph --device "0:none"
```

On macOS, grant camera permission to your terminal app in:
`System Settings -> Privacy & Security -> Camera`.

## Typical Commands

```bash
camgylph --list-devices
camgylph --device "0:none"
camgylph --device /dev/video0
camgylph --device "Integrated Camera"
```

## Controls (while running)

- `q` or `Esc`: quit
- `c`: cycle color mode
- `r`: cycle character ramp
- `m`: toggle metrics details
- `h`: show/hide footer shortcuts line
- `v`: toggle horizontal mirror
- `+` / `-`: increase / decrease gamma
- `]` / `[`: increase / decrease contrast

## Common Options

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

## Rendering Behavior

- Full-terminal `cover` render mode
- Aspect ratio preserved with centered crop
- Footer shortcuts line shown by default (`h` toggles visibility)
- Metrics can be shown/hidden inside footer (`m`)

## Smoke Test Checklist

1. `camgylph --list-devices` prints device information.
2. `camgylph --device <valid-device>` renders ASCII frames.
3. `q` exits and terminal restores cleanly.
4. `c`, `r`, `m`, `h`, and `v` controls work during runtime.
5. Disconnecting camera does not panic and follows restart policy.
6. `+/-` and `[]` adjust output tone in real time.

## Build From Source (Contributor Mode)

Prerequisites:

- Rust toolchain (`cargo`, `rustc`)
- `ffmpeg` available in `PATH`

Build release binary:

```bash
cargo build --release
```

Run from build output:

```bash
target/release/camgylph --list-devices
target/release/camgylph --device "0:none"
```

## Known limitations

- Linux device enumeration via ffmpeg is format-focused for a chosen v4l2 node, not a full camera list.
- Renderer currently expects `rgb24` output path.

## Scope Tracking

- `docs/SCOPES.md`
- Completed: `docs/scopes/v1.3.0-interactive-tui.md`
- Next plan: `docs/scopes/v1.4.0-release-mode-plan.md`

## License

MIT. See `LICENSE`.
