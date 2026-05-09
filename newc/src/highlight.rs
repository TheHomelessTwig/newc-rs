/// Minimal C syntax highlighter that returns an egui LayoutJob.
/// Works without any external highlighting crates.

use egui::{text::LayoutJob, Color32, FontId, TextFormat, FontFamily};

const KEYWORDS: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do",
    "double", "else", "enum", "extern", "float", "for", "goto", "if", "inline",
    "int", "long", "register", "restrict", "return", "short", "signed", "sizeof",
    "static", "struct", "switch", "typedef", "union", "unsigned", "void",
    "volatile", "while", "NULL", "true", "false", "bool",
];

const PREPROC: &[&str] = &[
    "#include", "#define", "#ifdef", "#ifndef", "#endif", "#if", "#else",
    "#elif", "#pragma", "#undef", "#error", "#warning",
];

struct Palette {
    keyword:   Color32,
    preproc:   Color32,
    string:    Color32,
    number:    Color32,
    comment:   Color32,
    operator:  Color32,
    default:   Color32,
}

fn dark_palette() -> Palette {
    Palette {
        keyword:  Color32::from_rgb(86, 156, 214),
        preproc:  Color32::from_rgb(155, 155, 100),
        string:   Color32::from_rgb(206, 145, 120),
        number:   Color32::from_rgb(181, 206, 168),
        comment:  Color32::from_rgb(106, 153, 85),
        operator: Color32::from_rgb(180, 180, 180),
        default:  Color32::from_rgb(212, 212, 212),
    }
}

fn light_palette() -> Palette {
    Palette {
        keyword:  Color32::from_rgb(0, 0, 200),
        preproc:  Color32::from_rgb(100, 100, 0),
        string:   Color32::from_rgb(160, 40, 0),
        number:   Color32::from_rgb(0, 120, 0),
        comment:  Color32::from_rgb(0, 128, 0),
        operator: Color32::from_rgb(60, 60, 60),
        default:  Color32::from_rgb(30, 30, 30),
    }
}

pub fn highlight_c(code: &str, is_dark: bool, font_size: f32) -> LayoutJob {
    let pal = if is_dark { dark_palette() } else { light_palette() };
    let font = FontId::new(font_size, FontFamily::Monospace);
    let mut job = LayoutJob::default();

    for line in code.lines() {
        tokenize_line(line, &pal, &font, &mut job);
        job.append("\n", 0.0, TextFormat { font_id: font.clone(), color: pal.default, ..Default::default() });
    }

    job
}

fn fmt(color: Color32, font: &FontId) -> TextFormat {
    TextFormat { font_id: font.clone(), color, ..Default::default() }
}

fn tokenize_line(line: &str, pal: &Palette, font: &FontId, job: &mut LayoutJob) {
    let mut chars = line.char_indices().peekable();

    // Check preprocessor directive
    if line.trim_start().starts_with('#') {
        for pp in PREPROC {
            if line.trim_start().starts_with(pp) {
                let leading_spaces = line.len() - line.trim_start().len();
                if leading_spaces > 0 {
                    job.append(&line[..leading_spaces], 0.0, fmt(pal.default, font));
                }
                job.append(pp, 0.0, fmt(pal.preproc, font));
                let rest = &line[leading_spaces + pp.len()..];
                // Colour the rest as preproc/string
                tokenize_rest(rest, pal, font, job, true);
                return;
            }
        }
    }

    // Line comment
    if let Some(pos) = line.find("//") {
        tokenize_rest(&line[..pos], pal, font, job, false);
        job.append(&line[pos..], 0.0, fmt(pal.comment, font));
        return;
    }

    tokenize_rest(line, pal, font, job, false);
}

fn tokenize_rest(s: &str, pal: &Palette, font: &FontId, job: &mut LayoutJob, _is_preproc: bool) {
    let mut i = 0;
    let bytes = s.as_bytes();
    let n = s.len();

    while i < n {
        // String literal
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let delim = bytes[i];
            let start = i;
            i += 1;
            while i < n {
                if bytes[i] == b'\\' { i += 2; continue; }
                if bytes[i] == delim { i += 1; break; }
                i += 1;
            }
            job.append(&s[start..i], 0.0, fmt(pal.string, font));
            continue;
        }

        // Number
        if bytes[i].is_ascii_digit() || (bytes[i] == b'-' && i + 1 < n && bytes[i + 1].is_ascii_digit()) {
            let start = i;
            if bytes[i] == b'-' { i += 1; }
            while i < n && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b'x' || bytes[i] == b'X' || (i > 0 && bytes[i-1] == b'x' && bytes[i].is_ascii_hexdigit())) {
                i += 1;
            }
            // Only colour if standalone number (not part of identifier)
            let prev_alpha = start > 0 && (bytes[start - 1].is_ascii_alphabetic() || bytes[start - 1] == b'_');
            if !prev_alpha {
                job.append(&s[start..i], 0.0, fmt(pal.number, font));
                continue;
            } else {
                job.append(&s[start..i], 0.0, fmt(pal.default, font));
                continue;
            }
        }

        // Identifier or keyword
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &s[start..i];
            let color = if KEYWORDS.contains(&word) { pal.keyword } else { pal.default };
            job.append(word, 0.0, fmt(color, font));
            continue;
        }

        // Operators and punctuation
        let color = match bytes[i] {
            b'{' | b'}' | b'(' | b')' | b'[' | b']' => pal.default,
            b'+' | b'-' | b'*' | b'/' | b'%' | b'=' | b'<' | b'>' | b'&' | b'|' | b'^' | b'~' | b'!' => pal.operator,
            _ => pal.default,
        };
        job.append(&s[i..i+1], 0.0, fmt(color, font));
        i += 1;
    }
}
