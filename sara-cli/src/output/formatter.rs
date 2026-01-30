//! Output formatting with colors and emojis.
//!
//! This module provides consistent output formatting for the CLI, including:
//!
//! - Colored text output (errors in red, success in green, etc.)
//! - Emoji prefixes for visual distinction
//! - Graceful degradation when colors/emojis are disabled
//!
//! # Configuration
//!
//! [`OutputConfig`] controls output behavior:
//! - `colors`: Enable/disable ANSI color codes
//! - `emojis`: Enable/disable emoji prefixes (falls back to text like `[OK]`)
//!
//! # Usage
//!
//! Use the helper functions like [`print_success`], [`print_error`], and
//! [`print_warning`] for consistent formatting across all commands.

use colored::Colorize;
use console::Emoji;

/// Output configuration.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub colors: bool,
    pub emojis: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            colors: true,
            emojis: true,
        }
    }
}

impl OutputConfig {
    /// Creates a new output config.
    pub fn new(colors: bool, emojis: bool) -> Self {
        Self { colors, emojis }
    }
}

// Common emojis
pub static EMOJI_SUCCESS: Emoji<'_, '_> = Emoji("✅", "[OK]");
pub static EMOJI_ERROR: Emoji<'_, '_> = Emoji("❌", "[ERR]");
pub static EMOJI_WARNING: Emoji<'_, '_> = Emoji("⚠️ ", "[WARN]");
pub static EMOJI_STATS: Emoji<'_, '_> = Emoji("📊", "[STATS]");
pub static EMOJI_ITEM: Emoji<'_, '_> = Emoji("📋", "[ITEM]");

/// Returns the appropriate emoji string based on config.
pub fn get_emoji<'a>(config: &OutputConfig, emoji: &Emoji<'a, 'a>) -> &'a str {
    if config.emojis { emoji.0 } else { emoji.1 }
}

/// Color options for text styling.
#[derive(Debug, Clone, Copy, Default)]
pub enum Color {
    #[default]
    None,
    Red,
    Green,
    Yellow,
    Cyan,
}

/// Style options for text styling.
#[derive(Debug, Clone, Copy, Default)]
pub enum Style {
    #[default]
    None,
    Bold,
    Dimmed,
}

/// Returns text with the specified color and/or style applied if colors are enabled.
pub fn colorize(config: &OutputConfig, text: &str, color: Color, style: Style) -> String {
    if !config.colors {
        return text.to_string();
    }

    // Apply color first
    let colored_text = match color {
        Color::Red => text.red(),
        Color::Green => text.green(),
        Color::Yellow => text.yellow(),
        Color::Cyan => text.cyan(),
        Color::None => text.normal(),
    };

    // Apply style
    match style {
        Style::Bold => colored_text.bold().to_string(),
        Style::Dimmed => colored_text.dimmed().to_string(),
        Style::None => colored_text.to_string(),
    }
}

/// Formats a message with emoji prefix based on config.
/// When emojis are disabled, the text prefix is colorized to match the message.
fn format_message(config: &OutputConfig, emoji: &Emoji, color: Color, message: &str) -> String {
    let prefix = get_emoji(config, emoji);
    // Only colorize the prefix when using text fallback (not emoji)
    let colored_prefix = if config.emojis {
        prefix.to_string()
    } else {
        colorize(config, prefix, color, Style::None)
    };
    format!("{} {}", colored_prefix, message)
}

/// Formats a success message.
pub fn format_success(config: &OutputConfig, message: &str) -> String {
    let msg = colorize(config, message, Color::Green, Style::None);
    format_message(config, &EMOJI_SUCCESS, Color::Green, &msg)
}

/// Prints a success message.
pub fn print_success(config: &OutputConfig, message: &str) {
    println!("{}", format_success(config, message));
}

/// Formats an error message.
pub fn format_error(config: &OutputConfig, message: &str) -> String {
    let msg = colorize(config, message, Color::Red, Style::None);
    format_message(config, &EMOJI_ERROR, Color::Red, &msg)
}

/// Prints an error message to stdout.
pub fn print_error(config: &OutputConfig, message: &str) {
    println!("{}", format_error(config, message));
}

/// Formats a warning message.
pub fn format_warning(config: &OutputConfig, message: &str) -> String {
    let msg = colorize(config, message, Color::Yellow, Style::None);
    format_message(config, &EMOJI_WARNING, Color::Yellow, &msg)
}

/// Prints a warning message.
pub fn print_warning(config: &OutputConfig, message: &str) {
    println!("{}", format_warning(config, message));
}

/// Prints a header message (bold if colors enabled).
pub fn print_header(config: &OutputConfig, message: &str) {
    println!("{}", colorize(config, message, Color::None, Style::Bold));
}

/// Formats a tree branch.
pub fn format_tree_branch(is_last: bool) -> &'static str {
    if is_last { "└─" } else { "├─" }
}
