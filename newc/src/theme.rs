// Central styling system for newc — Monokai Pro dark palette.
// All views import from here rather than hardcoding colors inline.
#![allow(dead_code)]

use iced::widget::{button, container, text, text_input};
use iced::{Background, Border, Color, Font, Shadow};

// ── Palette ───────────────────────────────────────────────────────────────────

pub mod color {
    use iced::Color;

    pub const BG_DEEP:    Color = Color::from_rgb8(0x1A, 0x17, 0x1C);
    pub const BG_BASE:    Color = Color::from_rgb8(0x2D, 0x2A, 0x2E);
    pub const BG_PANEL:   Color = Color::from_rgb8(0x26, 0x23, 0x27);
    pub const BG_CARD:    Color = Color::from_rgb8(0x35, 0x32, 0x37);
    pub const BG_RAISED:  Color = Color::from_rgb8(0x3A, 0x37, 0x3C);
    pub const BG_HOVER:   Color = Color::from_rgb8(0x44, 0x41, 0x46);
    pub const BG_ACTIVE:  Color = Color::from_rgba8(0xFF, 0x61, 0x88, 0.196);

    pub const BORDER:     Color = Color::from_rgb8(0x48, 0x45, 0x4A);
    pub const BORDER_DIM: Color = Color::from_rgb8(0x3D, 0x3A, 0x3F);
    pub const BORDER_ACCENT: Color = Color::from_rgb8(0xFF, 0x61, 0x88);

    pub const TEXT:       Color = Color::from_rgb8(0xFC, 0xFC, 0xFA);
    pub const TEXT_DIM:   Color = Color::from_rgb8(0x93, 0x90, 0x92);
    pub const TEXT_HINT:  Color = Color::from_rgb8(0x65, 0x62, 0x66);

    pub const ACCENT:     Color = Color::from_rgb8(0xFF, 0x61, 0x88);
    pub const GREEN:      Color = Color::from_rgb8(0xA9, 0xDC, 0x76);
    pub const CYAN:       Color = Color::from_rgb8(0x78, 0xDC, 0xE8);
    pub const YELLOW:     Color = Color::from_rgb8(0xFF, 0xD8, 0x66);
    pub const PURPLE:     Color = Color::from_rgb8(0xAB, 0x9D, 0xF2);
    pub const ORANGE:     Color = Color::from_rgb8(0xFC, 0x96, 0x67);
}

// ── Separator ─────────────────────────────────────────────────────────────────

/// 1px horizontal divider.
pub fn separator<'a, Message: 'a>() -> iced::widget::Container<'a, Message> {
    container(iced::widget::Space::new().width(iced::Length::Fill).height(1))
        .style(|_| container::Style {
            background: Some(Background::Color(color::BORDER_DIM)),
            ..Default::default()
        })
}

// ── Container styles ──────────────────────────────────────────────────────────

pub fn deep_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_DEEP)),
        border: Border { color: color::BORDER_DIM, width: 0.0, radius: 0.0.into() },
        ..Default::default()
    }
}

pub fn panel_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_PANEL)),
        border: Border { color: color::BORDER, width: 1.0, radius: 0.0.into() },
        ..Default::default()
    }
}

pub fn card_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_CARD)),
        border: Border { color: color::BORDER, width: 1.0, radius: 6.0.into() },
        ..Default::default()
    }
}

pub fn card_raised_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_RAISED)),
        border: Border { color: color::BORDER, width: 1.0, radius: 6.0.into() },
        shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.4), offset: iced::Vector::new(0.0, 2.0), blur_radius: 8.0 },
        ..Default::default()
    }
}

pub fn section_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_CARD)),
        border: Border { color: color::BORDER_DIM, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

pub fn accent_left_border(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_CARD)),
        border: Border { color: color::ACCENT, width: 3.0, radius: 0.0.into() },
        ..Default::default()
    }
}

pub fn selected_row_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(0xFF, 0x61, 0x88, 0.098))),
        border: Border { color: color::ACCENT, width: 1.0, radius: 3.0.into() },
        ..Default::default()
    }
}

pub fn stripe_even(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_CARD)),
        ..Default::default()
    }
}

pub fn stripe_odd(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_BASE)),
        ..Default::default()
    }
}

// ── Button styles ─────────────────────────────────────────────────────────────

pub fn btn_primary(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let bg = match status {
        button::Status::Active   => color::ACCENT,
        button::Status::Hovered  => Color { a: 0.85, ..color::ACCENT },
        button::Status::Pressed  => Color { a: 0.70, ..color::ACCENT },
        button::Status::Disabled => Color { a: 0.35, ..color::ACCENT },
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

pub fn btn_secondary(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Some(Background::Color(color::BG_HOVER)),
        _ => None,
    };
    let border_color = match status {
        button::Status::Hovered | button::Status::Pressed => color::BORDER,
        button::Status::Disabled => color::BORDER_DIM,
        _ => color::BORDER_DIM,
    };
    button::Style {
        background: bg,
        text_color: if matches!(status, button::Status::Disabled) { color::TEXT_HINT } else { color::TEXT },
        border: Border { color: border_color, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

pub fn btn_ghost(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Some(Background::Color(color::BG_HOVER)),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: if matches!(status, button::Status::Disabled) { color::TEXT_HINT } else { color::TEXT_DIM },
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

pub fn btn_danger(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let accent = Color::from_rgb8(0xFF, 0x60, 0x60);
    let bg = match status {
        button::Status::Active   => Some(Background::Color(Color::from_rgba8(0xFF, 0x60, 0x60, 0.118))),
        button::Status::Hovered  => Some(Background::Color(Color::from_rgba8(0xFF, 0x60, 0x60, 0.216))),
        button::Status::Pressed  => Some(Background::Color(Color::from_rgba8(0xFF, 0x60, 0x60, 0.314))),
        button::Status::Disabled => None,
    };
    button::Style {
        background: bg,
        text_color: accent,
        border: Border { color: accent, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

pub fn btn_nav_active(
    _theme: &iced::Theme,
    _status: button::Status,
) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgba8(0xFF, 0x61, 0x88, 0.137))),
        text_color: color::ACCENT,
        border: Border { color: color::ACCENT, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

pub fn btn_nav_inactive(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Some(Background::Color(color::BG_HOVER)),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: color::TEXT_DIM,
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

// ── Text input styles ─────────────────────────────────────────────────────────

pub fn input_style(
    _theme: &iced::Theme,
    status: text_input::Status,
) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => color::ACCENT,
        text_input::Status::Hovered => color::BORDER,
        _ => color::BORDER_DIM,
    };
    text_input::Style {
        background: Background::Color(color::BG_CARD),
        border: Border { color: border_color, width: 1.0, radius: 4.0.into() },
        icon: color::TEXT_DIM,
        placeholder: color::TEXT_HINT,
        value: color::TEXT,
        selection: Color::from_rgba8(0xFF, 0x61, 0x88, 0.235),
    }
}

// ── Typography helpers ────────────────────────────────────────────────────────

pub fn heading(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(20).color(color::TEXT)
}

pub fn subheading(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(15).color(color::TEXT_DIM)
}

pub fn section_title(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(13).color(color::CYAN)
}

pub fn label_text(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(12).color(color::TEXT)
}

pub fn hint_text(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(11).color(color::TEXT_HINT)
}

pub fn mono(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(12).font(Font::MONOSPACE).color(color::TEXT)
}

// ── Toast helpers ─────────────────────────────────────────────────────────────

pub fn toast_style(kind: &crate::state::ToastKind) -> container::Style {
    let (border_color, bg) = match kind {
        crate::state::ToastKind::Success => (color::GREEN, Color::from_rgba8(0xA9, 0xDC, 0x76, 0.071)),
        crate::state::ToastKind::Error   => (color::ACCENT, Color::from_rgba8(0xFF, 0x61, 0x88, 0.071)),
        crate::state::ToastKind::Info    => (color::CYAN, Color::from_rgba8(0x78, 0xDC, 0xE8, 0.071)),
    };
    container::Style {
        background: Some(Background::Color(bg)),
        border: Border { color: border_color, width: 2.0, radius: 6.0.into() },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}
