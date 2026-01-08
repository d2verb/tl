use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct Spinner {
    progress_bar: ProgressBar,
}

impl Spinner {
    #[allow(clippy::unwrap_used)]
    pub fn new(message: &str) -> Self {
        let progress_bar = ProgressBar::new_spinner();
        // unwrap is safe: template string is a compile-time constant
        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .unwrap(),
        );
        progress_bar.set_message(message.to_string());
        progress_bar.enable_steady_tick(Duration::from_millis(80));

        Self { progress_bar }
    }

    pub const fn start(&self) {
        // Spinner is already running from new()
    }

    pub fn stop(&self) {
        self.progress_bar.finish_and_clear();
    }
}
