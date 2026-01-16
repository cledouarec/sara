//! Output formatting modules.

mod formatter;
pub mod progress;

pub use formatter::{
    Color, EMOJI_ERROR, EMOJI_ITEM, EMOJI_STATS, EMOJI_WARNING, OutputConfig, Style, colorize,
    format_tree_branch, get_emoji, print_error, print_header, print_success, print_warning,
};
