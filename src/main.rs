mod camera;

use std::{thread, time::Duration};

use camera::ffmpeg::{maybe_list_devices_and_exit, preflight_source_check, run_capture_session, CaptureConfig};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const FPS: u32 = 30;
const PIX_FMT: &str = "rgb24";
const LOG_EVERY: u64 = 30;

const MAX_CONSECUTIVE_FAILURES: u32 = 5;
const BACKOFF_BASE_MS: u64 = 500;

fn main() -> std::io::Result<()> {
    if maybe_list_devices_and_exit()? {
        return Ok(());
    }
    let config = CaptureConfig {
        width: WIDTH,
        height: HEIGHT,
        fps: FPS,
        pix_fmt: PIX_FMT,
    };

    preflight_source_check(&config)?;

    let mut consecutive_failures: u32 = 0;
    let mut total_frames: u64 = 0;

    loop {
        let base_index = total_frames;
        let (session_frames, status) = run_capture_session(&config, |session_idx, frame| {
            let global_index = base_index + session_idx;
            if global_index % LOG_EVERY == 0 {
                let avg = avg_luminance_rgb24(frame);
                println!("frame={} avg_luma={:.2}", global_index, avg);
            }
        })?;
        total_frames += session_frames;

        eprintln!(
            "session ended: frames={} total_frames={} status={}",
            session_frames, total_frames, status
        );

        if session_frames == 0 {
            consecutive_failures += 1;
            if consecutive_failures > MAX_CONSECUTIVE_FAILURES {
                return Err(std::io::Error::other(format!(
                    "capture failed {} times in a row; stopping",
                    consecutive_failures
                )));
            }
        } else {
            consecutive_failures = 0;
        }

        let exp = consecutive_failures.saturating_sub(1).min(5);
        let backoff_ms = BACKOFF_BASE_MS.saturating_mul(1u64 << exp);
        eprintln!(
            "restarting ffmpeg in {} ms (consecutive failures: {})",
            backoff_ms, consecutive_failures
        );
        thread::sleep(Duration::from_millis(backoff_ms));
    }
}

fn avg_luminance_rgb24(frame: &[u8]) -> f32 {
    let mut sum: u64 = 0;
    let pixels = frame.len() / 3;

    for px in frame.chunks_exact(3) {
        let r = px[0] as u32;
        let g = px[1] as u32;
        let b = px[2] as u32;
        let y = (r * 77 + g * 150 + b * 29) >> 8;
        sum += y as u64;
    }

    sum as f32 / pixels as f32
}
