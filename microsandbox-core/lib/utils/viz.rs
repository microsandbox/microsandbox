//! This module provides a global multi-progress bar for CLI visualizations.
//! It allows for tracking multiple progress bars in a single view.
//!
//! The `MULTI_PROGRESS` constant is a lazy-initialized `Arc<MultiProgress>` that
//! manages a collection of progress bars. It is used to display multiple progress
//! indicators simultaneously, such as when downloading multiple layers or

use indicatif::{MultiProgress, MultiProgressAlignment};
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

pub(crate) static TICK_STRINGS: LazyLock<[&str; 11]> =
    LazyLock::new(|| ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", &CHECKMARK]);
