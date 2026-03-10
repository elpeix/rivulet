use ratatui::style::Modifier;
use ratatui::style::{Color, Style};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub header_bg: Color,
    pub border: Color,
    pub focus_border: Color,
    pub focus_title: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub focus_bg: Color,
    pub block_bg: Color,
    pub feeds_bg: Color,
    pub preview_bg: Color,
    pub text: Color,
    pub dim: Color,
    pub status_ok: Color,
    pub status_err: Color,
    pub accent: Color,
    pub accent_alt: Color,
    pub selection_bg: Color,
}

#[derive(Debug, Deserialize)]
struct ThemeFile {
    header_bg: String,
    border: String,
    focus_border: String,
    focus_title: String,
    highlight_bg: String,
    highlight_fg: String,
    focus_bg: String,
    block_bg: String,
    feeds_bg: String,
    preview_bg: String,
    text: String,
    dim: String,
    status_ok: String,
    status_err: String,
    accent: String,
    accent_alt: String,
    selection_bg: Option<String>,
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    // Hex: #RRGGBB
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
        return None;
    }
    // Named terminal colors
    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" | "dark_gray" | "dark_grey" => Some(Color::DarkGray),
        "lightred" | "light_red" => Some(Color::LightRed),
        "lightgreen" | "light_green" => Some(Color::LightGreen),
        "lightyellow" | "light_yellow" => Some(Color::LightYellow),
        "lightblue" | "light_blue" => Some(Color::LightBlue),
        "lightmagenta" | "light_magenta" => Some(Color::LightMagenta),
        "lightcyan" | "light_cyan" => Some(Color::LightCyan),
        "reset" => Some(Color::Reset),
        _ => None,
    }
}

fn themes_dir() -> Option<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .ok()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))?;
    Some(base.join("rivulet").join("themes"))
}

impl ThemeFile {
    fn into_theme(self) -> Option<Theme> {
        Some(Theme {
            header_bg: parse_color(&self.header_bg)?,
            border: parse_color(&self.border)?,
            focus_border: parse_color(&self.focus_border)?,
            focus_title: parse_color(&self.focus_title)?,
            highlight_bg: parse_color(&self.highlight_bg)?,
            highlight_fg: parse_color(&self.highlight_fg)?,
            focus_bg: parse_color(&self.focus_bg)?,
            block_bg: parse_color(&self.block_bg)?,
            feeds_bg: parse_color(&self.feeds_bg)?,
            preview_bg: parse_color(&self.preview_bg)?,
            text: parse_color(&self.text)?,
            dim: parse_color(&self.dim)?,
            status_ok: parse_color(&self.status_ok)?,
            status_err: parse_color(&self.status_err)?,
            accent: parse_color(&self.accent)?,
            accent_alt: parse_color(&self.accent_alt)?,
            selection_bg: self
                .selection_bg
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(Color::Rgb(50, 50, 70)),
        })
    }
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        // Try loading from TOML file first
        if let Some(theme) = Self::load_from_file(name) {
            return theme;
        }
        // Fall back to built-in themes
        match name {
            "light" => Self::light(),
            "terminal" => Self::terminal(),
            _ => Self::dark(),
        }
    }

    fn load_from_file(name: &str) -> Option<Self> {
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            log::warn!("Invalid theme name (only alphanumeric, '-', '_' allowed): {name}");
            return None;
        }
        let dir = themes_dir()?;
        let path = dir.join(format!("{name}.toml"));
        let contents = std::fs::read_to_string(&path).ok()?;
        match toml::from_str::<ThemeFile>(&contents) {
            Ok(tf) => match tf.into_theme() {
                Some(theme) => Some(theme),
                None => {
                    log::warn!("Invalid color in theme file: {}", path.display());
                    None
                }
            },
            Err(e) => {
                log::warn!("Failed to parse theme {}: {}", path.display(), e);
                None
            }
        }
    }

    pub fn dark() -> Self {
        Self {
            header_bg: Color::Rgb(31, 31, 40),
            border: Color::Rgb(84, 84, 109),
            focus_border: Color::Rgb(126, 156, 216),
            focus_title: Color::Rgb(220, 215, 186),
            highlight_bg: Color::Rgb(45, 79, 103),
            highlight_fg: Color::Rgb(220, 215, 186),
            focus_bg: Color::Rgb(26, 26, 34),
            block_bg: Color::Rgb(24, 24, 32),
            feeds_bg: Color::Rgb(29, 29, 39),
            preview_bg: Color::Rgb(22, 22, 30),
            text: Color::Rgb(220, 215, 186),
            dim: Color::Rgb(140, 140, 161),
            status_ok: Color::Rgb(152, 187, 108),
            status_err: Color::Rgb(228, 104, 118),
            accent: Color::Rgb(126, 156, 216),
            accent_alt: Color::Rgb(220, 165, 97),
            selection_bg: Color::Rgb(50, 50, 70),
        }
    }

    pub fn light() -> Self {
        Self {
            header_bg: Color::Rgb(218, 215, 205),
            border: Color::Rgb(160, 155, 145),
            focus_border: Color::Rgb(30, 70, 150),
            focus_title: Color::Rgb(20, 20, 30),
            highlight_bg: Color::Rgb(50, 100, 180),
            highlight_fg: Color::Rgb(255, 255, 255),
            focus_bg: Color::Rgb(238, 235, 225),
            block_bg: Color::Rgb(245, 242, 232),
            feeds_bg: Color::Rgb(235, 232, 222),
            preview_bg: Color::Rgb(248, 245, 237),
            text: Color::Rgb(25, 25, 35),
            dim: Color::Rgb(90, 88, 80),
            status_ok: Color::Rgb(40, 120, 30),
            status_err: Color::Rgb(180, 30, 40),
            accent: Color::Rgb(20, 70, 150),
            accent_alt: Color::Rgb(160, 85, 10),
            selection_bg: Color::Rgb(200, 210, 230),
        }
    }

    pub fn terminal() -> Self {
        Self {
            header_bg: Color::Reset,
            border: Color::DarkGray,
            focus_border: Color::Blue,
            focus_title: Color::White,
            highlight_bg: Color::Blue,
            highlight_fg: Color::White,
            focus_bg: Color::Reset,
            block_bg: Color::Reset,
            feeds_bg: Color::Reset,
            preview_bg: Color::Reset,
            text: Color::Reset,
            dim: Color::DarkGray,
            status_ok: Color::Green,
            status_err: Color::Red,
            accent: Color::Cyan,
            accent_alt: Color::Yellow,
            selection_bg: Color::DarkGray,
        }
    }

    pub fn section_title_style(&self, focused: bool) -> Style {
        let color = if focused { self.accent } else { self.dim };
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn focus_border_style(&self) -> Style {
        Style::default().fg(self.focus_border)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.highlight_fg)
            .bg(self.highlight_bg)
            .add_modifier(Modifier::empty())
    }

    pub fn focus_title_style(&self) -> Style {
        Style::default()
            .fg(self.focus_title)
            .add_modifier(Modifier::BOLD)
    }

    pub fn focus_block_style(&self) -> Style {
        Style::default().bg(self.focus_bg)
    }

    pub fn block_style(&self) -> Style {
        Style::default().bg(self.block_bg)
    }

    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color() {
        assert_eq!(parse_color("#1F1F28"), Some(Color::Rgb(31, 31, 40)));
        assert_eq!(parse_color("#FFFFFF"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(parse_color("#000000"), Some(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn parse_named_colors() {
        assert_eq!(parse_color("blue"), Some(Color::Blue));
        assert_eq!(parse_color("Reset"), Some(Color::Reset));
        assert_eq!(parse_color("dark_gray"), Some(Color::DarkGray));
        assert_eq!(parse_color("DarkGray"), Some(Color::DarkGray));
        assert_eq!(parse_color("light_red"), Some(Color::LightRed));
    }

    #[test]
    fn parse_invalid_color() {
        assert_eq!(parse_color("#GG0000"), None);
        assert_eq!(parse_color("#12345"), None);
        assert_eq!(parse_color("nope"), None);
    }

    #[test]
    fn theme_file_roundtrip() {
        let toml = r##"
            header_bg = "#1F1F28"
            border = "#54546D"
            focus_border = "blue"
            focus_title = "#DCD7BA"
            highlight_bg = "#2D4F67"
            highlight_fg = "#DCD7BA"
            focus_bg = "#1A1A22"
            block_bg = "#181820"
            feeds_bg = "#1D1D27"
            preview_bg = "#16161E"
            text = "#DCD7BA"
            dim = "dark_gray"
            status_ok = "#98BB6C"
            status_err = "#E46876"
            accent = "cyan"
            accent_alt = "#DCA561"
        "##;
        let tf: ThemeFile = toml::from_str(toml).unwrap();
        let theme = tf.into_theme().unwrap();
        assert_eq!(theme.header_bg, Color::Rgb(31, 31, 40));
        assert_eq!(theme.focus_border, Color::Blue);
        assert_eq!(theme.dim, Color::DarkGray);
        assert_eq!(theme.accent, Color::Cyan);
    }

    #[test]
    fn theme_file_invalid_color_returns_none() {
        let toml = r##"
            header_bg = "invalid"
            border = "#54546D"
            focus_border = "#7E9CD8"
            focus_title = "#DCD7BA"
            highlight_bg = "#2D4F67"
            highlight_fg = "#DCD7BA"
            focus_bg = "#1A1A22"
            block_bg = "#181820"
            feeds_bg = "#1D1D27"
            preview_bg = "#16161E"
            text = "#DCD7BA"
            dim = "#8C8CA1"
            status_ok = "#98BB6C"
            status_err = "#E46876"
            accent = "#7E9CD8"
            accent_alt = "#DCA561"
        "##;
        let tf: ThemeFile = toml::from_str(toml).unwrap();
        assert!(tf.into_theme().is_none());
    }

    #[test]
    fn builtin_themes_load() {
        let dark = Theme::from_name("dark");
        assert_eq!(dark.header_bg, Color::Rgb(31, 31, 40));
        let light = Theme::from_name("light");
        assert_eq!(light.text, Color::Rgb(25, 25, 35));
        let term = Theme::from_name("terminal");
        assert_eq!(term.accent, Color::Cyan);
    }
}
