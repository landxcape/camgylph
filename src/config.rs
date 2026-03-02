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
            show_help: false,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let mut cfg = Self::default();
        let mut args = std::env::args().skip(1);

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
                "--ramp" => cfg.ramp_name = next_value(&mut args, "--ramp")?,
                "--show-metrics" => cfg.show_metrics = true,
                "--hide-metrics" => cfg.show_metrics = false,
                "--fast" => apply_fast_preset(&mut cfg),
                "--gamma" => cfg.gamma = parse_f32(next_value(&mut args, "--gamma")?, "--gamma")?,
                "--contrast" => {
                    cfg.contrast = parse_f32(next_value(&mut args, "--contrast")?, "--contrast")?
                }
                "--render-fps" => {
                    cfg.render_fps =
                        parse_u32(next_value(&mut args, "--render-fps")?, "--render-fps")?
                }
                "--log-metrics-ms" => {
                    cfg.log_metrics_ms = parse_u64(
                        next_value(&mut args, "--log-metrics-ms")?,
                        "--log-metrics-ms",
                    )?
                }
                "--max-cols" => {
                    cfg.max_cols = parse_u16(next_value(&mut args, "--max-cols")?, "--max-cols")?
                }
                "--max-rows" => {
                    cfg.max_rows = parse_u16(next_value(&mut args, "--max-rows")?, "--max-rows")?
                }
                "--max-failures" => {
                    cfg.max_consecutive_failures =
                        parse_u32(next_value(&mut args, "--max-failures")?, "--max-failures")?
                }
                "--backoff-ms" => {
                    cfg.backoff_base_ms =
                        parse_u64(next_value(&mut args, "--backoff-ms")?, "--backoff-ms")?
                }
                "--no-color" => cfg.color_mode = ColorMode::None,
                "--ansi256" => cfg.color_mode = ColorMode::Ansi256,
                "--truecolor" => cfg.color_mode = ColorMode::Truecolor,
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
                    } else if let Some(value) = arg.strip_prefix("--gamma=") {
                        cfg.gamma = parse_f32(value.to_string(), "--gamma")?;
                    } else if let Some(value) = arg.strip_prefix("--contrast=") {
                        cfg.contrast = parse_f32(value.to_string(), "--contrast")?;
                    } else if let Some(value) = arg.strip_prefix("--render-fps=") {
                        cfg.render_fps = parse_u32(value.to_string(), "--render-fps")?;
                    } else if let Some(value) = arg.strip_prefix("--log-metrics-ms=") {
                        cfg.log_metrics_ms = parse_u64(value.to_string(), "--log-metrics-ms")?;
                    } else if let Some(value) = arg.strip_prefix("--max-cols=") {
                        cfg.max_cols = parse_u16(value.to_string(), "--max-cols")?;
                    } else if let Some(value) = arg.strip_prefix("--max-rows=") {
                        cfg.max_rows = parse_u16(value.to_string(), "--max-rows")?;
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
