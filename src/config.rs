use crate::{ascii::color::ColorMode, error::AppError};

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub pix_fmt: String,
    pub device: Option<String>,
    pub list_devices: bool,
    pub color_mode: ColorMode,
    pub ramp_name: String,
    pub show_metrics: bool,
    pub gamma: f32,
    pub contrast: f32,
    pub render_fps: u32,
    pub log_metrics_ms: u64,
    pub max_cols: u16,
    pub max_rows: u16,
    pub max_consecutive_failures: u32,
    pub backoff_base_ms: u64,
    pub release_mode: bool,
    pub show_help: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            fps: 30,
            pix_fmt: "rgb24".to_string(),
            device: std::env::var("CAMGLYPH_DEVICE").ok(),
            list_devices: false,
            color_mode: ColorMode::Ansi256,
            ramp_name: "standard".to_string(),
            show_metrics: false,
            gamma: 1.0,
            contrast: 1.0,
            render_fps: 30,
            log_metrics_ms: 0,
            max_cols: 0,
            max_rows: 0,
            max_consecutive_failures: 5,
            backoff_base_ms: 500,
            release_mode: false,
            show_help: false,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_args(std::env::args().skip(1))
    }

    fn from_args<I>(args: I) -> Result<Self, AppError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut cfg = Self::default();
        let mut args = args.into_iter();
        let mut use_fast_preset = false;
        let mut use_release_mode = false;
        let mut explicit = ExplicitOverrides::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => cfg.show_help = true,
                "--list-devices" => cfg.list_devices = true,
                "--device" => cfg.device = Some(next_value(&mut args, "--device")?),
                "--width" => cfg.width = parse_u32(next_value(&mut args, "--width")?, "--width")?,
                "--height" => {
                    cfg.height = parse_u32(next_value(&mut args, "--height")?, "--height")?
                }
                "--fps" => cfg.fps = parse_u32(next_value(&mut args, "--fps")?, "--fps")?,
                "--pix-fmt" => cfg.pix_fmt = next_value(&mut args, "--pix-fmt")?,
                "--ramp" => {
                    cfg.ramp_name = next_value(&mut args, "--ramp")?;
                    explicit.ramp_name = Some(cfg.ramp_name.clone());
                }
                "--show-metrics" => {
                    cfg.show_metrics = true;
                    explicit.show_metrics = Some(true);
                }
                "--hide-metrics" => {
                    cfg.show_metrics = false;
                    explicit.show_metrics = Some(false);
                }
                "--fast" => use_fast_preset = true,
                "--release-mode" => use_release_mode = true,
                "--gamma" => cfg.gamma = parse_f32(next_value(&mut args, "--gamma")?, "--gamma")?,
                "--contrast" => {
                    cfg.contrast = parse_f32(next_value(&mut args, "--contrast")?, "--contrast")?
                }
                "--render-fps" => {
                    cfg.render_fps =
                        parse_u32(next_value(&mut args, "--render-fps")?, "--render-fps")?;
                    explicit.render_fps = Some(cfg.render_fps);
                }
                "--log-metrics-ms" => {
                    cfg.log_metrics_ms = parse_u64(
                        next_value(&mut args, "--log-metrics-ms")?,
                        "--log-metrics-ms",
                    )?
                }
                "--max-cols" => {
                    cfg.max_cols = parse_u16(next_value(&mut args, "--max-cols")?, "--max-cols")?;
                    explicit.max_cols = Some(cfg.max_cols);
                }
                "--max-rows" => {
                    cfg.max_rows = parse_u16(next_value(&mut args, "--max-rows")?, "--max-rows")?;
                    explicit.max_rows = Some(cfg.max_rows);
                }
                "--max-failures" => {
                    cfg.max_consecutive_failures =
                        parse_u32(next_value(&mut args, "--max-failures")?, "--max-failures")?
                }
                "--backoff-ms" => {
                    cfg.backoff_base_ms =
                        parse_u64(next_value(&mut args, "--backoff-ms")?, "--backoff-ms")?
                }
                "--no-color" => {
                    cfg.color_mode = ColorMode::None;
                    explicit.color_mode = Some(cfg.color_mode);
                }
                "--ansi256" => {
                    cfg.color_mode = ColorMode::Ansi256;
                    explicit.color_mode = Some(cfg.color_mode);
                }
                "--truecolor" => {
                    cfg.color_mode = ColorMode::Truecolor;
                    explicit.color_mode = Some(cfg.color_mode);
                }
                _ => {
                    if let Some(value) = arg.strip_prefix("--device=") {
                        cfg.device = Some(value.to_string());
                    } else if let Some(value) = arg.strip_prefix("--width=") {
                        cfg.width = parse_u32(value.to_string(), "--width")?;
                    } else if let Some(value) = arg.strip_prefix("--height=") {
                        cfg.height = parse_u32(value.to_string(), "--height")?;
                    } else if let Some(value) = arg.strip_prefix("--fps=") {
                        cfg.fps = parse_u32(value.to_string(), "--fps")?;
                    } else if let Some(value) = arg.strip_prefix("--pix-fmt=") {
                        cfg.pix_fmt = value.to_string();
                    } else if let Some(value) = arg.strip_prefix("--ramp=") {
                        cfg.ramp_name = value.to_string();
                        explicit.ramp_name = Some(cfg.ramp_name.clone());
                    } else if let Some(value) = arg.strip_prefix("--gamma=") {
                        cfg.gamma = parse_f32(value.to_string(), "--gamma")?;
                    } else if let Some(value) = arg.strip_prefix("--contrast=") {
                        cfg.contrast = parse_f32(value.to_string(), "--contrast")?;
                    } else if let Some(value) = arg.strip_prefix("--render-fps=") {
                        cfg.render_fps = parse_u32(value.to_string(), "--render-fps")?;
                        explicit.render_fps = Some(cfg.render_fps);
                    } else if let Some(value) = arg.strip_prefix("--log-metrics-ms=") {
                        cfg.log_metrics_ms = parse_u64(value.to_string(), "--log-metrics-ms")?;
                    } else if let Some(value) = arg.strip_prefix("--max-cols=") {
                        cfg.max_cols = parse_u16(value.to_string(), "--max-cols")?;
                        explicit.max_cols = Some(cfg.max_cols);
                    } else if let Some(value) = arg.strip_prefix("--max-rows=") {
                        cfg.max_rows = parse_u16(value.to_string(), "--max-rows")?;
                        explicit.max_rows = Some(cfg.max_rows);
                    } else if let Some(value) = arg.strip_prefix("--max-failures=") {
                        cfg.max_consecutive_failures =
                            parse_u32(value.to_string(), "--max-failures")?;
                    } else if let Some(value) = arg.strip_prefix("--backoff-ms=") {
                        cfg.backoff_base_ms = parse_u64(value.to_string(), "--backoff-ms")?;
                    } else {
                        return Err(AppError::Config(format!(
                            "Unknown argument: {arg}. Use --help for usage."
                        )));
                    }
                }
            }
        }

        if use_fast_preset {
            apply_fast_preset(&mut cfg);
        }
        if use_release_mode {
            apply_release_mode_preset(&mut cfg);
            cfg.release_mode = true;
        }
        apply_explicit_overrides(&mut cfg, &explicit);

        if cfg.width == 0 || cfg.height == 0 {
            return Err(AppError::Config(
                "Width and height must be greater than zero.".to_string(),
            ));
        }

        if cfg.fps == 0 {
            return Err(AppError::Config(
                "FPS must be greater than zero.".to_string(),
            ));
        }

        if cfg.gamma <= 0.0 {
            return Err(AppError::Config(
                "--gamma must be positive (for example, 1.0).".to_string(),
            ));
        }

        if cfg.contrast <= 0.0 {
            return Err(AppError::Config(
                "--contrast must be positive (for example, 1.0).".to_string(),
            ));
        }

        Ok(cfg)
    }
}

#[derive(Default)]
struct ExplicitOverrides {
    color_mode: Option<ColorMode>,
    ramp_name: Option<String>,
    show_metrics: Option<bool>,
    render_fps: Option<u32>,
    max_cols: Option<u16>,
    max_rows: Option<u16>,
}

pub fn print_help() {
    println!(
        "camgylph - real-time colored ASCII camera renderer\n\
\nUSAGE:\n  camgylph [options]\n\
\nOPTIONS:\n\
  --list-devices           List available camera devices and exit\n\
  --device <value>         Camera device input (e.g. \"0:none\", /dev/video0, \"Integrated Camera\")\n\
  --width <n>              Capture width (default: 640)\n\
  --height <n>             Capture height (default: 480)\n\
  --fps <n>                Capture FPS (default: 30)\n\
  --pix-fmt <fmt>          FFmpeg output pixel format (default: rgb24)\n\
  --ramp <name>            Ramp preset: standard | detailed\n\
  --no-color               Disable color output\n\
  --ansi256                Use ANSI 256-color mode\n\
  --truecolor              Use ANSI truecolor mode\n\
  --show-metrics           Show metrics/status line (default: off)\n\
  --hide-metrics           Hide metrics/status line\n\
  --fast                   Performance preset (ansi256, 20 FPS, no metrics, 120x40 render cap)\n\
  --release-mode           Production preset (ansi256, 60 FPS cap, no metrics, full terminal; explicit flags override)\n\
  --gamma <value>          Gamma adjustment (default: 1.0)\n\
  --contrast <value>       Contrast multiplier (default: 1.0)\n\
  --render-fps <n>         Max render FPS cap; 0 = follow camera FPS (default: 30)\n\
  --log-metrics-ms <n>     Periodic metrics logging to stderr (default: 0)\n\
  --max-cols <n>           Render width cap in terminal cells (default: 0 = terminal width)\n\
  --max-rows <n>           Render height cap in terminal cells (default: 0 = terminal height)\n\
  --max-failures <n>       Restart failure threshold (default: 5)\n\
  --backoff-ms <n>         Restart backoff base in ms (default: 500)\n\
  -h, --help               Show this help\n\
\nCONTROLS (while running):\n\
  q / Esc                  Quit\n\
  c                        Cycle color mode\n\
  r                        Cycle character ramp\n\
  m                        Toggle metrics details in the footer\n\
  h                        Show / hide footer shortcuts line\n\
  v                        Toggle horizontal mirror\n\
  + / -                    Increase / decrease gamma\n\
  ] / [                    Increase / decrease contrast"
    );
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, AppError> {
    args.next()
        .ok_or_else(|| AppError::Config(format!("Missing value for {flag}")))
}

fn parse_u32(raw: String, flag: &str) -> Result<u32, AppError> {
    raw.parse::<u32>()
        .map_err(|_| AppError::Config(format!("Invalid value for {flag}: {raw}")))
}

fn parse_u64(raw: String, flag: &str) -> Result<u64, AppError> {
    raw.parse::<u64>()
        .map_err(|_| AppError::Config(format!("Invalid value for {flag}: {raw}")))
}

fn parse_u16(raw: String, flag: &str) -> Result<u16, AppError> {
    raw.parse::<u16>()
        .map_err(|_| AppError::Config(format!("Invalid value for {flag}: {raw}")))
}

fn parse_f32(raw: String, flag: &str) -> Result<f32, AppError> {
    raw.parse::<f32>()
        .map_err(|_| AppError::Config(format!("Invalid value for {flag}: {raw}")))
}

fn apply_fast_preset(cfg: &mut Config) {
    cfg.color_mode = ColorMode::Ansi256;
    cfg.ramp_name = "standard".to_string();
    cfg.show_metrics = false;
    cfg.render_fps = 20;
    cfg.max_cols = 120;
    cfg.max_rows = 40;
}

fn apply_release_mode_preset(cfg: &mut Config) {
    cfg.color_mode = ColorMode::Ansi256;
    cfg.ramp_name = "standard".to_string();
    cfg.show_metrics = false;
    cfg.render_fps = 60;
    cfg.max_cols = 0;
    cfg.max_rows = 0;
}

fn apply_explicit_overrides(cfg: &mut Config, explicit: &ExplicitOverrides) {
    if let Some(color_mode) = explicit.color_mode {
        cfg.color_mode = color_mode;
    }
    if let Some(ramp_name) = &explicit.ramp_name {
        cfg.ramp_name = ramp_name.clone();
    }
    if let Some(show_metrics) = explicit.show_metrics {
        cfg.show_metrics = show_metrics;
    }
    if let Some(render_fps) = explicit.render_fps {
        cfg.render_fps = render_fps;
    }
    if let Some(max_cols) = explicit.max_cols {
        cfg.max_cols = max_cols;
    }
    if let Some(max_rows) = explicit.max_rows {
        cfg.max_rows = max_rows;
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::ascii::color::ColorMode;

    fn parse(args: &[&str]) -> Config {
        Config::from_args(args.iter().map(|s| s.to_string())).expect("valid args")
    }

    #[test]
    fn release_mode_applies_expected_defaults() {
        let cfg = parse(&["--release-mode"]);
        assert!(cfg.release_mode);
        assert_eq!(cfg.color_mode, ColorMode::Ansi256);
        assert_eq!(cfg.ramp_name, "standard");
        assert!(!cfg.show_metrics);
        assert_eq!(cfg.render_fps, 60);
        assert_eq!(cfg.max_cols, 0);
        assert_eq!(cfg.max_rows, 0);
        assert_eq!(cfg.fps, 30);
    }

    #[test]
    fn explicit_overrides_win_even_after_release_mode() {
        let cfg = parse(&[
            "--release-mode",
            "--truecolor",
            "--ramp",
            "detailed",
            "--show-metrics",
            "--render-fps",
            "48",
            "--max-cols",
            "100",
            "--max-rows",
            "30",
        ]);

        assert!(cfg.release_mode);
        assert_eq!(cfg.color_mode, ColorMode::Truecolor);
        assert_eq!(cfg.ramp_name, "detailed");
        assert!(cfg.show_metrics);
        assert_eq!(cfg.render_fps, 48);
        assert_eq!(cfg.max_cols, 100);
        assert_eq!(cfg.max_rows, 30);
    }

    #[test]
    fn explicit_overrides_win_even_before_release_mode() {
        let cfg = parse(&[
            "--truecolor",
            "--ramp=detailed",
            "--show-metrics",
            "--render-fps=48",
            "--max-cols=100",
            "--max-rows=30",
            "--release-mode",
        ]);

        assert!(cfg.release_mode);
        assert_eq!(cfg.color_mode, ColorMode::Truecolor);
        assert_eq!(cfg.ramp_name, "detailed");
        assert!(cfg.show_metrics);
        assert_eq!(cfg.render_fps, 48);
        assert_eq!(cfg.max_cols, 100);
        assert_eq!(cfg.max_rows, 30);
    }
}
