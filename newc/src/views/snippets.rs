use egui::{Color32, Context, RichText, ScrollArea, SidePanel, CentralPanel, Window};

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
            s!("for loop",
               "Iterate with an index counter from 0 to n-1.",
               "for (int i = 0; i < n; i++) {\n    \n}"),
            s!("while loop",
               "Loop while a condition holds; check before each iteration.",
               "while (condition) {\n    \n}"),
            s!("do-while loop",
               "Execute the body at least once, then check the condition.",
               "do {\n    \n} while (condition);"),
            s!("if / else if / else",
               "Branch on multiple mutually exclusive conditions.",
               "if (condition) {\n    \n} else if (other) {\n    \n} else {\n    \n}"),
            s!("switch statement",
               "Dispatch on an integer or enum value; always include default.",
               "switch (var) {\n    case 1:\n        break;\n    case 2:\n        break;\n    default:\n        break;\n}"),
        ],
    },
    SnippetCat {
        name: "Data Types",
        items: &[
            s!("struct definition",
               "Define a named aggregate type with typedef for clean usage.",
               "typedef struct {\n    int id;\n    char name[64];\n    double value;\n} MyStruct;"),
            s!("struct with pointer",
               "Self-referential struct — the basis for linked lists and trees.",
               "typedef struct Node {\n    int data;\n    struct Node *next;\n} Node;"),
            s!("enum definition",
               "Named integer constants; use an explicit starting value to avoid surprises.",
               "typedef enum {\n    STATE_IDLE = 0,\n    STATE_RUNNING,\n    STATE_DONE,\n} State;"),
            s!("array init (memset)",
               "Declare a fixed-size array and zero-initialise every element.",
               "int arr[100];\nmemset(arr, 0, sizeof(arr));"),
            s!("2D array",
               "Declare a rows×cols grid and initialise all cells.",
               "int grid[ROWS][COLS];\nfor (int r = 0; r < ROWS; r++) {\n    for (int c = 0; c < COLS; c++) {\n        grid[r][c] = 0;\n    }\n}"),
        ],
    },
    SnippetCat {
        name: "Pointers",
        items: &[
            s!("pointer basics",
               "Declare a pointer, take an address with &, dereference with *.",
               "int x = 42;\nint *ptr = &x;       // ptr holds address of x\nprintf(\"%d\\n\", *ptr); // dereference: prints 42\n*ptr = 100;          // modifies x through pointer"),
            s!("double pointer",
               "Pointer-to-pointer — lets a function allocate and return a pointer to the caller.",
               "void set_value(int **pp, int val) {\n    *pp = malloc(sizeof(int));\n    **pp = val;\n}\nint *p = NULL;\nset_value(&p, 42);\nprintf(\"%d\\n\", *p);\nfree(p);"),
            s!("pointer arithmetic",
               "Add an offset to a pointer to index into an array without [].",
               "int arr[] = {10, 20, 30, 40};\nint *p = arr;              // points to arr[0]\nprintf(\"%d\\n\", *(p + 2)); // prints 30 (arr[2])\np++;                       // now points to arr[1]"),
            s!("NULL check pattern",
               "Always check malloc's return value before using the pointer; set to NULL after free.",
               "int *ptr = malloc(sizeof(int));\nif (ptr == NULL) {\n    fprintf(stderr, \"malloc failed\\n\");\n    return 1;\n}\n// ... use ptr ...\nfree(ptr);\nptr = NULL;"),
            s!("function pointer",
               "Store and call a function through a pointer; typedef makes the syntax readable.",
               "int (*fn_ptr)(int, int);   // declare\nfn_ptr = my_function;      // assign\nint result = fn_ptr(a, b); // call\n\n// typedef for readability:\ntypedef int (*BinOp)(int, int);\nBinOp op = add;"),
        ],
    },
    SnippetCat {
        name: "Memory",
        items: &[
            s!("malloc / free",
               "Allocate a heap array; check for NULL; set pointer to NULL after freeing.",
               "int *arr = malloc(n * sizeof(int));\nif (arr == NULL) {\n    fprintf(stderr, \"malloc failed\\n\");\n    return 1;\n}\n// ... use arr ...\nfree(arr);\narr = NULL;"),
            s!("calloc (zero-init)",
               "Like malloc but guarantees all bytes are zero — avoids reading uninitialised data.",
               "int *arr = calloc(n, sizeof(int)); // zero-initialised\nif (arr == NULL) {\n    fprintf(stderr, \"calloc failed\\n\");\n    return 1;\n}\nfree(arr);\narr = NULL;"),
            s!("realloc",
               "Resize a heap block; save the original pointer first — realloc returns NULL on failure.",
               "arr = realloc(arr, new_size * sizeof(int));\nif (arr == NULL) {\n    fprintf(stderr, \"realloc failed\\n\");\n    // original arr is now lost — save pointer first!\n    return 1;\n}"),
            s!("qsort int",
               "Sort an integer array in-place using the standard library comparator pattern.",
               "int cmp_int(const void *a, const void *b) {\n    return *(int*)a - *(int*)b;\n}\nqsort(arr, n, sizeof(int), cmp_int);"),
            s!("qsort struct",
               "Sort an array of structs by a field — here descending by score.",
               "typedef struct { char name[64]; int score; } Record;\nint cmp_score(const void *a, const void *b) {\n    const Record *ra = (const Record *)a;\n    const Record *rb = (const Record *)b;\n    return rb->score - ra->score; // descending\n}\nqsort(records, count, sizeof(Record), cmp_score);"),
        ],
    },
    SnippetCat {
        name: "Strings",
        items: &[
            s!("safe string read",
               "Read a line from stdin with fgets; strip the trailing newline.",
               "char buf[128];\nif (fgets(buf, sizeof(buf), stdin) != NULL) {\n    buf[strcspn(buf, \"\\n\")] = '\\0'; // strip newline\n}"),
            s!("string copy safe",
               "Use snprintf or strncpy+null-terminate instead of strcpy to prevent buffer overflow.",
               "// Always use snprintf or strncpy over strcpy\nchar dst[64];\nsnprintf(dst, sizeof(dst), \"%s\", src);\n// or:\nstrncpy(dst, src, sizeof(dst) - 1);\ndst[sizeof(dst) - 1] = '\\0';"),
            s!("string tokenise",
               "Split a string by delimiter with strtok; note it modifies the original string.",
               "char str[] = \"hello,world,foo\";\nchar *token = strtok(str, \",\");\nwhile (token != NULL) {\n    printf(\"%s\\n\", token);\n    token = strtok(NULL, \",\");\n}"),
            s!("string to int (safe)",
               "Use strtol instead of atoi — it reports conversion errors via the end pointer.",
               "// atoi has no error checking — use strtol instead\nchar *end;\nlong val = strtol(str, &end, 10);\nif (end == str || *end != '\\0') {\n    fprintf(stderr, \"Invalid number: %s\\n\", str);\n} else {\n    printf(\"Value: %ld\\n\", val);\n}"),
            s!("string search",
               "Find a substring using strstr; the return value points into the original string.",
               "char *found = strstr(haystack, needle);\nif (found != NULL) {\n    printf(\"Found at index %ld\\n\", found - haystack);\n} else {\n    puts(\"Not found\");\n}"),
        ],
    },
    SnippetCat {
        name: "File I/O",
        items: &[
            s!("fopen / fclose",
               "Open a file for reading; always check for NULL and always close when done.",
               "FILE *fp = fopen(\"data.txt\", \"r\");\nif (fp == NULL) {\n    perror(\"fopen\");\n    return 1;\n}\n// ... use fp ...\nfclose(fp);"),
            s!("fgets line loop",
               "Read a text file line by line; strip newlines for clean string comparison.",
               "char line[256];\nwhile (fgets(line, sizeof(line), fp) != NULL) {\n    line[strcspn(line, \"\\n\")] = '\\0'; // strip newline\n    // process line\n}"),
            s!("fprintf to file",
               "Write formatted text to a file — same format string as printf.",
               "FILE *fp = fopen(\"output.txt\", \"w\");\nif (fp == NULL) { perror(\"fopen\"); return 1; }\nfprintf(fp, \"Name: %s, Score: %d\\n\", name, score);\nfclose(fp);"),
            s!("binary fread/fwrite",
               "Read and write structs as raw binary data — faster and exact for numeric fields.",
               "typedef struct { int id; double value; } Record;\nRecord r = {1, 3.14};\n// Write:\nFILE *fp = fopen(\"data.bin\", \"wb\");\nfwrite(&r, sizeof(Record), 1, fp);\nfclose(fp);\n// Read:\nRecord r2;\nfp = fopen(\"data.bin\", \"rb\");\nfread(&r2, sizeof(Record), 1, fp);\nfclose(fp);"),
            s!("append to file",
               "Open in append mode ('a') so writes add to the end without truncating.",
               "FILE *fp = fopen(\"log.txt\", \"a\"); // 'a' = append\nif (fp == NULL) { perror(\"fopen\"); return 1; }\nfprintf(fp, \"[LOG] %s\\n\", message);\nfclose(fp);"),
        ],
    },
    SnippetCat {
        name: "Recursion",
        items: &[
            s!("factorial",
               "Classic base-case + recursive-case pattern. Returns 1 for n ≤ 1.",
               "int factorial(int n) {\n    if (n <= 1) return 1;      // base case\n    return n * factorial(n - 1); // recursive case\n}"),
            s!("fibonacci",
               "Double recursion — exponential time but illustrates the pattern clearly.",
               "int fibonacci(int n) {\n    if (n <= 0) return 0;\n    if (n == 1) return 1;\n    return fibonacci(n - 1) + fibonacci(n - 2);\n}"),
            s!("power",
               "Raise a base to an integer exponent; handles negative exponents via reciprocal.",
               "double power(double base, int exp) {\n    if (exp == 0) return 1.0;\n    if (exp < 0)  return 1.0 / power(base, -exp);\n    return base * power(base, exp - 1);\n}"),
            s!("sum of array",
               "Recursively sum an array by processing one element per call.",
               "int sum_recursive(int arr[], int size) {\n    if (size == 0) return 0;\n    return arr[0] + sum_recursive(arr + 1, size - 1);\n}"),
            s!("binary search (rec)",
               "Halve the search space each call — O(log n). Returns index or -1 if not found.",
               "int binary_search_rec(int arr[], int low, int high, int target) {\n    if (low > high) return -1;\n    int mid = low + (high - low) / 2;\n    if (arr[mid] == target) return mid;\n    if (arr[mid] < target) return binary_search_rec(arr, mid + 1, high, target);\n    return binary_search_rec(arr, low, mid - 1, target);\n}"),
        ],
    },
    SnippetCat {
        name: "Error Handling",
        items: &[
            s!("errno + perror",
               "perror prints the system error message matching the current errno value.",
               "#include <errno.h>\n// After a failed syscall:\nif (result == -1) {\n    perror(\"operation name\"); // prints: \"operation name: <reason>\"\n    return 1;\n}"),
            s!("exit codes",
               "Conventions for process exit codes; prefer EXIT_SUCCESS/EXIT_FAILURE over literals.",
               "// Convention:\n// 0  = success\n// 1  = general error\n// 2  = misuse of command\n// Use EXIT_SUCCESS / EXIT_FAILURE from <stdlib.h>\nreturn EXIT_SUCCESS;\nreturn EXIT_FAILURE;"),
            s!("defensive NULL guard",
               "Validate pointer and size arguments at the top of every function that dereferences them.",
               "void process(int *data, int size) {\n    if (data == NULL || size <= 0) {\n        fprintf(stderr, \"process: invalid arguments\\n\");\n        return;\n    }\n    // safe to use data here\n}"),
            s!("error propagation",
               "Return 0/−1 style: propagate errors up the call stack without global state.",
               "// Return 0 on success, -1 on error (Unix style)\nint do_thing(void) {\n    if (condition_failed) {\n        fprintf(stderr, \"do_thing: reason\\n\");\n        return -1;\n    }\n    return 0;\n}\n// Caller:\nif (do_thing() != 0) {\n    return -1; // propagate\n}"),
        ],
    },
    SnippetCat {
        name: "Preprocessor",
        items: &[
            s!("include guard",
               "Prevents a header from being included more than once per translation unit.",
               "#ifndef MY_HEADER_H\n#define MY_HEADER_H\n\n// declarations here\n\n#endif /* MY_HEADER_H */"),
            s!("#pragma once",
               "Simpler alternative to the #ifndef guard — supported by all major compilers.",
               "#pragma once\n\n// declarations here\n// (simpler alternative to #ifndef guard)"),
            s!("#define constant",
               "Define named constants; prefer enum or const for type safety where possible.",
               "#define MAX_SIZE   100\n#define PI         3.14159265358979\n#define BUFFER_LEN 256"),
            s!("#define macro",
               "Function-like macros — always parenthesise arguments to avoid operator-precedence bugs.",
               "// Macro with argument (use parentheses!)\n#define MAX(a, b) ((a) > (b) ? (a) : (b))\n#define MIN(a, b) ((a) < (b) ? (a) : (b))\n#define ABS(x)    ((x) >= 0 ? (x) : -(x))"),
            s!("conditional compile",
               "Include debug-only code at compile time via a -D flag or #define.",
               "#ifdef DEBUG\n    printf(\"[DEBUG] value = %d\\n\", value);\n#endif\n\n// Compile with: gcc -DDEBUG ...\n// Or: #define DEBUG 1 at top of file"),
        ],
    },
    SnippetCat {
        name: "Functions",
        items: &[
            s!("argc/argv main",
               "Command-line argument entry point; argv[0] is the program name.",
               "int main(int argc, char *argv[]) {\n    if (argc < 2) {\n        fprintf(stderr, \"Usage: %s <arg>\\n\", argv[0]);\n        return 1;\n    }\n    printf(\"Arg 1: %s\\n\", argv[1]);\n    return 0;\n}"),
            s!("variadic function",
               "Accept a variable number of arguments using va_list from <stdarg.h>.",
               "#include <stdarg.h>\nvoid my_printf(const char *fmt, ...) {\n    va_list args;\n    va_start(args, fmt);\n    vprintf(fmt, args);\n    va_end(args);\n}"),
            s!("callback pattern",
               "Pass a function pointer as an argument to customise behaviour at call site.",
               "void apply(int arr[], int n, int (*fn)(int)) {\n    for (int i = 0; i < n; i++) {\n        arr[i] = fn(arr[i]);\n    }\n}\nint double_it(int x) { return x * 2; }\napply(arr, n, double_it);"),
            s!("generic swap",
               "Swap any two values of the same size using void* and memcpy.",
               "void swap(void *a, void *b, size_t size) {\n    char tmp[size];\n    memcpy(tmp, a, size);\n    memcpy(a, b, size);\n    memcpy(b, tmp, size);\n}\n// Usage: swap(&x, &y, sizeof(int));"),
        ],
    },
    SnippetCat {
        name: "Reference",
        items: &[
            s!("printf format specifiers",
               "Quick reference for all printf/scanf format codes with width and padding examples.",
               "// int:      %d  or  %i\n// unsigned: %u\n// float:    %f  (%.2f = 2 decimal places)\n// double:   %lf\n// char:     %c\n// string:   %s\n// pointer:  %p\n// hex:      %x (lower) / %X (upper)\n// octal:    %o\n// width:    %-10s  (left-align, width 10)\n//           %08d   (zero-pad to width 8)\nprintf(\"%-20s %8.2f\\n\", name, value);"),
            s!("string.h cheatsheet",
               "Common <string.h> functions at a glance — lengths, copies, searches, and memory ops.",
               "#include <string.h>\nstrlen(s)           // length\nstrcpy(dst, src)    // UNSAFE — use snprintf\nstrncpy(dst,src,n)  // bounded copy\nstrcmp(a, b)        // 0 = equal, <0 if a<b\nstrcat(dst, src)    // concatenate\nstrtok(s, delim)    // tokenise (modifies s)\nstrchr(s, c)        // first occurrence of c\nstrrchr(s, c)       // last occurrence of c\nstrstr(s, sub)      // find substring\nmemset(ptr,val,n)   // fill memory\nmemcpy(dst,src,n)   // copy bytes"),
            s!("math.h cheatsheet",
               "Common <math.h> functions — remember to link with -lm when compiling.",
               "#include <math.h>  // link with -lm\nsqrt(x)   // square root\npow(x, y) // x to the power y\nfabs(x)   // absolute value (double)\nfloor(x)  // round down\nceil(x)   // round up\nround(x)  // round to nearest\nsin/cos/tan(x) // trig (radians)\nlog(x)    // natural log\nlog2(x)   // log base 2\nlog10(x)  // log base 10"),
            s!("ctype.h cheatsheet",
               "Character classification and conversion functions from <ctype.h>.",
               "#include <ctype.h>\nisdigit(c)  // '0'-'9'\nisalpha(c)  // 'a'-'z', 'A'-'Z'\nisalnum(c)  // digit or alpha\nisspace(c)  // space, tab, newline\nisupper(c)  // uppercase letter\nislower(c)  // lowercase letter\ntoupper(c)  // convert to uppercase\ntolower(c)  // convert to lowercase"),
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
        .default_size([750.0, 480.0])
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

    // Right: snippet list + code pane
    CentralPanel::default().show_inside(ui, |ui| {
        let cat = &CATEGORIES[state.selected_cat.min(CATEGORIES.len() - 1)];
        SidePanel::left("snippets_list")
            .min_width(150.0)
            .max_width(220.0)
            .show_inside(ui, |ui| {
                ui.label(RichText::new(cat.name).strong());
                ui.separator();
                ScrollArea::vertical().id_salt("snip_list_scroll").show(ui, |ui| {
                    for (i, snippet) in cat.items.iter().enumerate() {
                        let sel = state.selected_snippet == Some(i);
                        if ui.selectable_label(sel, snippet.name).clicked() {
                            state.selected_snippet = Some(i);
                        }
                    }
                });
            });

        CentralPanel::default().show_inside(ui, |ui| {
            if let Some(idx) = state.selected_snippet {
                if let Some(snippet) = cat.items.get(idx) {
                    ui.heading(snippet.name);
                    ui.label(RichText::new(snippet.desc).color(Color32::GRAY).italics());
                    ui.separator();
                    ScrollArea::vertical().id_salt("snip_code_scroll").show(ui, |ui| {
                        egui::Frame::new()
                            .fill(ui.visuals().extreme_bg_color)
                            .inner_margin(egui::Margin::same(8))
                            .corner_radius(egui::CornerRadius::same(4))
                            .show(ui, |ui| {
                                let is_dark = ui.visuals().dark_mode;
                                let job = crate::highlight::highlight_c(snippet.code, is_dark, 12.0);
                                ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Extend));
                            });
                        ui.add_space(8.0);
                        if ui.button("Copy to clipboard").clicked() {
                            ui.ctx().copy_text(snippet.code.to_string());
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
