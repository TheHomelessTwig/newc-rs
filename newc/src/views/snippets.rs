use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Color, Element, Length};

#[derive(Default, Clone)]
pub struct SnippetsState {
    pub selected_cat: usize,
    pub selected_snippet: Option<usize>,
}

struct Snippet {
    name: &'static str,
    desc: &'static str,
    code: &'static str,
}

struct SnippetCat {
    name: &'static str,
    items: &'static [Snippet],
}

macro_rules! s {
    ($name:expr, $desc:expr, $code:expr) => {
        Snippet { name: $name, desc: $desc, code: $code }
    };
}

const CATEGORIES: &[SnippetCat] = &[
    SnippetCat {
        name: "Control Flow",
        items: &[
            s!("for loop", "Iterate with an index counter from 0 to n-1.",
               "for (int i = 0; i < n; i++) {\n    \n}"),
            s!("while loop", "Loop while a condition holds.",
               "while (condition) {\n    \n}"),
            s!("do-while", "Execute body at least once.",
               "do {\n    \n} while (condition);"),
            s!("if / else if / else", "Branch on multiple conditions.",
               "if (condition) {\n    \n} else if (other) {\n    \n} else {\n    \n}"),
            s!("switch", "Dispatch on an integer or enum value.",
               "switch (var) {\n    case 1:\n        break;\n    default:\n        break;\n}"),
        ],
    },
    SnippetCat {
        name: "Data Types",
        items: &[
            s!("struct definition", "Define a named aggregate type.",
               "typedef struct {\n    int id;\n    char name[64];\n} MyStruct;"),
            s!("enum definition", "Named integer constants.",
               "typedef enum {\n    STATE_IDLE = 0,\n    STATE_RUNNING,\n} State;"),
            s!("array init", "Declare and zero-initialise an array.",
               "int arr[100];\nmemset(arr, 0, sizeof(arr));"),
        ],
    },
    SnippetCat {
        name: "Pointers",
        items: &[
            s!("pointer basics", "Declare, address, dereference.",
               "int x = 42;\nint *ptr = &x;\nprintf(\"%d\\n\", *ptr);"),
            s!("malloc / free", "Heap allocate and release.",
               "int *p = malloc(sizeof(int));\nif (!p) { perror(\"malloc\"); exit(1); }\nfree(p);\np = NULL;"),
            s!("pointer to array", "Dynamic array pattern.",
               "int n = 10;\nint *arr = malloc(n * sizeof(int));\nif (!arr) { perror(\"malloc\"); exit(1); }\nfree(arr);\narr = NULL;"),
        ],
    },
    SnippetCat {
        name: "File I/O",
        items: &[
            s!("open / close file", "Open a file, check for errors, then close.",
               "FILE *fp = fopen(\"file.txt\", \"r\");\nif (!fp) { perror(\"fopen\"); return 1; }\n/* use fp */\nfclose(fp);"),
            s!("read line by line", "Read a text file one line at a time.",
               "char line[256];\nwhile (fgets(line, sizeof(line), fp)) {\n    /* process line */\n}"),
            s!("write to file", "Write a string to a file.",
               "fprintf(fp, \"Hello, %s!\\n\", name);"),
        ],
    },
    SnippetCat {
        name: "Strings",
        items: &[
            s!("strcmp / strncmp", "Compare two strings safely.",
               "if (strncmp(a, b, sizeof(b)) == 0) {\n    /* equal */\n}"),
            s!("strcpy / strncpy", "Copy string safely.",
               "strncpy(dest, src, sizeof(dest) - 1);\ndest[sizeof(dest) - 1] = '\\0';"),
            s!("sprintf", "Format into a string buffer.",
               "char buf[128];\nsnprintf(buf, sizeof(buf), \"Value: %d\", val);"),
        ],
    },
];

pub fn view(state: &crate::state::AppState) -> Element<'_, crate::state::Message> {
    use crate::state::Message;

    let _s = &state.snippets_cat;
    let sel_snippet = state.snippets_selected;
    let cat_idx = state.snippets_cat;

    // Category tabs
    let cat_btns: Vec<Element<Message>> = CATEGORIES.iter().enumerate().map(|(i, cat)| {
        button(text(cat.name).size(12))
            .on_press(Message::SnippetsCat(i))
            .into()
    }).collect();

    let cat_row = row(cat_btns).spacing(4).wrap();

    let cat = &CATEGORIES[cat_idx.min(CATEGORIES.len() - 1)];

    // Snippet list
    let snippet_btns: Vec<Element<Message>> = cat.items.iter().enumerate().map(|(i, snippet)| {
        column![
            button(text(snippet.name).size(13))
                .on_press(Message::SnippetsSelect(Some(i)))
                .width(Length::Fill),
            text(snippet.desc).size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(2)
        .into()
    }).collect();

    // Code detail
    let code_panel: Element<Message> = if let Some(idx) = sel_snippet {
        if let Some(snippet) = cat.items.get(idx) {
            column![
                row![
                    text(snippet.name).size(14),
                    Space::new().width(Length::Fill),
                    button(text("Copy").size(12))
                        .on_press(Message::SnippetsCopy(snippet.code.to_string())),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                text(snippet.desc).size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
                scrollable(
                    text(snippet.code).font(iced::Font::MONOSPACE).size(13)
                ).height(Length::Fill),
            ]
            .spacing(8)
            .into()
        } else {
            text("Select a snippet").color(Color::from_rgb(0.5, 0.5, 0.5)).into()
        }
    } else {
        text("Select a snippet").color(Color::from_rgb(0.5, 0.5, 0.5)).into()
    };

    let header_row: Element<Message> = if state.snippets_window.is_none() {
        row![
            text("C Snippets").size(18),
            iced::widget::Space::new().width(iced::Length::Fill),
            button(text("⊞").size(12))
                .on_press(Message::OpenSnippetsWindow)
                .style(crate::theme::btn_ghost),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        text("C Snippets").size(18).into()
    };

    column![
        header_row,
        cat_row,
        row![
            scrollable(column(snippet_btns).spacing(6)).width(220).height(Length::Fill),
            scrollable(code_panel).height(Length::Fill),
        ]
        .spacing(12)
        .height(Length::Fill),
    ]
    .spacing(10)
    .padding(12)
    .into()
}
