use egui::{text::LayoutJob, Color32, FontFamily, FontId, TextFormat};

// Type/storage keywords → cyan
const TYPE_KEYWORDS: &[&str] = &[
    "auto", "bool", "char", "const", "double", "enum", "extern", "float",
    "inline", "int", "long", "register", "restrict", "short", "signed",
    "static", "struct", "typedef", "union", "unsigned", "void", "volatile",
    "size_t", "FILE", "NULL", "true", "false",
];

// Control-flow keywords → purple
const FLOW_KEYWORDS: &[&str] = &[
    "break", "case", "continue", "default", "do", "else", "for",
    "goto", "if", "return", "sizeof", "switch", "while",
];

const PREPROC: &[&str] = &[
    "#include", "#define", "#ifdef", "#ifndef", "#endif", "#if", "#else",
    "#elif", "#pragma", "#undef", "#error", "#warning",
];

struct Palette {
    type_kw:  Color32, // int, char, void …
    flow_kw:  Color32, // if, return, while …
    func:     Color32, // identifiers before (
    preproc:  Color32, // #include #define
    string:   Color32, // "…" '…'
    number:   Color32, // 42 3.14
    comment:  Color32, // // and /* */
    operator: Color32, // + - * = < > …
    default:  Color32, // everything else
}

// Monokai Pro dark palette
fn dark_palette() -> Palette {
    Palette {
        type_kw:  Color32::from_rgb(120, 220, 232),  // #78DCE8 cyan
        flow_kw:  Color32::from_rgb(255,  97, 136),  // #FF6188 coral
        func:     Color32::from_rgb(169, 220, 118),  // #A9DC76 green
        preproc:  Color32::from_rgb(255,  97, 136),  // #FF6188 coral
        string:   Color32::from_rgb(255, 216, 102),  // #FFD866 yellow
        number:   Color32::from_rgb(171, 157, 242),  // #AB9DF2 lavender
        comment:  Color32::from_rgb(114, 112, 114),  // #727072 muted gray
        operator: Color32::from_rgb(255,  97, 136),  // #FF6188 coral
        default:  Color32::from_rgb(252, 252, 250),  // #FCFCFA near-white
    }
}

fn light_palette() -> Palette {
    Palette {
        type_kw:  Color32::from_rgb(0,   130, 150),
        flow_kw:  Color32::from_rgb(200,  20,  80),
        func:     Color32::from_rgb(60,  150,  20),
        preproc:  Color32::from_rgb(200,  20,  80),
        string:   Color32::from_rgb(160, 120,   0),
        number:   Color32::from_rgb(100,  60, 200),
        comment:  Color32::from_rgb(110, 108, 110),
        operator: Color32::from_rgb(200,  20,  80),
        default:  Color32::from_rgb(30,   30,  30),
    }
}

fn fmt(color: Color32, font: &FontId) -> TextFormat {
    TextFormat { font_id: font.clone(), color, ..Default::default() }
}

pub fn highlight_c(code: &str, is_dark: bool, font_size: f32) -> LayoutJob {
    let pal = if is_dark { dark_palette() } else { light_palette() };
    let font = FontId::new(font_size, FontFamily::Monospace);
    let mut job = LayoutJob::default();
    let mut in_block_comment = false;

    for line in code.lines() {
        tokenize_line(line, &pal, &font, &mut job, &mut in_block_comment);
        job.append("\n", 0.0, fmt(pal.default, &font));
    }

    job
}

fn tokenize_line(
    line: &str,
    pal: &Palette,
    font: &FontId,
    job: &mut LayoutJob,
    in_block_comment: &mut bool,
) {
    // Inside a multi-line block comment
    if *in_block_comment {
        if let Some(end) = line.find("*/") {
            job.append(&line[..end + 2], 0.0, fmt(pal.comment, font));
            *in_block_comment = false;
            let rest = &line[end + 2..];
            if !rest.is_empty() {
                tokenize_rest(rest, pal, font, job, in_block_comment);
            }
        } else {
            job.append(line, 0.0, fmt(pal.comment, font));
        }
        return;
    }

    // Preprocessor directive
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        for pp in PREPROC {
            if trimmed.starts_with(pp) {
                let leading = line.len() - trimmed.len();
                if leading > 0 {
                    job.append(&line[..leading], 0.0, fmt(pal.default, font));
                }
                job.append(pp, 0.0, fmt(pal.preproc, font));
                tokenize_rest(&line[leading + pp.len()..], pal, font, job, in_block_comment);
                return;
            }
        }
    }

    tokenize_rest(line, pal, font, job, in_block_comment);
}

fn tokenize_rest(
    s: &str,
    pal: &Palette,
    font: &FontId,
    job: &mut LayoutJob,
    in_block_comment: &mut bool,
) {
    let mut i = 0;
    let bytes = s.as_bytes();
    let n = s.len();

    while i < n {
        // Block comment /*
        if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let start = i;
            i += 2;
            loop {
                if i + 1 < n && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    job.append(&s[start..i], 0.0, fmt(pal.comment, font));
                    break;
                }
                if i >= n {
                    job.append(&s[start..], 0.0, fmt(pal.comment, font));
                    *in_block_comment = true;
                    return;
                }
                i += 1;
            }
            continue;
        }

        // Line comment //
        if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            job.append(&s[i..], 0.0, fmt(pal.comment, font));
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
            job.append(&s[start..i], 0.0, fmt(pal.string, font));
            continue;
        }

        // Number (not preceded by identifier char)
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
            let color = if prev_alpha { pal.default } else { pal.number };
            job.append(&s[start..i], 0.0, fmt(color, font));
            continue;
        }

        // Identifier or keyword
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &s[start..i];

            // Peek past whitespace: function name if followed by '('
            let is_func = {
                let mut j = i;
                while j < n && bytes[j] == b' ' { j += 1; }
                j < n && bytes[j] == b'('
            };

            let color = if TYPE_KEYWORDS.contains(&word) {
                pal.type_kw
            } else if FLOW_KEYWORDS.contains(&word) {
                pal.flow_kw
            } else if is_func {
                pal.func
            } else {
                pal.default
            };
            job.append(word, 0.0, fmt(color, font));
            continue;
        }

        // Operators / fallback — advance by full UTF-8 character, not one byte
        let color = match bytes[i] {
            b'+' | b'-' | b'*' | b'/' | b'%' | b'=' | b'<' | b'>'
            | b'&' | b'|' | b'^' | b'~' | b'!' | b'?' | b':' => pal.operator,
            _ => pal.default,
        };
        let mut char_end = i + 1;
        while char_end < n && bytes[char_end] & 0xC0 == 0x80 {
            char_end += 1; // skip UTF-8 continuation bytes
        }
        job.append(&s[i..char_end], 0.0, fmt(color, font));
        i = char_end;
    }
}
