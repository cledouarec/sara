//! Output formatting with colors and emojis.

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
pub static EMOJI_WARNING: Emoji<'_, '_> = Emoji("⚠️", "[WARN]");
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
fn format_message(config: &OutputConfig, emoji: &Emoji, message: &str) -> String {
    let prefix = get_emoji(config, emoji);
    format!("{} {}", prefix, message)
}

/// Prints a success message.
pub fn print_success(config: &OutputConfig, message: &str) {
    let msg = colorize(config, message, Color::Green, Style::None);
    println!("{}", format_message(config, &EMOJI_SUCCESS, &msg));
}

/// Prints an error message.
pub fn print_error(config: &OutputConfig, message: &str) {
    let msg = colorize(config, message, Color::Red, Style::None);
    eprintln!("{}", format_message(config, &EMOJI_ERROR, &msg));
}

/// Prints a warning message.
pub fn print_warning(config: &OutputConfig, message: &str) {
    let msg = colorize(config, message, Color::Yellow, Style::None);
    println!("{}", format_message(config, &EMOJI_WARNING, &msg));
}

/// Prints a header message (bold if colors enabled).
pub fn print_header(config: &OutputConfig, message: &str) {
    println!("{}", colorize(config, message, Color::None, Style::Bold));
}

/// Formats a tree branch.
pub fn format_tree_branch(is_last: bool) -> &'static str {
    if is_last { "└─" } else { "├─" }
}
