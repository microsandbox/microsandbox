//! This module provides a global multi-progress bar for CLI visualizations.
//! It allows for tracking multiple progress bars in a single view.
//!
//! The `MULTI_PROGRESS` constant is a lazy-initialized `Arc<MultiProgress>` that
//! manages a collection of progress bars. It is used to display multiple progress
//! indicators simultaneously, such as when downloading multiple layers or

use indicatif::{MultiProgress, MultiProgressAlignment, ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use std::sync::{Arc, LazyLock};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

#[cfg(feature = "cli-viz")]
pub(crate) static MULTI_PROGRESS: Lazy<Arc<MultiProgress>> = Lazy::new(|| {
    let mp = MultiProgress::new();
    mp.set_alignment(MultiProgressAlignment::Top);
    Arc::new(mp)
});

static CHECKMARK: LazyLock<String> = LazyLock::new(|| format!("{}", console::style("✓").green()));
static ERROR_MARK: LazyLock<String> = LazyLock::new(|| format!("{}", console::style("✗").red()));

pub(crate) static TICK_STRINGS: LazyLock<[&str; 11]> =
    LazyLock::new(|| ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", &CHECKMARK]);

pub(crate) static ERROR_TICK_STRINGS: LazyLock<[&str; 11]> = LazyLock::new(|| {
    [
        "⠋",
        "⠙",
        "⠹",
        "⠸",
        "⠼",
        "⠴",
        "⠦",
        "⠧",
        "⠇",
        "⠏",
        &ERROR_MARK,
    ]
});

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Creates a spinner progress bar with a message for visualizing operations like fetching.
///
/// This is a utility function to standardize the creation of progress spinners across
/// different operations such as fetching indexes, manifests, and configs.
///
/// ## Arguments
///
/// * `message` - The message to display next to the spinner
/// * `insert_at_position` - Optional position to insert the spinner at in the multi-progress display
///
/// ## Returns
///
/// An Option containing the progress bar, or None if the cli-viz feature is not enabled
#[cfg(feature = "cli-viz")]
pub(crate) fn create_spinner(
    message: String,
    insert_at_position: Option<usize>,
    len: Option<u64>,
) -> ProgressBar {
    let pb = if let Some(len) = len {
        ProgressBar::new(len)
    } else {
        ProgressBar::new_spinner()
    };

    let pb = if let Some(pos) = insert_at_position {
        MULTI_PROGRESS.insert(pos, pb)
    } else {
        MULTI_PROGRESS.add(pb)
    };

    let style = if let Some(_) = len {
        ProgressStyle::with_template("{spinner} {msg} {pos:.bold} / {len:.dim}")
            .unwrap()
            .tick_strings(&*TICK_STRINGS)
    } else {
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
            .tick_strings(&*TICK_STRINGS)
    };

    pb.set_style(style);
    pb.set_message(message);
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

/// Finishes a spinner with an error mark (✗) instead of a checkmark.
/// Used for error paths to visually indicate failure.
///
/// ## Arguments
///
/// * `pb` - The progress bar to finish with an error mark
/// * `message` - The message to display next to the error mark
#[cfg(feature = "cli-viz")]
pub(crate) fn finish_with_error(pb: &ProgressBar, message: &str) {
    let style = ProgressStyle::with_template("{spinner} {msg}")
        .unwrap()
        .tick_strings(&*ERROR_TICK_STRINGS);

    pb.set_style(style);
    pb.finish_with_message(message.to_string());
}
