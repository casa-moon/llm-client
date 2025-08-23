use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// Start a standardized spinner with message and tick style
pub fn start(message: impl Into<String>) -> ProgressBar {
  let pb = ProgressBar::new_spinner();
  pb.set_style(
    ProgressStyle::with_template("{spinner} {msg}")
      .unwrap()
      .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
  );
  pb.set_message(message.into());
  pb.enable_steady_tick(Duration::from_millis(80));
  pb
}

// Stop and clear the spinner (no message)
pub fn stop(pb: &ProgressBar) {
  pb.finish_and_clear();
}
