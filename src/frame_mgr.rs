use crate::app::App;

pub(crate) struct FrameManager {
    current_instant: std::time::Instant,
    previous_instant: std::time::Instant,
    accumulated: std::time::Duration,
    target: std::time::Duration,
}

impl FrameManager {
    pub(crate) fn new(framerate: f64) -> Self {
        Self {
            current_instant: std::time::Instant::now(),
            previous_instant: std::time::Instant::now(),
            accumulated: std::time::Duration::from_secs(0),
            target: std::time::Duration::from_secs_f64(1. / framerate),
        }
    }

    pub(crate) fn next_frame(&mut self, app: &App) -> bool {
        self.current_instant = std::time::Instant::now();
        let time_delta = self.current_instant - self.previous_instant;

        self.accumulated += time_delta;

        if self.accumulated >= self.target {
            app.update();

            self.accumulated -= self.target;
        }

        if let Err(e) = app.render() {
            eprintln!("Error rendering: {}", e);
            return false;
        }

        self.previous_instant = self.current_instant;

        true
    }
}
