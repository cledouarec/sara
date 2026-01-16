//! Progress indicators for long operations.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Creates a spinner for indeterminate progress operations.
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .expect("invalid template"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Creates a spinner for building the knowledge graph.
pub fn create_graph_building_spinner() -> ProgressBar {
    create_spinner("Building knowledge graph...")
}

/// Finishes a progress bar with a success message.
pub fn finish_with_success(pb: &ProgressBar, message: &str) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("invalid template"),
    );
    pb.finish_with_message(format!("✅ {}", message));
}

/// Finishes a progress bar with an error message.
pub fn finish_with_error(pb: &ProgressBar, message: &str) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.red} {msg}")
            .expect("invalid template"),
    );
    pb.finish_with_message(format!("❌ {}", message));
}

/// Finishes a progress bar by clearing it.
pub fn finish_and_clear(pb: &ProgressBar) {
    pb.finish_and_clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_spinner() {
        let pb = create_spinner("Test operation");
        pb.finish_and_clear();
    }

    #[test]
    fn test_finish_functions() {
        let pb = create_spinner("Test");
        finish_with_success(&pb, "Done");

        let pb2 = create_spinner("Test");
        finish_with_error(&pb2, "Failed");

        let pb3 = create_spinner("Test");
        finish_and_clear(&pb3);
    }
}
