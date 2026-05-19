//! Central styling system for newc — Monokai Pro dark palette.
//!
//! All views import style functions and colour constants from this module
//! rather than hardcoding values inline. The public API falls into four
//! groups:
//!
//! - **[`color`]** — named colour constants (`ACCENT`, `GREEN`, `TEXT`, …).
//! - **Container styles** — closures compatible with `container::style()`
//!   (`card_style`, `panel_style`, `code_block_style`, …).
//! - **Button styles** — closures compatible with `button::style()`
//!   (`btn_primary`, `btn_secondary`, `btn_ghost`, `btn_danger`, `btn_nav_*`).
//! - **Typography helpers** — pre-styled `Text` widgets (`heading`,
//!   `subheading`, `section_title`, `hint_text`, `mono`).
//!
//! All style functions take an `&iced::Theme` argument (unused — the palette is
//! hardcoded) so they satisfy the iced styling trait bounds.
#![allow(dead_code)]

use iced::widget::{button, container, text, text_input};
use iced::{Background, Border, Color, Font, Shadow};

// ── Palette ───────────────────────────────────────────────────────────────────

/// Named colour constants from the Monokai Pro palette.
///
/// Grouped by role: backgrounds (`BG_*`), borders (`BORDER*`), text (`TEXT*`),
/// and semantic accent colours (`ACCENT`, `GREEN`, `CYAN`, `YELLOW`, `PURPLE`, `ORANGE`).
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

/// Deepest background — used for the top bar and status bar.
pub fn deep_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_DEEP)),
        border: Border { color: color::BORDER_DIM, width: 0.0, radius: 0.0.into() },
        ..Default::default()
    }
}

/// Sidebar and detached-window background.
pub fn panel_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_PANEL)),
        border: Border { color: color::BORDER, width: 1.0, radius: 0.0.into() },
        ..Default::default()
    }
}

/// Slightly elevated card surface with a 6 px corner radius.
pub fn card_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_CARD)),
        border: Border { color: color::BORDER, width: 1.0, radius: 6.0.into() },
        ..Default::default()
    }
}

/// Card with a drop shadow — used for modal-like elements.
pub fn card_raised_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_RAISED)),
        border: Border { color: color::BORDER, width: 1.0, radius: 6.0.into() },
        shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.4), offset: iced::Vector::new(0.0, 2.0), blur_radius: 8.0 },
        ..Default::default()
    }
}

/// Section background with a 4 px radius — lighter border than `card_style`.
pub fn section_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_CARD)),
        border: Border { color: color::BORDER_DIM, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

/// Card with a 3 px accent-coloured left border — used for selected/active rows.
pub fn accent_left_border(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_CARD)),
        border: Border { color: color::ACCENT, width: 3.0, radius: 0.0.into() },
        ..Default::default()
    }
}

/// Translucent accent tint used for selected list rows.
pub fn selected_row_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(0xFF, 0x61, 0x88, 0.098))),
        border: Border { color: color::ACCENT, width: 1.0, radius: 3.0.into() },
        ..Default::default()
    }
}

/// Even-row background for zebra-striped tables.
pub fn stripe_even(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_CARD)),
        ..Default::default()
    }
}

/// Odd-row background for zebra-striped tables.
pub fn stripe_odd(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_BASE)),
        ..Default::default()
    }
}

/// Code block background — deep dark with a dim border and 4 px radius.
pub fn code_block_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::BG_DEEP)),
        border: Border { color: color::BORDER_DIM, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

// ── Button styles ─────────────────────────────────────────────────────────────

/// Filled accent-coloured button — primary actions (save, submit).
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

/// Outline button with no background by default — secondary actions.
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

/// Borderless button — only shows a background tint on hover/press.
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

/// Red-tinted button for destructive actions (delete, remove).
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

/// Top-bar navigation button in its active (current view) state.
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

/// Top-bar navigation button in its inactive (other view) state.
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

/// Uniform text-input style: card background, dim border that turns accent on focus.
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

/// 20 px view heading text in the default text colour.
pub fn heading(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(20).color(color::TEXT)
}

/// 15 px subheading in dim text colour.
pub fn subheading(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(15).color(color::TEXT_DIM)
}

/// 13 px section label in cyan — used for sidebar section headings.
pub fn section_title(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(13).color(color::CYAN)
}

/// 12 px body label in the default text colour.
pub fn label_text(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(12).color(color::TEXT)
}

/// 11 px hint text in the faintest hint colour — used for empty-state messages.
pub fn hint_text(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(11).color(color::TEXT_HINT)
}

/// 12 px monospace text in the default text colour.
pub fn mono(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(12).font(Font::MONOSPACE).color(color::TEXT)
}

// ── Toast helpers ─────────────────────────────────────────────────────────────

/// Container style for a [`crate::state::Toast`] notification.
///
/// The border colour and translucent background tint vary by [`crate::state::ToastKind`]:
/// green for success, accent-red for error, cyan for info.
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
