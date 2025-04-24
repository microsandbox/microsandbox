//! Module containing terminal utilities

use indicatif::{MultiProgress, MultiProgressAlignment, ProgressBar, ProgressStyle};
use std::sync::{Arc, LazyLock};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// The multi-progress bar for CLI visualizations
pub static MULTI_PROGRESS: LazyLock<Arc<MultiProgress>> = LazyLock::new(|| {
    let mp = MultiProgress::new();
    mp.set_alignment(MultiProgressAlignment::Top);
    Arc::new(mp)
});

/// The checkmark for CLI visualizations
pub static CHECKMARK: LazyLock<String> =
    LazyLock::new(|| format!("{}", console::style("✓").green()));

/// The error mark  for CLI visualizations
pub static ERROR_MARK: LazyLock<String> =
    LazyLock::new(|| format!("{}", console::style("✗").red()));

/// The tick strings for CLI visualizations
pub static TICK_STRINGS: LazyLock<[&str; 11]> =
    LazyLock::new(|| ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", &CHECKMARK]);

/// The error tick strings for CLI visualizations
pub static ERROR_TICK_STRINGS: LazyLock<[&str; 2]> = LazyLock::new(|| ["⠏", &ERROR_MARK]);

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Determines if the process is running in an interactive terminal environment
pub fn is_interactive_terminal() -> bool {
    // Check if stdin and stdout are TTYs
    let stdin_is_tty = unsafe { libc::isatty(libc::STDIN_FILENO) == 1 };
    let stdout_is_tty = unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 };

    // Base check: both stdin and stdout must be TTYs
    let is_tty = stdin_is_tty && stdout_is_tty;

    // Optional enhancement: check for TERM, but don't require it
    let has_term = std::env::var("TERM").is_ok();

    // Log the detection for debugging
    if is_tty && !has_term {
        tracing::debug!("detected TTY without TERM environment variable");
    }

    // Return true if we have TTYs, regardless of TERM
    is_tty
}

/// Determines if the process is running in an ANSI terminal environment
pub fn is_ansi_interactive_terminal() -> bool {
    is_interactive_terminal() && !std::env::var("TERM").unwrap_or_default().contains("dumb")
}

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
/// An Option containing the progress bar, or None if the cli feature is not enabled
pub fn create_spinner(
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
pub fn finish_with_error(pb: &ProgressBar) {
    let style = ProgressStyle::with_template("{spinner} {msg}")
        .unwrap()
        .tick_strings(&*ERROR_TICK_STRINGS);

    pb.set_style(style);
    pb.finish();
}
