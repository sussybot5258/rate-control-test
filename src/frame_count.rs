use std::time::SystemTime;

pub struct FrameCounter {
    frame_counts: u32,
    frame_timer: SystemTime,
}

impl FrameCounter {
    pub fn new() -> Self {
        FrameCounter {
            frame_counts: 0,
            frame_timer: SystemTime::now(),
        }
    }
    pub fn reset(&mut self) {
        self.frame_timer = SystemTime::now();
        self.frame_counts = 0;
    }
    pub fn count(&mut self) {
        // compare time
        // if over 1 sec, print frame counts, reset time and count
        if SystemTime::now()
            .duration_since(self.frame_timer)
            .unwrap()
            .as_secs_f64()
            > 1.
        {
            println!("frames in one sec: {}", self.frame_counts);
            self.reset();
        } else {
            self.frame_counts += 1;
        }
    }
}
