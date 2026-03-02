use std::{
    io::Read,
    process::{Command, Stdio},
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const FPS: u32 = 30;
const PIX_FMT: &str = "rgb24";

fn main() -> std::io::Result<()> {
    let frame_size = (WIDTH * HEIGHT * 3) as usize;

    let mut args = vec!["-loglevel".into(), "error".into(), "-nostdin".into()];

    args.extend(input_args(WIDTH, HEIGHT, FPS)?);
    args.extend(output_args(PIX_FMT));

    let mut child = Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "ffmpeg stdout not piped"))?;

    let mut frame = vec![0u8; frame_size];
    stdout.read_exact(&mut frame)?;

    println!("got frame: {}", frame.len());

    let _ = child.kill();
    let _ = child.wait();

    Ok(())
}

fn input_args(width: u32, height: u32, fps: u32) -> std::io::Result<Vec<String>> {
    #[cfg(target_os = "macos")]
    {
        return Ok(vec![
            "-f".into(),
            "avfoundation".into(),
            "-framerate".into(),
            fps.to_string(),
            "-video_size".into(),
            format!("{width}x{height}"),
            "-i".into(),
            "0:none".into(),
        ]);
    }

    #[cfg(target_os = "linux")]
    {
        return Ok(vec![
            "-f".into(),
            "v4l2".into(),
            "-framerate".into(),
            fps.to_string(),
            "-video_size".into(),
            format!("{width}x{height}"),
            "-i".into(),
            "/dev/video0".into(),
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        let camera_name = std::env::args()
            .nth(1)
            .or_else(|| std::env::var("CAMGLYPH_CAMERA").ok())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Windows needs camera name: pass argv[1] or CAMGLYPH_CAMERA",
                )
            })?;

        return Ok(vec![
            "-f".into(),
            "dshow".into(),
            "-framerate".into(),
            fps.to_string(),
            "-video_size".into(),
            format!("{width}x{height}"),
            "-i".into(),
            format!("video={camera_name}"),
        ]);
    }

    #[allow(unreachable_code)]
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unsupported OS",
        ));
    }
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
