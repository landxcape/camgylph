use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct MetricsSnapshot {
    pub total_frames: u64,
    pub fps: f32,
    pub frame_time_ms: f32,
}

pub struct Metrics {
    total_frames: u64,
    frame_time_ms: f32,
    fps: f32,
    fps_window_start: Instant,
    fps_window_frames: u32,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_frames: 0,
            frame_time_ms: 0.0,
            fps: 0.0,
            fps_window_start: Instant::now(),
            fps_window_frames: 0,
        }
    }

    pub fn begin_frame(&self) -> Instant {
        Instant::now()
    }

    pub fn end_frame(&mut self, started_at: Instant, total_frames: u64) {
        self.total_frames = total_frames;
        self.frame_time_ms = started_at.elapsed().as_secs_f32() * 1000.0;
        self.fps_window_frames += 1;

        let elapsed = self.fps_window_start.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.fps = self.fps_window_frames as f32 / elapsed.as_secs_f32();
            self.fps_window_frames = 0;
            self.fps_window_start = Instant::now();
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_frames: self.total_frames,
            fps: self.fps,
            frame_time_ms: self.frame_time_ms,
        }
    }
}
