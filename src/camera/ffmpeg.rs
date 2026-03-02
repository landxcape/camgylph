use std::{
    io::{ErrorKind, Read},
    process::{Command, ExitStatus, Stdio},
};

pub struct CaptureConfig<'a> {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub pix_fmt: &'a str,
}

pub fn run_capture_session<F>(config: &CaptureConfig<'_>, mut on_frame: F) -> std::io::Result<(u64, ExitStatus)>
where
    F: FnMut(u64, &[u8]),
{
    let frame_size = (config.width * config.height * 3) as usize;

    let mut args = vec!["-loglevel".into(), "error".into(), "-nostdin".into()];
    args.extend(input_args(config.width, config.height, config.fps)?);
    args.extend(output_args(config.pix_fmt));

    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("ffmpeg stdout not piped"))?;

    let mut frame = vec![0u8; frame_size];
    let mut session_frames: u64 = 0;

    loop {
        match stdout.read_exact(&mut frame) {
            Ok(()) => {
                on_frame(session_frames, &frame);
                session_frames += 1;
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                eprintln!("stream ended (EOF)");
                break;
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        }
    }

    let status = child.wait()?;
    Ok((session_frames, status))
}

pub fn maybe_list_devices_and_exit() -> std::io::Result<bool> {
    if !cli_has_flag("--list-devices") {
        return Ok(false);
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("ffmpeg")
            .args(["-hide_banner", "-f", "avfoundation", "-list_devices", "true", "-i", ""])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut printed = 0usize;
        for line in stderr.lines() {
            if line.contains("AVFoundation indev") {
                println!("{line}");
                printed += 1;
            }
        }

        if printed == 0 {
            eprintln!("No AVFoundation devices detected or ffmpeg output format changed.");
        }

        return Ok(true);
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("ffmpeg")
            .args(["-hide_banner", "-f", "dshow", "-list_devices", "true", "-i", "dummy"])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut printed = 0usize;
        for line in stderr.lines() {
            if line.contains("dshow") || line.contains("DirectShow") || line.trim_start().starts_with('"') {
                println!("{line}");
                printed += 1;
            }
        }

        if printed == 0 {
            eprintln!("No DirectShow devices detected or ffmpeg output format changed.");
        }

        return Ok(true);
    }

    #[cfg(target_os = "linux")]
    {
        let device = cli_device_arg()
            .or_else(|| std::env::var("CAMGLYPH_DEVICE").ok())
            .unwrap_or_else(|| "/dev/video0".to_string());

        eprintln!(
            "Listing formats for selected Linux device: {} (ffmpeg does not enumerate all v4l2 devices)",
            device
        );

        let status = Command::new("ffmpeg")
            .args(["-hide_banner", "-f", "v4l2", "-list_formats", "all", "-i", &device])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if !status.success() {
            eprintln!("device listing command exited with status {}", status);
        }

        return Ok(true);
    }

    #[allow(unreachable_code)]
    Ok(false)
}

pub fn preflight_source_check(config: &CaptureConfig<'_>) -> std::io::Result<()> {
    let mut args = vec!["-hide_banner".into(), "-loglevel".into(), "error".into(), "-nostdin".into()];
    args.extend(input_args(config.width, config.height, config.fps)?);
    args.extend([
        "-frames:v".into(),
        "1".into(),
        "-f".into(),
        "null".into(),
        "-".into(),
    ]);

    let output = Command::new("ffmpeg").args(&args).output()?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = classify_preflight_error(&stderr, config.width, config.height, config.fps);
    Err(std::io::Error::other(message))
}

fn classify_preflight_error(stderr: &str, width: u32, height: u32, fps: u32) -> String {
    let lower = stderr.to_lowercase();

    if lower.contains("not supported by the device") || lower.contains("selected video size") {
        return format!(
            "Unsupported camera mode {}x{} @ {}fps. Use --list-devices and choose a supported mode or device.",
            width, height, fps
        );
    }

    if lower.contains("permission denied")
        || lower.contains("operation not permitted")
        || lower.contains("not authorized")
        || lower.contains("cannot be accessed")
    {
        #[cfg(target_os = "macos")]
        {
            return "Camera permission missing. Enable camera access for your terminal app in System Settings -> Privacy & Security -> Camera.".to_string();
        }

        #[cfg(not(target_os = "macos"))]
        {
            return "Camera permission missing or denied for selected device. Check OS camera privacy settings and device access permissions.".to_string();
        }
    }

    if lower.contains("no such file or directory")
        || lower.contains("error opening input file")
        || lower.contains("could not find video device")
        || lower.contains("device not found")
    {
        return "Camera device could not be opened. Verify --device / CAMGLYPH_DEVICE and run --list-devices to discover valid inputs.".to_string();
    }

    let preview = stderr.lines().take(4).collect::<Vec<_>>().join(" | ");
    format!("Camera preflight failed: {}", preview)
}

fn cli_has_flag(flag: &str) -> bool {
    std::env::args().skip(1).any(|arg| arg == flag)
}

fn cli_device_arg() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--device" {
            return args.next();
        }
        if let Some(value) = arg.strip_prefix("--device=") {
            return Some(value.to_string());
        }
    }
    None
}

fn input_args(width: u32, height: u32, fps: u32) -> std::io::Result<Vec<String>> {
    #[cfg(target_os = "macos")]
    {
        let device = cli_device_arg()
            .or_else(|| std::env::var("CAMGLYPH_DEVICE").ok())
            .unwrap_or_else(|| "0:none".to_string());

        return Ok(vec![
            "-f".into(),
            "avfoundation".into(),
            "-pixel_format".into(),
            "nv12".into(),
            "-framerate".into(),
            fps.to_string(),
            "-video_size".into(),
            format!("{width}x{height}"),
            "-i".into(),
            device,
        ]);
    }

    #[cfg(target_os = "linux")]
    {
        let device = cli_device_arg()
            .or_else(|| std::env::var("CAMGLYPH_DEVICE").ok())
            .unwrap_or_else(|| "/dev/video0".to_string());

        return Ok(vec![
            "-f".into(),
            "v4l2".into(),
            "-framerate".into(),
            fps.to_string(),
            "-video_size".into(),
            format!("{width}x{height}"),
            "-i".into(),
            device,
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        let raw_device = cli_device_arg()
            .or_else(|| std::env::var("CAMGLYPH_DEVICE").ok())
            .or_else(|| std::env::var("CAMGLYPH_CAMERA").ok())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Windows needs device: --device \"Integrated Camera\" or CAMGLYPH_DEVICE",
                )
            })?;

        let input_device = if raw_device.starts_with("video=") {
            raw_device
        } else {
            format!("video={raw_device}")
        };

        return Ok(vec![
            "-f".into(),
            "dshow".into(),
            "-framerate".into(),
            fps.to_string(),
            "-video_size".into(),
            format!("{width}x{height}"),
            "-i".into(),
            input_device,
        ]);
    }

    #[allow(unreachable_code)]
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Unsupported OS",
    ))
}

fn output_args(pix_fmt: &str) -> Vec<String> {
    vec![
        "-pix_fmt".into(),
        pix_fmt.into(),
        "-f".into(),
        "rawvideo".into(),
        "-".into(),
    ]
}
