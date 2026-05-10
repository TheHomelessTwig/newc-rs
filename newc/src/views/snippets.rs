use egui::{Color32, Context, RichText, ScrollArea, SidePanel, CentralPanel, Window};

#[derive(Default, Clone)]
pub struct SnippetsState {
    pub selected_cat: usize,
    pub selected_snippet: Option<usize>,
}

struct SnippetCat {
    name: &'static str,
    items: &'static [(&'static str, &'static str)],
}

const CATEGORIES: &[SnippetCat] = &[
    SnippetCat {
        name: "Control Flow",
        items: &[
            ("for loop",            "for (int i = 0; i < n; i++) {\n    \n}"),
            ("while loop",          "while (condition) {\n    \n}"),
            ("do-while loop",       "do {\n    \n} while (condition);"),
            ("if / else if / else", "if (condition) {\n    \n} else if (other) {\n    \n} else {\n    \n}"),
            ("switch statement",    "switch (var) {\n    case 1:\n        break;\n    case 2:\n        break;\n    default:\n        break;\n}"),
        ],
    },
    SnippetCat {
        name: "Data Types",
        items: &[
            ("struct definition",    "typedef struct {\n    int id;\n    char name[64];\n    double value;\n} MyStruct;"),
            ("struct with pointer",  "typedef struct Node {\n    int data;\n    struct Node *next;\n} Node;"),
            ("enum definition",      "typedef enum {\n    STATE_IDLE = 0,\n    STATE_RUNNING,\n    STATE_DONE,\n} State;"),
            ("array init (memset)",  "int arr[100];\nmemset(arr, 0, sizeof(arr));"),
            ("2D array",             "int grid[ROWS][COLS];\nfor (int r = 0; r < ROWS; r++) {\n    for (int c = 0; c < COLS; c++) {\n        grid[r][c] = 0;\n    }\n}"),
        ],
    },
    SnippetCat {
        name: "Pointers",
        items: &[
            ("pointer basics",       "int x = 42;\nint *ptr = &x;       // ptr holds address of x\nprintf(\"%d\\n\", *ptr); // dereference: prints 42\n*ptr = 100;          // modifies x through pointer"),
            ("double pointer",       "void set_value(int **pp, int val) {\n    *pp = malloc(sizeof(int));\n    **pp = val;\n}\nint *p = NULL;\nset_value(&p, 42);\nprintf(\"%d\\n\", *p);\nfree(p);"),
            ("pointer arithmetic",   "int arr[] = {10, 20, 30, 40};\nint *p = arr;              // points to arr[0]\nprintf(\"%d\\n\", *(p + 2)); // prints 30 (arr[2])\np++;                       // now points to arr[1]"),
            ("NULL check pattern",   "int *ptr = malloc(sizeof(int));\nif (ptr == NULL) {\n    fprintf(stderr, \"malloc failed\\n\");\n    return 1;\n}\n// ... use ptr ...\nfree(ptr);\nptr = NULL;"),
            ("function pointer",     "int (*fn_ptr)(int, int);   // declare\nfn_ptr = my_function;      // assign\nint result = fn_ptr(a, b); // call\n\n// typedef for readability:\ntypedef int (*BinOp)(int, int);\nBinOp op = add;"),
        ],
    },
    SnippetCat {
        name: "Memory",
        items: &[
            ("malloc / free",        "int *arr = malloc(n * sizeof(int));\nif (arr == NULL) {\n    fprintf(stderr, \"malloc failed\\n\");\n    return 1;\n}\n// ... use arr ...\nfree(arr);\narr = NULL;"),
            ("calloc (zero-init)",   "int *arr = calloc(n, sizeof(int)); // zero-initialised\nif (arr == NULL) {\n    fprintf(stderr, \"calloc failed\\n\");\n    return 1;\n}\nfree(arr);\narr = NULL;"),
            ("realloc",              "arr = realloc(arr, new_size * sizeof(int));\nif (arr == NULL) {\n    fprintf(stderr, \"realloc failed\\n\");\n    // original arr is now lost — save pointer first!\n    return 1;\n}"),
            ("qsort int",            "int cmp_int(const void *a, const void *b) {\n    return *(int*)a - *(int*)b;\n}\nqsort(arr, n, sizeof(int), cmp_int);"),
            ("qsort struct",         "typedef struct { char name[64]; int score; } Record;\nint cmp_score(const void *a, const void *b) {\n    const Record *ra = (const Record *)a;\n    const Record *rb = (const Record *)b;\n    return rb->score - ra->score; // descending\n}\nqsort(records, count, sizeof(Record), cmp_score);"),
        ],
    },
    SnippetCat {
        name: "Strings",
        items: &[
            ("safe string read",     "char buf[128];\nif (fgets(buf, sizeof(buf), stdin) != NULL) {\n    buf[strcspn(buf, \"\\n\")] = '\\0'; // strip newline\n}"),
            ("string copy safe",     "// Always use snprintf or strncpy over strcpy\nchar dst[64];\nsnprintf(dst, sizeof(dst), \"%s\", src);\n// or:\nstrncpy(dst, src, sizeof(dst) - 1);\ndst[sizeof(dst) - 1] = '\\0';"),
            ("string split/tokenise","char str[] = \"hello,world,foo\";\nchar *token = strtok(str, \",\");\nwhile (token != NULL) {\n    printf(\"%s\\n\", token);\n    token = strtok(NULL, \",\");\n}"),
            ("string to int (safe)", "// atoi has no error checking — use strtol instead\nchar *end;\nlong val = strtol(str, &end, 10);\nif (end == str || *end != '\\0') {\n    fprintf(stderr, \"Invalid number: %s\\n\", str);\n} else {\n    printf(\"Value: %ld\\n\", val);\n}"),
            ("string search",        "char *found = strstr(haystack, needle);\nif (found != NULL) {\n    printf(\"Found at index %ld\\n\", found - haystack);\n} else {\n    puts(\"Not found\");\n}"),
        ],
    },
    SnippetCat {
        name: "File I/O",
        items: &[
            ("fopen / fclose",       "FILE *fp = fopen(\"data.txt\", \"r\");\nif (fp == NULL) {\n    perror(\"fopen\");\n    return 1;\n}\n// ... use fp ...\nfclose(fp);"),
            ("fgets line loop",      "char line[256];\nwhile (fgets(line, sizeof(line), fp) != NULL) {\n    line[strcspn(line, \"\\n\")] = '\\0'; // strip newline\n    // process line\n}"),
            ("fprintf to file",      "FILE *fp = fopen(\"output.txt\", \"w\");\nif (fp == NULL) { perror(\"fopen\"); return 1; }\nfprintf(fp, \"Name: %s, Score: %d\\n\", name, score);\nfclose(fp);"),
            ("binary fread/fwrite",  "typedef struct { int id; double value; } Record;\nRecord r = {1, 3.14};\n// Write:\nFILE *fp = fopen(\"data.bin\", \"wb\");\nfwrite(&r, sizeof(Record), 1, fp);\nfclose(fp);\n// Read:\nRecord r2;\nfp = fopen(\"data.bin\", \"rb\");\nfread(&r2, sizeof(Record), 1, fp);\nfclose(fp);"),
            ("append to file",       "FILE *fp = fopen(\"log.txt\", \"a\"); // 'a' = append\nif (fp == NULL) { perror(\"fopen\"); return 1; }\nfprintf(fp, \"[LOG] %s\\n\", message);\nfclose(fp);"),
        ],
    },
    SnippetCat {
        name: "Recursion",
        items: &[
            ("factorial",            "int factorial(int n) {\n    if (n <= 1) return 1;      // base case\n    return n * factorial(n - 1); // recursive case\n}"),
            ("fibonacci",            "int fibonacci(int n) {\n    if (n <= 0) return 0;\n    if (n == 1) return 1;\n    return fibonacci(n - 1) + fibonacci(n - 2);\n}"),
            ("power",                "double power(double base, int exp) {\n    if (exp == 0) return 1.0;\n    if (exp < 0)  return 1.0 / power(base, -exp);\n    return base * power(base, exp - 1);\n}"),
            ("sum of array",         "int sum_recursive(int arr[], int size) {\n    if (size == 0) return 0;\n    return arr[0] + sum_recursive(arr + 1, size - 1);\n}"),
            ("binary search (rec)",  "int binary_search_rec(int arr[], int low, int high, int target) {\n    if (low > high) return -1;\n    int mid = low + (high - low) / 2;\n    if (arr[mid] == target) return mid;\n    if (arr[mid] < target) return binary_search_rec(arr, mid + 1, high, target);\n    return binary_search_rec(arr, low, mid - 1, target);\n}"),
        ],
    },
    SnippetCat {
        name: "Error Handling",
        items: &[
            ("errno + perror",       "#include <errno.h>\n// After a failed syscall:\nif (result == -1) {\n    perror(\"operation name\"); // prints: \"operation name: <reason>\"\n    return 1;\n}"),
            ("exit codes",           "// Convention:\n// 0  = success\n// 1  = general error\n// 2  = misuse of command\n// Use EXIT_SUCCESS / EXIT_FAILURE from <stdlib.h>\nreturn EXIT_SUCCESS;\nreturn EXIT_FAILURE;"),
            ("defensive NULL guard", "void process(int *data, int size) {\n    if (data == NULL || size <= 0) {\n        fprintf(stderr, \"process: invalid arguments\\n\");\n        return;\n    }\n    // safe to use data here\n}"),
            ("error propagation",    "// Return 0 on success, -1 on error (Unix style)\nint do_thing(void) {\n    if (condition_failed) {\n        fprintf(stderr, \"do_thing: reason\\n\");\n        return -1;\n    }\n    return 0;\n}\n// Caller:\nif (do_thing() != 0) {\n    return -1; // propagate\n}"),
        ],
    },
    SnippetCat {
        name: "Preprocessor",
        items: &[
            ("include guard",        "#ifndef MY_HEADER_H\n#define MY_HEADER_H\n\n// declarations here\n\n#endif /* MY_HEADER_H */"),
            ("#pragma once",         "#pragma once\n\n// declarations here\n// (simpler alternative to #ifndef guard)"),
            ("#define constant",     "#define MAX_SIZE   100\n#define PI         3.14159265358979\n#define BUFFER_LEN 256"),
            ("#define macro",        "// Macro with argument (use parentheses!)\n#define MAX(a, b) ((a) > (b) ? (a) : (b))\n#define MIN(a, b) ((a) < (b) ? (a) : (b))\n#define ABS(x)    ((x) >= 0 ? (x) : -(x))"),
            ("conditional compile",  "#ifdef DEBUG\n    printf(\"[DEBUG] value = %d\\n\", value);\n#endif\n\n// Compile with: gcc -DDEBUG ...\n// Or: #define DEBUG 1 at top of file"),
        ],
    },
    SnippetCat {
        name: "Functions",
        items: &[
            ("argc/argv main",       "int main(int argc, char *argv[]) {\n    if (argc < 2) {\n        fprintf(stderr, \"Usage: %s <arg>\\n\", argv[0]);\n        return 1;\n    }\n    printf(\"Arg 1: %s\\n\", argv[1]);\n    return 0;\n}"),
            ("variadic function",    "#include <stdarg.h>\nvoid my_printf(const char *fmt, ...) {\n    va_list args;\n    va_start(args, fmt);\n    vprintf(fmt, args);\n    va_end(args);\n}"),
            ("callback pattern",     "void apply(int arr[], int n, int (*fn)(int)) {\n    for (int i = 0; i < n; i++) {\n        arr[i] = fn(arr[i]);\n    }\n}\nint double_it(int x) { return x * 2; }\napply(arr, n, double_it);"),
            ("swap (generic)",       "void swap(void *a, void *b, size_t size) {\n    char tmp[size];\n    memcpy(tmp, a, size);\n    memcpy(a, b, size);\n    memcpy(b, tmp, size);\n}\n// Usage: swap(&x, &y, sizeof(int));"),
        ],
    },
    SnippetCat {
        name: "Reference",
        items: &[
            ("printf format",        "// int:      %d  or  %i\n// unsigned: %u\n// float:    %f  (%.2f = 2 decimal places)\n// double:   %lf\n// char:     %c\n// string:   %s\n// pointer:  %p\n// hex:      %x (lower) / %X (upper)\n// octal:    %o\n// width:    %-10s  (left-align, width 10)\n//           %08d   (zero-pad to width 8)\nprintf(\"%-20s %8.2f\\n\", name, value);"),
            ("string.h cheatsheet",  "#include <string.h>\nstrlen(s)           // length\nstrcpy(dst, src)    // UNSAFE — use snprintf\nstrncpy(dst,src,n)  // bounded copy\nstrcmp(a, b)        // 0 = equal, <0 if a<b\nstrcat(dst, src)    // concatenate\nstrtok(s, delim)    // tokenise (modifies s)\nstrchr(s, c)        // first occurrence of c\nstrrchr(s, c)       // last occurrence of c\nstrstr(s, sub)      // find substring\nmemset(ptr,val,n)   // fill memory\nmemcpy(dst,src,n)   // copy bytes"),
            ("math.h cheatsheet",    "#include <math.h>  // link with -lm\nsqrt(x)   // square root\npow(x, y) // x to the power y\nfabs(x)   // absolute value (double)\nfloor(x)  // round down\nceil(x)   // round up\nround(x)  // round to nearest\nsin/cos/tan(x) // trig (radians)\nlog(x)    // natural log\nlog2(x)   // log base 2\nlog10(x)  // log base 10"),
            ("ctype.h cheatsheet",   "#include <ctype.h>\nisdigit(c)  // '0'-'9'\nisalpha(c)  // 'a'-'z', 'A'-'Z'\nisalnum(c)  // digit or alpha\nisspace(c)  // space, tab, newline\nisupper(c)  // uppercase letter\nislower(c)  // lowercase letter\ntoupper(c)  // convert to uppercase\ntolower(c)  // convert to lowercase"),
        ],
    },
];

pub fn show_panel(ctx: &Context, open: &mut bool, state: &mut SnippetsState) {
    if !*open {
        return;
    }
    Window::new("C Snippets")
        .open(open)
        .resizable(true)
        .default_size([700.0, 450.0])
        .min_width(480.0)
        .min_height(300.0)
        .show(ctx, |ui| {
            show_contents(ui, state);
        });
}

pub fn show_contents(ui: &mut egui::Ui, state: &mut SnippetsState) {
    // Left: category list
    SidePanel::left("snippets_cats")
        .min_width(120.0)
        .max_width(180.0)
        .show_inside(ui, |ui| {
            ui.label(RichText::new("Categories").strong());
            ui.separator();
            ScrollArea::vertical().id_salt("snip_cats_scroll").show(ui, |ui| {
                for (i, cat) in CATEGORIES.iter().enumerate() {
                    let sel = state.selected_cat == i;
                    if ui.selectable_label(sel, cat.name).clicked() {
                        if state.selected_cat != i {
                            state.selected_cat = i;
                            state.selected_snippet = None;
                        }
                    }
                }
            });
        });

    // Right: snippets in selected category
    CentralPanel::default().show_inside(ui, |ui| {
        let cat = &CATEGORIES[state.selected_cat.min(CATEGORIES.len() - 1)];
        SidePanel::left("snippets_list")
            .min_width(150.0)
            .max_width(220.0)
            .show_inside(ui, |ui| {
                ui.label(RichText::new(cat.name).strong());
                ui.separator();
                ScrollArea::vertical().id_salt("snip_list_scroll").show(ui, |ui| {
                    for (i, (label, _)) in cat.items.iter().enumerate() {
                        let sel = state.selected_snippet == Some(i);
                        if ui.selectable_label(sel, *label).clicked() {
                            state.selected_snippet = Some(i);
                        }
                    }
                });
            });

        CentralPanel::default().show_inside(ui, |ui| {
            if let Some(idx) = state.selected_snippet {
                if let Some((label, code)) = cat.items.get(idx) {
                    ui.heading(*label);
                    ui.separator();
                    ScrollArea::vertical().id_salt("snip_code_scroll").show(ui, |ui| {
                        egui::Frame::new()
                            .fill(ui.visuals().extreme_bg_color)
                            .inner_margin(egui::Margin::same(8))
                            .corner_radius(egui::CornerRadius::same(4))
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(*code).monospace().size(12.0),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            });
                        ui.add_space(8.0);
                        if ui.button("Copy to clipboard").clicked() {
                            ui.ctx().copy_text(code.to_string());
                        }
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Select a snippet").color(Color32::GRAY));
                });
            }
        });
    });
}
