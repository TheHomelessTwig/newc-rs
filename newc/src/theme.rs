//! Central styling system for newc — palette-driven, follows the active theme.
//!
//! All views import style functions and colour accessors from this module
//! rather than hardcoding values inline. The public API falls into four
//! groups:
//!
//! - **[`color`]** — named colour accessors (`accent()`, `green()`, `text()`, …).
//! - **Container styles** — closures compatible with `container::style()`
//!   (`card_style`, `panel_style`, `code_block_style`, …).
//! - **Button styles** — closures compatible with `button::style()`
//!   (`btn_primary`, `btn_secondary`, `btn_ghost`, `btn_danger`, `btn_nav_*`).
//! - **Typography helpers** — pre-styled `Text` widgets (`heading`,
//!   `subheading`, `section_title`, `hint_text`, `mono`).
//!
//! Colours are resolved from the active [`iced::Theme`]'s extended palette by
//! [`set_theme`] into a process-wide snapshot ([`ResolvedColors`]); all windows
//! share one theme, and every theme change flows through `update()`, which
//! re-renders, so the snapshot is never stale. The hand-tuned Monokai Pro
//! palette is used verbatim when that theme is selected (it is the default).
#![allow(dead_code)]

use std::sync::RwLock;

use iced::theme::palette::{darken, mix};
use iced::widget::{button, container, text, text_input};
use iced::{Background, Border, Color, Font, Shadow};

// ── Resolved palette ─────────────────────────────────────────────────────────

/// A snapshot of every named colour, resolved from one [`iced::Theme`].
#[derive(Debug, Clone, Copy)]
pub struct ResolvedColors {
    pub bg_deep: Color,
    pub bg_base: Color,
    pub bg_panel: Color,
    pub bg_card: Color,
    pub bg_raised: Color,
    pub bg_hover: Color,
    pub bg_active: Color,
    pub border: Color,
    pub border_dim: Color,
    pub border_accent: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_hint: Color,
    pub accent: Color,
    /// Text colour readable on top of `accent` fills.
    pub on_accent: Color,
    pub green: Color,
    pub cyan: Color,
    pub yellow: Color,
    pub purple: Color,
    pub orange: Color,
    pub red: Color,
    pub is_dark: bool,
}

/// The hand-tuned Monokai Pro palette — the default look, used verbatim when
/// the "Monokai Pro" theme is active rather than the auto-generated palette.
const MONOKAI: ResolvedColors = ResolvedColors {
    bg_deep: Color::from_rgb8(0x1A, 0x17, 0x1C),
    bg_base: Color::from_rgb8(0x2D, 0x2A, 0x2E),
    bg_panel: Color::from_rgb8(0x26, 0x23, 0x27),
    bg_card: Color::from_rgb8(0x35, 0x32, 0x37),
    bg_raised: Color::from_rgb8(0x3A, 0x37, 0x3C),
    bg_hover: Color::from_rgb8(0x44, 0x41, 0x46),
    bg_active: Color::from_rgba8(0xFF, 0x61, 0x88, 0.196),
    border: Color::from_rgb8(0x48, 0x45, 0x4A),
    border_dim: Color::from_rgb8(0x3D, 0x3A, 0x3F),
    border_accent: Color::from_rgb8(0xFF, 0x61, 0x88),
    text: Color::from_rgb8(0xFC, 0xFC, 0xFA),
    text_dim: Color::from_rgb8(0x93, 0x90, 0x92),
    text_hint: Color::from_rgb8(0x65, 0x62, 0x66),
    accent: Color::from_rgb8(0xFF, 0x61, 0x88),
    on_accent: Color::WHITE,
    green: Color::from_rgb8(0xA9, 0xDC, 0x76),
    cyan: Color::from_rgb8(0x78, 0xDC, 0xE8),
    yellow: Color::from_rgb8(0xFF, 0xD8, 0x66),
    purple: Color::from_rgb8(0xAB, 0x9D, 0xF2),
    orange: Color::from_rgb8(0xFC, 0x96, 0x67),
    red: Color::from_rgb8(0xFF, 0x60, 0x60),
    is_dark: true,
};

static CURRENT: RwLock<ResolvedColors> = RwLock::new(MONOKAI);

impl ResolvedColors {
    fn from_theme(theme: &iced::Theme) -> Self {
        let p = theme.extended_palette();
        let base = p.background.base.color;
        let text = p.background.base.text;
        Self {
            bg_deep: darken(base, 0.03),
            bg_base: base,
            bg_panel: darken(base, 0.015),
            bg_card: p.background.weak.color,
            bg_raised: p.background.neutral.color,
            bg_hover: p.background.strong.color,
            bg_active: p.primary.base.color.scale_alpha(0.2),
            border: p.background.strongest.color,
            border_dim: p.background.neutral.color,
            border_accent: p.primary.base.color,
            text,
            text_dim: mix(text, base, 0.35),
            text_hint: mix(text, base, 0.6),
            accent: p.primary.base.color,
            on_accent: p.primary.base.text,
            green: p.success.base.color,
            yellow: p.warning.base.color,
            red: p.danger.base.color,
            // No palette slot for these — readable hand-picked pairs per mode
            cyan: if p.is_dark {
                Color::from_rgb8(0x78, 0xDC, 0xE8)
            } else {
                Color::from_rgb8(0x0E, 0x74, 0x90)
            },
            purple: if p.is_dark {
                Color::from_rgb8(0xAB, 0x9D, 0xF2)
            } else {
                Color::from_rgb8(0x6D, 0x28, 0xD9)
            },
            orange: if p.is_dark {
                Color::from_rgb8(0xFC, 0x96, 0x67)
            } else {
                Color::from_rgb8(0xC2, 0x41, 0x0C)
            },
            is_dark: p.is_dark,
        }
    }
}

/// Re-resolve the colour snapshot from `theme`. Call whenever the active
/// theme changes (startup, settings save, live theme pick).
pub fn set_theme(theme: &iced::Theme) {
    let resolved = if theme.to_string() == "Monokai Pro" {
        MONOKAI
    } else {
        ResolvedColors::from_theme(theme)
    };
    *CURRENT.write().unwrap() = resolved;
}

/// Current colour snapshot (cheap copy).
pub fn resolved() -> ResolvedColors {
    *CURRENT.read().unwrap()
}

/// Whether the active theme is dark.
pub fn is_dark() -> bool {
    resolved().is_dark
}

// ── Palette accessors ─────────────────────────────────────────────────────────

/// Named colour accessors resolved from the active theme.
///
/// Grouped by role: backgrounds (`bg_*`), borders (`border*`), text (`text*`),
/// and semantic accent colours (`accent`, `green`, `cyan`, `yellow`, `purple`, `orange`).
pub mod color {
    use iced::Color;

    use super::resolved;

    pub fn bg_deep() -> Color { resolved().bg_deep }
    pub fn bg_base() -> Color { resolved().bg_base }
    pub fn bg_panel() -> Color { resolved().bg_panel }
    pub fn bg_card() -> Color { resolved().bg_card }
    pub fn bg_raised() -> Color { resolved().bg_raised }
    pub fn bg_hover() -> Color { resolved().bg_hover }
    pub fn bg_active() -> Color { resolved().bg_active }

    pub fn border() -> Color { resolved().border }
    pub fn border_dim() -> Color { resolved().border_dim }
    pub fn border_accent() -> Color { resolved().border_accent }

    pub fn text() -> Color { resolved().text }
    pub fn text_dim() -> Color { resolved().text_dim }
    pub fn text_hint() -> Color { resolved().text_hint }

    pub fn accent() -> Color { resolved().accent }
    pub fn green() -> Color { resolved().green }
    pub fn cyan() -> Color { resolved().cyan }
    pub fn yellow() -> Color { resolved().yellow }
    pub fn purple() -> Color { resolved().purple }
    pub fn orange() -> Color { resolved().orange }
    pub fn red() -> Color { resolved().red }
}

// ── Separator ─────────────────────────────────────────────────────────────────

/// 1px horizontal divider.
pub fn separator<'a, Message: 'a>() -> iced::widget::Container<'a, Message> {
    container(iced::widget::Space::new().width(iced::Length::Fill).height(1))
        .style(|_| container::Style {
            background: Some(Background::Color(color::border_dim())),
            ..Default::default()
        })
}

// ── Container styles ──────────────────────────────────────────────────────────

/// Deepest background — used for the top bar and status bar.
pub fn deep_style(_: &iced::Theme) -> container::Style {
    let c = resolved();
    container::Style {
        background: Some(Background::Color(c.bg_deep)),
        border: Border { color: c.border_dim, width: 0.0, radius: 0.0.into() },
        ..Default::default()
    }
}

/// Sidebar and detached-window background.
pub fn panel_style(_: &iced::Theme) -> container::Style {
    let c = resolved();
    container::Style {
        background: Some(Background::Color(c.bg_panel)),
        border: Border { color: c.border, width: 1.0, radius: 0.0.into() },
        ..Default::default()
    }
}

/// Slightly elevated card surface with a 6 px corner radius.
pub fn card_style(_: &iced::Theme) -> container::Style {
    let c = resolved();
    container::Style {
        background: Some(Background::Color(c.bg_card)),
        border: Border { color: c.border, width: 1.0, radius: 6.0.into() },
        ..Default::default()
    }
}

/// Card with a drop shadow — used for modal-like elements.
pub fn card_raised_style(_: &iced::Theme) -> container::Style {
    let c = resolved();
    container::Style {
        background: Some(Background::Color(c.bg_raised)),
        border: Border { color: c.border, width: 1.0, radius: 6.0.into() },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if c.is_dark { 0.4 } else { 0.15 }),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}

/// Section background with a 4 px radius — lighter border than `card_style`.
pub fn section_style(_: &iced::Theme) -> container::Style {
    let c = resolved();
    container::Style {
        background: Some(Background::Color(c.bg_card)),
        border: Border { color: c.border_dim, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

/// Card with a 3 px accent-coloured left border — used for selected/active rows.
pub fn accent_left_border(_: &iced::Theme) -> container::Style {
    let c = resolved();
    container::Style {
        background: Some(Background::Color(c.bg_card)),
        border: Border { color: c.accent, width: 3.0, radius: 0.0.into() },
        ..Default::default()
    }
}

/// Translucent accent tint used for selected list rows.
pub fn selected_row_style(_: &iced::Theme) -> container::Style {
    let c = resolved();
    container::Style {
        background: Some(Background::Color(c.accent.scale_alpha(0.1))),
        border: Border { color: c.accent, width: 1.0, radius: 3.0.into() },
        ..Default::default()
    }
}

/// Even-row background for zebra-striped tables.
pub fn stripe_even(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::bg_card())),
        ..Default::default()
    }
}

/// Odd-row background for zebra-striped tables.
pub fn stripe_odd(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color::bg_base())),
        ..Default::default()
    }
}

/// Code block background — deep with a dim border and 4 px radius.
pub fn code_block_style(_: &iced::Theme) -> container::Style {
    let c = resolved();
    container::Style {
        background: Some(Background::Color(c.bg_deep)),
        border: Border { color: c.border_dim, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

// ── Button styles ─────────────────────────────────────────────────────────────

/// Filled accent-coloured button — primary actions (save, submit).
pub fn btn_primary(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let c = resolved();
    let bg = match status {
        button::Status::Active   => c.accent,
        button::Status::Hovered  => c.accent.scale_alpha(0.85),
        button::Status::Pressed  => c.accent.scale_alpha(0.70),
        button::Status::Disabled => c.accent.scale_alpha(0.35),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: c.on_accent,
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

/// Outline button with no background by default — secondary actions.
pub fn btn_secondary(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let c = resolved();
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Some(Background::Color(c.bg_hover)),
        _ => None,
    };
    let border_color = match status {
        button::Status::Hovered | button::Status::Pressed => c.border,
        _ => c.border_dim,
    };
    button::Style {
        background: bg,
        text_color: if matches!(status, button::Status::Disabled) { c.text_hint } else { c.text },
        border: Border { color: border_color, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

/// Borderless button — only shows a background tint on hover/press.
pub fn btn_ghost(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let c = resolved();
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Some(Background::Color(c.bg_hover)),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: if matches!(status, button::Status::Disabled) { c.text_hint } else { c.text_dim },
        border: Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    }
}

/// Red-tinted button for destructive actions (delete, remove).
pub fn btn_danger(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let c = resolved();
    let bg = match status {
        button::Status::Active   => Some(Background::Color(c.red.scale_alpha(0.12))),
        button::Status::Hovered  => Some(Background::Color(c.red.scale_alpha(0.22))),
        button::Status::Pressed  => Some(Background::Color(c.red.scale_alpha(0.31))),
        button::Status::Disabled => None,
    };
    button::Style {
        background: bg,
        text_color: c.red,
        border: Border { color: c.red, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

/// Top-bar navigation button in its active (current view) state.
pub fn btn_nav_active(
    _theme: &iced::Theme,
    _status: button::Status,
) -> button::Style {
    let c = resolved();
    button::Style {
        background: Some(Background::Color(c.accent.scale_alpha(0.14))),
        text_color: c.accent,
        border: Border { color: c.accent, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    }
}

/// Top-bar navigation button in its inactive (other view) state.
pub fn btn_nav_inactive(
    _theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let c = resolved();
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Some(Background::Color(c.bg_hover)),
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: c.text_dim,
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
    let c = resolved();
    let border_color = match status {
        text_input::Status::Focused { .. } => c.accent,
        text_input::Status::Hovered => c.border,
        _ => c.border_dim,
    };
    text_input::Style {
        background: Background::Color(c.bg_card),
        border: Border { color: border_color, width: 1.0, radius: 4.0.into() },
        icon: c.text_dim,
        placeholder: c.text_hint,
        value: c.text,
        selection: c.accent.scale_alpha(0.25),
    }
}

// ── Typography helpers ────────────────────────────────────────────────────────

/// 20 px view heading text in the default text colour.
pub fn heading(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(20).color(color::text())
}

/// 15 px subheading in dim text colour.
pub fn subheading(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(15).color(color::text_dim())
}

/// 13 px section label in cyan — used for sidebar section headings.
pub fn section_title(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(13).color(color::cyan())
}

/// 12 px body label in the default text colour.
pub fn label_text(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(12).color(color::text())
}

/// 11 px hint text in the faintest hint colour — used for empty-state messages.
pub fn hint_text(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(11).color(color::text_hint())
}

/// 12 px monospace text in the default text colour.
pub fn mono(s: impl ToString) -> iced::widget::Text<'static> {
    text(s.to_string()).size(12).font(Font::MONOSPACE).color(color::text())
}

// ── Toast helpers ─────────────────────────────────────────────────────────────

/// Container style for a [`crate::state::Toast`] notification.
///
/// The border colour and translucent background tint vary by [`crate::state::ToastKind`]:
/// green for success, red for error, cyan for info.
pub fn toast_style(kind: &crate::state::ToastKind) -> container::Style {
    let c = resolved();
    let border_color = match kind {
        crate::state::ToastKind::Success => c.green,
        crate::state::ToastKind::Error => c.red,
        crate::state::ToastKind::Info => c.cyan,
    };
    container::Style {
        background: Some(Background::Color(border_color.scale_alpha(0.07))),
        border: Border { color: border_color, width: 2.0, radius: 6.0.into() },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, if c.is_dark { 0.5 } else { 0.2 }),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}
