use crate::camera::device;
use std::{
    io::{self, ErrorKind, Read},
    process::{Command, Stdio},
};

pub struct CaptureConfig<'a> {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub pix_fmt: &'a str,
    pub device: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureLoopControl {
    Continue,
    Stop,
}

pub struct CaptureSessionOutcome {
    pub frames: u64,
    pub stopped_by_user: bool,
}

pub fn run_capture_session<F>(
    config: &CaptureConfig<'_>,
    mut on_frame: F,
) -> io::Result<CaptureSessionOutcome>
where
    F: FnMut(u64, &[u8]) -> io::Result<CaptureLoopControl>,
{
    let frame_size = frame_size(config)?;

    let mut args = vec!["-loglevel".into(), "error".into(), "-nostdin".into()];
    args.extend(input_args(config)?);
    args.extend(output_args(config.pix_fmt, config.fps));

    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("ffmpeg stdout not piped"))?;

    let mut frame = vec![0u8; frame_size];
    let mut session_frames: u64 = 0;

    loop {
        match stdout.read_exact(&mut frame) {
            Ok(()) => {
                let control = on_frame(session_frames, &frame)?;
                session_frames += 1;

                if control == CaptureLoopControl::Stop {
                    let _ = child.kill();
                    let status = child.wait()?;
                    let _ = status;
                    return Ok(CaptureSessionOutcome {
                        frames: session_frames,
                        stopped_by_user: true,
                    });
                }
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
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
    let _ = status;
    Ok(CaptureSessionOutcome {
        frames: session_frames,
        stopped_by_user: false,
    })
}

pub fn list_devices(device_hint: Option<&str>) -> io::Result<()> {
    #[cfg(not(target_os = "linux"))]
    let _ = device_hint;

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-f",
                "avfoundation",
                "-list_devices",
                "true",
                "-i",
                "",
            ])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        print_filtered_lines(&stderr, |line| line.contains("AVFoundation indev"));
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-f",
                "dshow",
                "-list_devices",
                "true",
                "-i",
                "dummy",
            ])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        print_filtered_lines(&stderr, |line| {
            line.contains("dshow")
                || line.contains("DirectShow")
                || line.trim_start().starts_with('"')
        });
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let target = device::listing_target(device_hint);
        eprintln!(
            "Listing formats for selected Linux device: {} (ffmpeg does not enumerate all v4l2 devices)",
            target
        );

        let status = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-f",
                "v4l2",
                "-list_formats",
                "all",
                "-i",
                &target,
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if !status.success() {
            eprintln!("device listing command exited with status {}", status);
        }
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

pub fn preflight_source_check(config: &CaptureConfig<'_>) -> io::Result<()> {
    let mut args = vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-nostdin".into(),
    ];
    args.extend(input_args(config)?);
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
    Err(io::Error::other(message))
}

fn frame_size(config: &CaptureConfig<'_>) -> io::Result<usize> {
    if config.pix_fmt != "rgb24" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Unsupported pix_fmt '{}'. Current renderer expects rgb24.",
                config.pix_fmt
            ),
        ));
    }

    Ok((config.width as usize)
        .saturating_mul(config.height as usize)
        .saturating_mul(3))
}

fn input_args(config: &CaptureConfig<'_>) -> io::Result<Vec<String>> {
    let device_spec = device::resolve_input_spec(config.device.as_deref())?;

    #[cfg(target_os = "macos")]
    {
        return Ok(vec![
            "-f".into(),
            "avfoundation".into(),
            "-pixel_format".into(),
            "nv12".into(),
            "-framerate".into(),
            config.fps.to_string(),
            "-video_size".into(),
            format!("{}x{}", config.width, config.height),
            "-i".into(),
            device_spec,
        ]);
    }

    #[cfg(target_os = "linux")]
    {
        return Ok(vec![
            "-f".into(),
            "v4l2".into(),
            "-framerate".into(),
            config.fps.to_string(),
            "-video_size".into(),
            format!("{}x{}", config.width, config.height),
            "-i".into(),
            device_spec,
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        return Ok(vec![
            "-f".into(),
            "dshow".into(),
            "-framerate".into(),
            config.fps.to_string(),
            "-video_size".into(),
            format!("{}x{}", config.width, config.height),
            "-i".into(),
            device_spec,
        ]);
    }

    #[allow(unreachable_code)]
    Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported OS"))
}

fn output_args(pix_fmt: &str, fps: u32) -> Vec<String> {
    vec![
        "-vf".into(),
        format!("fps={fps}"),
        "-r".into(),
        fps.to_string(),
        "-pix_fmt".into(),
        pix_fmt.into(),
        "-f".into(),
        "rawvideo".into(),
        "-".into(),
    ]
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

fn print_filtered_lines<F>(stderr: &str, mut predicate: F)
where
    F: FnMut(&str) -> bool,
{
    let mut printed = 0usize;
    for line in stderr.lines() {
        if predicate(line) {
            println!("{line}");
            printed += 1;
        }
    }

    if printed == 0 {
        eprintln!("No camera devices were detected, or ffmpeg output format changed.");
    }
}
