//! Syntax highlighting for C source code.
//!
//! [`highlight_c`] tokenizes a C source string into coloured [`Span`]s using a
//! Monokai Pro palette. [`code_view`] wraps those spans in a scrollable iced
//! widget suitable for the module-detail and header-editor panels.
//!
//! The highlighter is a single-pass byte scanner: it handles preprocessor
//! lines, block/line comments, string/char literals, numeric literals,
//! keywords (type and flow), function calls (identifier followed by `(`), and
//! operators. It does not parse the full C grammar.

use iced::widget::{column, container, rich_text, scrollable};
use iced::widget::text::Span as TSpan;
use iced::{Color, Element, Length};
use crate::state::Message;
use crate::theme as th;

/// A contiguous run of text with a single display colour.
///
/// Produced by [`highlight_c`] and consumed by [`code_view`].
#[derive(Debug, Clone)]
pub struct Span {
    pub text: String,
    pub color: Color,
}

// Monokai Pro palette
const TYPE_KW:   Color = Color::from_rgb(0.471, 0.863, 0.910); // #78DCE8 cyan
const FLOW_KW:   Color = Color::from_rgb(1.000, 0.380, 0.533); // #FF6188 coral
const FUNC_NAME: Color = Color::from_rgb(0.663, 0.863, 0.463); // #A9DC76 green
const PREPROC:   Color = Color::from_rgb(1.000, 0.380, 0.533); // #FF6188 coral
const STRING:    Color = Color::from_rgb(1.000, 0.847, 0.400); // #FFD866 yellow
const NUMBER:    Color = Color::from_rgb(0.671, 0.616, 0.949); // #AB9DF2 lavender
const COMMENT:   Color = Color::from_rgb(0.447, 0.439, 0.447); // #727072 gray
const OPERATOR:  Color = Color::from_rgb(1.000, 0.380, 0.533); // #FF6188 coral
const DEFAULT:   Color = Color::from_rgb(0.988, 0.988, 0.980); // #FCFCFA near-white

static TYPE_KEYWORDS: &[&str] = &[
    "auto", "bool", "char", "const", "double", "enum", "extern", "float",
    "inline", "int", "long", "register", "restrict", "short", "signed",
    "static", "struct", "typedef", "union", "unsigned", "void", "volatile",
    "size_t", "FILE", "NULL", "true", "false",
];

static FLOW_KEYWORDS: &[&str] = &[
    "break", "case", "continue", "default", "do", "else", "for",
    "goto", "if", "return", "sizeof", "switch", "while",
];

/// Tokenize a C source string into coloured [`Span`]s.
///
/// Each logical line is terminated by a `"\n"` span. Tabs are left as-is;
/// [`code_view`] expands them to four spaces before rendering.
pub fn highlight_c(source: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut in_block_comment = false;

    for line in source.lines() {
        tokenize_line(line.as_bytes(), line, &mut spans, &mut in_block_comment);
        spans.push(Span { text: "\n".into(), color: DEFAULT });
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
pub fn code_view<'a>(source: &str, font_size: f32, highlight_line: Option<usize>, scroll_id: Option<&'static str>) -> Element<'a, Message> {
    let spans = highlight_c(source);

    let mut lines: Vec<Vec<(String, Color)>> = vec![Vec::new()];
    for span in spans {
        if span.text == "\n" {
            lines.push(Vec::new());
        } else {
            let t = span.text.replace('\t', "    ");
            if let Some(last) = lines.last_mut() {
                last.push((t, span.color));
            }
        }
    }
    if lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    let line_els: Vec<Element<'a, Message>> = lines.into_iter().enumerate().map(|(i, line)| {
        let tspans: Vec<TSpan<'static>> = if line.is_empty() {
            vec![TSpan::new(" ").font(iced::Font::MONOSPACE).size(font_size)]
        } else {
            line.into_iter().map(|(t, c)| {
                TSpan::new(t).font(iced::Font::MONOSPACE).size(font_size).color(c)
            }).collect()
        };
        let line_widget = rich_text(tspans);
        if highlight_line == Some(i + 1) {
            container(line_widget)
                .width(Length::Fill)
                .style(|_| iced::widget::container::Style {
                    background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.08).into()),
                    ..Default::default()
                })
                .into()
        } else {
            line_widget.into()
        }
    }).collect();

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
        spans.push(Span { text: text.to_string(), color });
    }
}

fn tokenize_line(
    bytes: &[u8],
    s: &str,
    spans: &mut Vec<Span>,
    in_block_comment: &mut bool,
) {
    let n = bytes.len();
    let mut i = 0;

    // Continuation of a block comment from a previous line
    if *in_block_comment {
        if let Some(end) = s.find("*/") {
            push(spans, &s[..end + 2], COMMENT);
            *in_block_comment = false;
            let rest_i = end + 2;
            tokenize_line(&bytes[rest_i..], &s[rest_i..], spans, in_block_comment);
        } else {
            push(spans, s, COMMENT);
        }
        return;
    }

    // Preprocessor line (#include, #define, …)
    let trimmed = s.trim_start();
    if trimmed.starts_with('#') {
        push(spans, s, PREPROC);
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
                    push(spans, &s[start..i], COMMENT);
                    break;
                }
                if i >= n {
                    push(spans, &s[start..], COMMENT);
                    *in_block_comment = true;
                    return;
                }
                i += 1;
            }
            continue;
        }

        // Line comment //
        if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            push(spans, &s[i..], COMMENT);
            return;
        }

        // String / char literal
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let delim = bytes[i];
            let start = i;
            i += 1;
            while i < n {
                if bytes[i] == b'\\' { i += 2; continue; }
                if bytes[i] == delim { i += 1; break; }
                i += 1;
            }
            push(spans, &s[start..i], STRING);
            continue;
        }

        // Number
        if bytes[i].is_ascii_digit() {
            let prev_alpha = i > 0
                && (bytes[i - 1].is_ascii_alphabetic() || bytes[i - 1] == b'_');
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
            push(spans, &s[start..i], if prev_alpha { DEFAULT } else { NUMBER });
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
                while j < n && bytes[j] == b' ' { j += 1; }
                j < n && bytes[j] == b'('
            };
            let color = if TYPE_KEYWORDS.contains(&word) {
                TYPE_KW
            } else if FLOW_KEYWORDS.contains(&word) {
                FLOW_KW
            } else if is_func {
                FUNC_NAME
            } else {
                DEFAULT
            };
            push(spans, word, color);
            continue;
        }

        // Operators
        let color = match bytes[i] {
            b'+' | b'-' | b'*' | b'/' | b'%' | b'=' | b'<' | b'>'
            | b'&' | b'|' | b'^' | b'~' | b'!' | b'?' | b':' => OPERATOR,
            _ => DEFAULT,
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
