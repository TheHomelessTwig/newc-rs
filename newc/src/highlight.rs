//! Syntax highlighting for C source code.
//!
//! [`highlight_c`] tokenizes a C source string into coloured [`Span`]s using a
//! mode-aware palette (Monokai Pro on dark themes, a standard light scheme on
//! light themes). [`code_view`] wraps those spans in a scrollable iced widget
//! suitable for the module-detail and header-editor panels.
//!
//! The highlighter is a single-pass byte scanner: it handles preprocessor
//! lines, block/line comments, string/char literals, numeric literals,
//! keywords (type and flow), function calls (identifier followed by `(`), and
//! operators. It does not parse the full C grammar.

use crate::state::Message;
use crate::theme as th;
use iced::widget::text::Span as TSpan;
use iced::widget::{column, container, rich_text, row, scrollable, text};
use iced::{Color, Element, Length};

/// A contiguous run of text with a single display colour.
///
/// Produced by [`highlight_c`] and consumed by [`code_view`].
#[derive(Debug, Clone)]
pub struct Span {
    pub text: String,
    pub color: Color,
}

/// Token colours for one display mode (dark or light).
struct CodePalette {
    type_kw: Color,
    flow_kw: Color,
    func_name: Color,
    preproc: Color,
    string: Color,
    number: Color,
    comment: Color,
    operator: Color,
    default: Color,
    gutter: Color,
}

// Monokai Pro — used on dark themes
const DARK_PALETTE: CodePalette = CodePalette {
    type_kw: Color::from_rgb(0.471, 0.863, 0.910), // #78DCE8 cyan
    flow_kw: Color::from_rgb(1.000, 0.380, 0.533), // #FF6188 coral
    func_name: Color::from_rgb(0.663, 0.863, 0.463), // #A9DC76 green
    preproc: Color::from_rgb(1.000, 0.380, 0.533), // #FF6188 coral
    string: Color::from_rgb(1.000, 0.847, 0.400),  // #FFD866 yellow
    number: Color::from_rgb(0.671, 0.616, 0.949),  // #AB9DF2 lavender
    comment: Color::from_rgb(0.447, 0.439, 0.447), // #727072 gray
    operator: Color::from_rgb(1.000, 0.380, 0.533), // #FF6188 coral
    default: Color::from_rgb(0.988, 0.988, 0.980), // #FCFCFA near-white
    gutter: Color::from_rgb(0.447, 0.439, 0.447),  // #727072 gray, dimmed line numbers
};

// Standard light-scheme picks — used on light themes
const LIGHT_PALETTE: CodePalette = CodePalette {
    type_kw: Color::from_rgb8(0x0E, 0x74, 0x90),   // teal
    flow_kw: Color::from_rgb8(0xD1, 0x2A, 0x5C),   // crimson
    func_name: Color::from_rgb8(0x3F, 0x6E, 0x1E), // green
    preproc: Color::from_rgb8(0xD1, 0x2A, 0x5C),   // crimson
    string: Color::from_rgb8(0x9A, 0x67, 0x00),    // amber
    number: Color::from_rgb8(0x6D, 0x28, 0xD9),    // violet
    comment: Color::from_rgb8(0x8B, 0x8B, 0x8B),   // gray
    operator: Color::from_rgb8(0xD1, 0x2A, 0x5C),  // crimson
    default: Color::from_rgb8(0x1F, 0x1F, 0x1F),   // near-black
    gutter: Color::from_rgb8(0x8B, 0x8B, 0x8B),    // gray
};

fn code_palette() -> &'static CodePalette {
    if th::is_dark() {
        &DARK_PALETTE
    } else {
        &LIGHT_PALETTE
    }
}

static TYPE_KEYWORDS: &[&str] = &[
    "auto", "bool", "char", "const", "double", "enum", "extern", "float", "inline", "int", "long",
    "register", "restrict", "short", "signed", "static", "struct", "typedef", "union", "unsigned",
    "void", "volatile", "size_t", "FILE", "NULL", "true", "false",
];

static FLOW_KEYWORDS: &[&str] = &[
    "break", "case", "continue", "default", "do", "else", "for", "goto", "if", "return", "sizeof",
    "switch", "while",
];

/// Tokenize a C source string into coloured [`Span`]s.
///
/// Each logical line is terminated by a `"\n"` span. Tabs are left as-is;
/// [`code_view`] expands them to four spaces before rendering.
pub fn highlight_c(source: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut in_block_comment = false;

    let pal = code_palette();
    for line in source.lines() {
        tokenize_line(
            line.as_bytes(),
            line,
            &mut spans,
            &mut in_block_comment,
            pal,
        );
        spans.push(Span {
            text: "\n".into(),
            color: pal.default,
        });
    }

    spans
}

/// Stable widget ID for the scrollable code view in the module-detail panel.
///
/// Used by [`crate::app::NewcApp::update`] to scroll programmatically when
/// the user clicks a diagnostic line in the build panel.
pub const MODULE_CODE_SCROLL: &str = "module_code_scroll";

/// Render a highlighted C source string as a scrollable iced widget.
///
/// - `font_size` — monospace font size in logical pixels.
/// - `highlight_line` — 1-based line number to highlight (light background tint); `None` disables highlighting.
/// - `scroll_id` — if `Some`, assigns a stable [`iced::widget::Id`] to the
///   scrollable so it can be scrolled programmatically (use [`MODULE_CODE_SCROLL`]).
pub fn code_view<'a>(
    source: &str,
    font_size: f32,
    highlight_line: Option<usize>,
    scroll_id: Option<&'static str>,
) -> Element<'a, Message> {
    let pal = code_palette();
    let spans = highlight_c(source);

    let mut lines: Vec<Vec<(String, Color)>> = vec![Vec::new()];
    for span in spans {
        if span.text == "\n" {
            lines.push(Vec::new());
        } else {
            let expanded_text = span.text.replace('\t', "    ");
            if let Some(last) = lines.last_mut() {
                last.push((expanded_text, span.color));
            }
        }
    }
    if lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    let gutter_width = lines.len().to_string().len().max(2) as f32 * font_size * 0.62 + 8.0;

    let line_els: Vec<Element<'a, Message>> = lines
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            let tspans: Vec<TSpan<'static>> = if line.is_empty() {
                vec![TSpan::new(" ").font(iced::Font::MONOSPACE).size(font_size)]
            } else {
                line.into_iter()
                    .map(|(t, c)| {
                        TSpan::new(t)
                            .font(iced::Font::MONOSPACE)
                            .size(font_size)
                            .color(c)
                    })
                    .collect()
            };
            let line_widget = rich_text(tspans);
            let gutter = text((i + 1).to_string())
                .font(iced::Font::MONOSPACE)
                .size(font_size)
                .color(pal.gutter)
                .width(Length::Fixed(gutter_width))
                .align_x(iced::alignment::Horizontal::Right);
            let line_row = row![
                gutter,
                container(line_widget).padding(iced::Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 8.0
                })
            ]
            .width(Length::Fill);
            if highlight_line == Some(i + 1) {
                let tint = if th::is_dark() {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.06)
                };
                container(line_row)
                    .width(Length::Fill)
                    .style(move |_| iced::widget::container::Style {
                        background: Some(tint.into()),
                        ..Default::default()
                    })
                    .into()
            } else {
                line_row.into()
            }
        })
        .collect();

    let scr = scrollable(column(line_els).spacing(0)).height(Length::Fill);
    let scr = if let Some(id) = scroll_id {
        scr.id(iced::widget::Id::new(id))
    } else {
        scr
    };
    container(scr)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(8)
        .style(th::code_block_style)
        .into()
}

fn push(spans: &mut Vec<Span>, text: &str, color: Color) {
    if !text.is_empty() {
        spans.push(Span {
            text: text.to_string(),
            color,
        });
    }
}

fn tokenize_line(
    bytes: &[u8],
    s: &str,
    spans: &mut Vec<Span>,
    in_block_comment: &mut bool,
    pal: &CodePalette,
) {
    let n = bytes.len();
    let mut i = 0;

    // Continuation of a block comment from a previous line
    if *in_block_comment {
        if let Some(end) = s.find("*/") {
            push(spans, &s[..end + 2], pal.comment);
            *in_block_comment = false;
            let rest_i = end + 2;
            tokenize_line(&bytes[rest_i..], &s[rest_i..], spans, in_block_comment, pal);
        } else {
            push(spans, s, pal.comment);
        }
        return;
    }

    // Preprocessor line (#include, #define, …)
    let trimmed = s.trim_start();
    if trimmed.starts_with('#') {
        push(spans, s, pal.preproc);
        return;
    }

    while i < n {
        // Block comment /*
        if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            loop {
                if i + 1 < n && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    push(spans, &s[start..i], pal.comment);
                    break;
                }
                if i >= n {
                    push(spans, &s[start..], pal.comment);
                    *in_block_comment = true;
                    return;
                }
                i += 1;
            }
            continue;
        }

        // Line comment //
        if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            push(spans, &s[i..], pal.comment);
            return;
        }

        // String / char literal
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let delim = bytes[i];
            let start = i;
            i += 1;
            while i < n {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == delim {
                    i += 1;
                    break;
                }
                i += 1;
            }
            push(spans, &s[start..i], pal.string);
            continue;
        }

        // Number
        if bytes[i].is_ascii_digit() {
            let prev_alpha = i > 0 && (bytes[i - 1].is_ascii_alphabetic() || bytes[i - 1] == b'_');
            let start = i;
            while i < n
                && (bytes[i].is_ascii_alphanumeric()
                    || bytes[i] == b'.'
                    || bytes[i] == b'x'
                    || bytes[i] == b'X'
                    || bytes[i] == b'_')
            {
                i += 1;
            }
            push(
                spans,
                &s[start..i],
                if prev_alpha { pal.default } else { pal.number },
            );
            continue;
        }

        // Identifier or keyword
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &s[start..i];
            // Peek past spaces: function name if followed by '('
            let is_func = {
                let mut j = i;
                while j < n && bytes[j] == b' ' {
                    j += 1;
                }
                j < n && bytes[j] == b'('
            };
            let color = if TYPE_KEYWORDS.contains(&word) {
                pal.type_kw
            } else if FLOW_KEYWORDS.contains(&word) {
                pal.flow_kw
            } else if is_func {
                pal.func_name
            } else {
                pal.default
            };
            push(spans, word, color);
            continue;
        }

        // Operators
        let color = match bytes[i] {
            b'+' | b'-' | b'*' | b'/' | b'%' | b'=' | b'<' | b'>' | b'&' | b'|' | b'^' | b'~'
            | b'!' | b'?' | b':' => pal.operator,
            _ => pal.default,
        };
        // Advance by full UTF-8 char
        let mut char_end = i + 1;
        while char_end < n && bytes[char_end] & 0xC0 == 0x80 {
            char_end += 1;
        }
        push(spans, &s[i..char_end], color);
        i = char_end;
    }
}
