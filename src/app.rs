use crate::{
    ascii::{
        color::{ColorMode, mode_label},
        mapper::map_rgb_frame,
        ramp,
    },
    camera::ffmpeg::{
        CaptureConfig, CaptureLoopControl, list_devices, preflight_source_check,
        run_capture_session,
    },
    config::Config,
    error::AppError,
    frame::frame::RgbFrameView,
    metrics::Metrics,
    terminal::{
        input::{Control, poll_controls},
        renderer::TerminalRenderer,
        screen::TerminalScreen,
    },
};
use std::{
    io, thread,
    time::{Duration, Instant},
};

const TERMINAL_CELL_ASPECT_RATIO: f32 = 0.5;

struct RuntimeState {
    quit: bool,
    show_metrics: bool,
    color_mode: ColorMode,
    ramp_name: String,
    gamma: f32,
    contrast: f32,
}

impl RuntimeState {
    fn from_config(config: &Config) -> Self {
        Self {
            quit: false,
            show_metrics: config.show_metrics,
            color_mode: config.color_mode,
            ramp_name: config.ramp_name.clone(),
            gamma: config.gamma,
            contrast: config.contrast,
        }
    }
}

pub fn run(config: Config) -> Result<(), AppError> {
    if !ramp::is_valid(&config.ramp_name) {
        return Err(AppError::Config(format!(
            "Unknown ramp '{}'. Valid values: standard, detailed",
            config.ramp_name
        )));
    }

    if config.list_devices {
        list_devices(config.device.as_deref())?;
        return Ok(());
    }

    let capture_config = CaptureConfig {
        width: config.width,
        height: config.height,
        fps: config.fps,
        pix_fmt: &config.pix_fmt,
        device: config.device.clone(),
    };

    preflight_source_check(&capture_config)?;

    let _screen = TerminalScreen::enter()?;
    let mut renderer = TerminalRenderer::new();
    let mut metrics = Metrics::new();
    let mut state = RuntimeState::from_config(&config);

    let mut consecutive_failures = 0u32;
    let mut total_frames = 0u64;
    let mut last_render_at: Option<Instant> = None;
    let mut last_metrics_log = Instant::now();
    let effective_render_fps = if config.render_fps == 0 {
        capture_config.fps
    } else {
        config.render_fps
    };

    loop {
        let base_index = total_frames;
        let outcome = run_capture_session(&capture_config, |session_idx, frame_bytes| {
            for control in poll_controls()? {
                apply_control(&mut state, control);
            }

            if state.quit {
                return Ok(CaptureLoopControl::Stop);
            }

            if !should_render_now(effective_render_fps, &mut last_render_at) {
                return Ok(CaptureLoopControl::Continue);
            }

            let started_at = metrics.begin_frame();

            let frame = RgbFrameView::new(capture_config.width, capture_config.height, frame_bytes)
                .ok_or_else(|| io::Error::other("Received unexpected frame byte length"))?;

            let (mut term_w, mut term_h) = renderer.current_size()?;
            if config.max_cols > 0 {
                term_w = term_w.min(config.max_cols);
            }
            if config.max_rows > 0 {
                term_h = term_h.min(config.max_rows);
            }
            let (out_w, out_h) = compute_render_dimensions(term_w, term_h, state.show_metrics);
            let mapped = map_rgb_frame(
                &frame,
                out_w,
                out_h,
                ramp::by_name(&state.ramp_name),
                state.gamma,
                state.contrast,
                TERMINAL_CELL_ASPECT_RATIO,
            );

            let status_line = if state.show_metrics {
                Some(build_status_line(
                    &state,
                    &metrics,
                    base_index + session_idx,
                ))
            } else {
                None
            };
            renderer.render(&mapped, state.color_mode, status_line.as_deref())?;

            metrics.end_frame(started_at, base_index + session_idx + 1);
            maybe_log_metrics(config.log_metrics_ms, &metrics, &mut last_metrics_log);
            Ok(CaptureLoopControl::Continue)
        })?;

        total_frames += outcome.frames;

        if state.quit || outcome.stopped_by_user {
            break;
        }

        if outcome.frames == 0 {
            consecutive_failures += 1;
            if consecutive_failures > config.max_consecutive_failures {
                return Err(AppError::Io(io::Error::other(format!(
                    "Capture failed {} times in a row; stopping.",
                    consecutive_failures
                ))));
            }
        } else {
            consecutive_failures = 0;
        }

        let exp = consecutive_failures.saturating_sub(1).min(5);
        let backoff_ms = config.backoff_base_ms.saturating_mul(1u64 << exp);

        if backoff_ms > 0 {
            thread::sleep(Duration::from_millis(backoff_ms));
        }
    }

    renderer.finish()?;
    Ok(())
}

fn apply_control(state: &mut RuntimeState, control: Control) {
    match control {
        Control::Quit => state.quit = true,
        Control::ToggleColorMode => state.color_mode = state.color_mode.next(),
        Control::ToggleRamp => state.ramp_name = ramp::next_name(&state.ramp_name).to_string(),
        Control::ToggleMetrics => state.show_metrics = !state.show_metrics,
        Control::IncreaseGamma => state.gamma = (state.gamma + 0.1).clamp(0.2, 3.0),
        Control::DecreaseGamma => state.gamma = (state.gamma - 0.1).clamp(0.2, 3.0),
        Control::IncreaseContrast => state.contrast = (state.contrast + 0.1).clamp(0.2, 3.0),
        Control::DecreaseContrast => state.contrast = (state.contrast - 0.1).clamp(0.2, 3.0),
    }
}

fn build_status_line(state: &RuntimeState, metrics: &Metrics, frame_idx: u64) -> String {
    let snap = metrics.snapshot();
    format!(
        "frame:{} total:{} fps:{:.1} frame_ms:{:.2} color:{} ramp:{} gamma:{:.1} contrast:{:.1} | q:quit c:color r:ramp +/-:gamma []:contrast m:metrics",
        frame_idx,
        snap.total_frames,
        snap.fps,
        snap.frame_time_ms,
        mode_label(state.color_mode),
        state.ramp_name,
        state.gamma,
        state.contrast,
    )
}

fn compute_render_dimensions(term_w: u16, term_h: u16, show_metrics: bool) -> (u16, u16) {
    let out_w = term_w.max(1);
    let mut out_h = term_h.max(1);
    if show_metrics {
        out_h = out_h.saturating_sub(1).max(1);
    }
    (out_w, out_h)
}

fn should_render_now(render_fps: u32, last_render_at: &mut Option<Instant>) -> bool {
    if render_fps == 0 {
        return true;
    }

    let now = Instant::now();
    let target = Duration::from_secs_f64(1.0 / render_fps as f64);

    if let Some(last) = last_render_at {
        if now.duration_since(*last) < target {
            return false;
        }
    }

    *last_render_at = Some(now);
    true
}

fn maybe_log_metrics(interval_ms: u64, metrics: &Metrics, last_logged: &mut Instant) {
    if interval_ms == 0 {
        return;
    }

    if last_logged.elapsed() < Duration::from_millis(interval_ms) {
        return;
    }

    let snap = metrics.snapshot();
    eprintln!(
        "metrics total={} fps={:.1} frame_ms={:.2}",
        snap.total_frames, snap.fps, snap.frame_time_ms
    );
    *last_logged = Instant::now();
}
