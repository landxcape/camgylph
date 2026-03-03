# camgylph Usage Guide

## Install

### Homebrew

```bash
brew tap landxcape/tap
brew install landxcape/tap/camgylph
```

## Quick Start

1. List camera devices:

```bash
camgylph --list-devices
```

2. Start camera view:

```bash
camgylph --device "0:none"
```

On macOS, allow camera access for your terminal app in:
`System Settings -> Privacy & Security -> Camera`

## Runtime Controls

- `q` or `Esc`: quit
- `c`: cycle color mode
- `r`: cycle character ramp
- `m`: toggle metrics details
- `h`: show/hide shortcuts footer
- `v`: toggle horizontal mirror
- `+` / `-`: increase / decrease gamma
- `]` / `[`: increase / decrease contrast

## Useful Options

- `--device <value>`
- `--width <n>` / `--height <n>`
- `--fps <n>`
- `--truecolor` / `--ansi256` / `--no-color`
- `--ramp standard|detailed`
- `--release-mode`

## Common Issues

### `--list-devices` fails on macOS

Run with:

```bash
camgylph --list-devices
```

If no camera is shown, verify permission and close apps that may lock the camera.

### Camera mode not supported

If you see an unsupported size/fps error, use a supported mode for your device and retry with `--width`, `--height`, and `--fps`.

### Slow rendering

Try release mode:

```bash
camgylph --device "0:none" --release-mode
```

If needed, lower render pressure with smaller capture resolution.
