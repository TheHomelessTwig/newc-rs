// C syntax highlighting — produces (text, Color) spans for iced rich_text.
// Full tokenizer will be ported from the egui version during module_detail porting.

use iced::Color;

#[derive(Debug, Clone)]
pub struct Span {
    pub text: String,
    pub color: Color,
}

pub fn highlight_c(source: &str) -> Vec<Span> {
    vec![Span { text: source.to_string(), color: Color::WHITE }]
}
