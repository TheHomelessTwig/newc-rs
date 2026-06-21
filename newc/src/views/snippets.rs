//! C snippet browser — categorised ready-to-copy code snippets with syntax highlighting.

use iced::widget::{button, column, pane_grid, row, scrollable, text, Space};
use iced::{Color, Element, Length};
use crate::highlight::code_view;

/// Identifies which pane of the Snippets resizable pane-grid is being rendered.
#[derive(Clone, Copy)]
pub enum SnippetsPane { List, Code }

/// Persistent UI state for the snippets browser.
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
            s!("strtok_r", "Split a string by delimiter (re-entrant).",
               "char *saveptr;\nchar *token = strtok_r(str, \",\", &saveptr);\nwhile (token) {\n    /* process token */\n    token = strtok_r(NULL, \",\", &saveptr);\n}"),
            s!("string to int (safe)", "Convert string to integer with error check.",
               "char *endptr;\nlong val = strtol(str, &endptr, 10);\nif (endptr == str || *endptr != '\\0') {\n    /* conversion failed */\n}"),
        ],
    },
    SnippetCat {
        name: "Error Handling",
        items: &[
            s!("perror / exit", "Print system error and exit.",
               "if (result < 0) {\n    perror(\"operation failed\");\n    exit(EXIT_FAILURE);\n}"),
            s!("errno check", "Inspect errno after a failed syscall.",
               "#include <errno.h>\nif (errno == ENOENT) {\n    fprintf(stderr, \"File not found\\n\");\n} else if (errno == EACCES) {\n    fprintf(stderr, \"Permission denied\\n\");\n}"),
            s!("realloc safe", "Reallocate without losing original pointer on failure.",
               "void *tmp = realloc(ptr, new_size);\nif (!tmp) {\n    perror(\"realloc\");\n    free(ptr);\n    exit(EXIT_FAILURE);\n}\nptr = tmp;"),
            s!("NULL guard", "Check pointer before dereferencing.",
               "if (!ptr) {\n    fprintf(stderr, \"Error: null pointer\\n\");\n    return -1;\n}"),
        ],
    },
    SnippetCat {
        name: "Sorting",
        items: &[
            s!("qsort integers", "Sort an integer array with qsort.",
               "int cmp_int(const void *a, const void *b) {\n    return (*(int*)a - *(int*)b);\n}\nqsort(arr, n, sizeof(int), cmp_int);"),
            s!("qsort strings", "Sort a string array with qsort.",
               "int cmp_str(const void *a, const void *b) {\n    return strcmp(*(const char**)a, *(const char**)b);\n}\nqsort(arr, n, sizeof(char*), cmp_str);"),
            s!("bubble sort", "Simple O(n²) sort for small arrays.",
               "for (int i = 0; i < n - 1; i++) {\n    for (int j = 0; j < n - i - 1; j++) {\n        if (arr[j] > arr[j+1]) {\n            int tmp = arr[j];\n            arr[j] = arr[j+1];\n            arr[j+1] = tmp;\n        }\n    }\n}"),
            s!("binary search", "Search a sorted array — O(log n).",
               "int lo = 0, hi = n - 1;\nwhile (lo <= hi) {\n    int mid = lo + (hi - lo) / 2;\n    if (arr[mid] == target) { /* found */ break; }\n    else if (arr[mid] < target) lo = mid + 1;\n    else hi = mid - 1;\n}"),
        ],
    },
    SnippetCat {
        name: "Bit Ops",
        items: &[
            s!("set bit", "Set bit N in a value.",
               "value |= (1 << N);"),
            s!("clear bit", "Clear bit N in a value.",
               "value &= ~(1 << N);"),
            s!("toggle bit", "Flip bit N in a value.",
               "value ^= (1 << N);"),
            s!("test bit", "Check if bit N is set.",
               "if (value & (1 << N)) { /* bit is set */ }"),
            s!("count set bits", "Population count (Hamming weight).",
               "int count = 0;\nunsigned int v = value;\nwhile (v) { count += v & 1; v >>= 1; }"),
            s!("is power of two", "Test if a positive integer is a power of 2.",
               "int is_pow2 = (n > 0) && ((n & (n - 1)) == 0);"),
        ],
    },
    SnippetCat {
        name: "Linked List",
        items: &[
            s!("node definition", "Singly-linked list node.",
               "typedef struct Node {\n    int data;\n    struct Node *next;\n} Node;"),
            s!("prepend node", "Insert at head — O(1).",
               "Node *node = malloc(sizeof(Node));\nif (!node) { perror(\"malloc\"); exit(1); }\nnode->data = val;\nnode->next = *head;\n*head = node;"),
            s!("traverse list", "Walk every node in a singly-linked list.",
               "Node *cur = head;\nwhile (cur) {\n    /* process cur->data */\n    cur = cur->next;\n}"),
            s!("free list", "Release all nodes in a list.",
               "while (head) {\n    Node *tmp = head;\n    head = head->next;\n    free(tmp);\n}"),
        ],
    },
    SnippetCat {
        name: "Signal Handling",
        items: &[
            s!("sigaction handler", "Install a signal handler with sigaction.",
               "#include <signal.h>\nvoid handler(int sig) {\n    /* handle signal */\n}\nstruct sigaction sa = {0};\nsa.sa_handler = handler;\nsigemptyset(&sa.sa_mask);\nsigaction(SIGINT, &sa, NULL);"),
            s!("volatile flag", "Safe flag set by a signal handler.",
               "static volatile sig_atomic_t running = 1;\nvoid handler(int sig) { running = 0; }\n/* in main loop */\nwhile (running) { /* ... */ }"),
            s!("ignore signal", "Suppress a signal.",
               "signal(SIGPIPE, SIG_IGN);"),
        ],
    },
];

/// Renders the C snippet browser with category tabs, snippet list, and code detail panel.
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
                code_view(snippet.code, state.config.code_font_size, None, None),
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

    let list_cell = std::cell::RefCell::new(Some(
        scrollable(column(snippet_btns).spacing(6)).height(Length::Fill).into()
    ));
    let code_cell = std::cell::RefCell::new(Some(
        scrollable(code_panel).height(Length::Fill).into()
    ));

    let grid: Element<Message> = pane_grid::PaneGrid::new(&state.snippets_panes, |_, pane, _| {
        let body: Element<Message> = match pane {
            SnippetsPane::List => list_cell.borrow_mut().take().unwrap(),
            SnippetsPane::Code => code_cell.borrow_mut().take().unwrap(),
        };
        pane_grid::Content::new(body)
    })
    .on_resize(8, Message::SnippetsPaneResized)
    .spacing(4)
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    column![
        header_row,
        cat_row,
        grid,
    ]
    .spacing(10)
    .padding(12)
    .into()
}
