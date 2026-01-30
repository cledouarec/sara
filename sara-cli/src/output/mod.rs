//! Output formatting modules.

mod formatter;

pub use formatter::{
    Color, EMOJI_ERROR, EMOJI_ITEM, EMOJI_STATS, EMOJI_WARNING, OutputConfig, Style, colorize,
    format_error, format_success, format_tree_branch, format_warning, get_emoji, print_error,
    print_header, print_success, print_warning,
};
