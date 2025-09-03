use std::time::{Duration, Instant};

// Configuration constants
pub const ENABLE_FPS_LIMIT: bool = true;
pub const MAX_FPS: f32 = 320.0;

pub struct FrameTiming {
    last_frame_time: Instant,
    frame_count_since_last_update: u32,
    fps_update_time: Instant,
}

impl FrameTiming {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_frame_time: now,
            frame_count_since_last_update: 0,
            fps_update_time: now,
        }
    }

    pub fn update_fps(&mut self) -> Option<f32> {
        let now = Instant::now();
        self.frame_count_since_last_update += 1;

        // Update FPS every second
        if now.duration_since(self.fps_update_time) >= Duration::from_secs(1) {
            let fps = self.frame_count_since_last_update as f32
                / now.duration_since(self.fps_update_time).as_secs_f32();

            self.frame_count_since_last_update = 0;
            self.fps_update_time = now;
            
            self.last_frame_time = now;
            Some(fps)
        } else {
            self.last_frame_time = now;
            None
        }
    }

    pub fn apply_fps_limit(&self) {
        if ENABLE_FPS_LIMIT {
            let target_frame_duration = Duration::from_secs_f32(1.0 / MAX_FPS);
            let elapsed = self.last_frame_time.elapsed();

            if elapsed < target_frame_duration {
                let sleep_time = target_frame_duration - elapsed;
                std::thread::sleep(sleep_time);
            }
        }
    }
}

impl Default for FrameTiming {
    fn default() -> Self {
        Self::new()
    }
}