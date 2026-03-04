use ratatui::style::Modifier;
use ratatui::style::{Color, Style};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub header_bg: Color,
    pub header_fg: Color,
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
}

impl Theme {
    pub fn default() -> Self {
        Self {
            header_bg: Color::Rgb(31, 31, 40),
            header_fg: Color::Rgb(220, 215, 186),
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

    pub fn header_style(&self) -> Style {
        Style::default().fg(self.header_fg).bg(self.header_bg)
    }

    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.dim)
    }
}
