use egui::{Color32, Context, RichText, ScrollArea, Window};

pub fn show_panel(ctx: &Context, open: &mut bool) {
    if !*open {
        return;
    }

    Window::new("C Snippets")
        .open(open)
        .resizable(true)
        .min_width(420.0)
        .min_height(300.0)
        .show(ctx, |ui| {
            ui.label(RichText::new("Click a snippet to copy it to clipboard.").small().color(Color32::GRAY));
            ui.separator();
            ScrollArea::vertical().show(ui, |ui| {
                for (label, code) in SNIPPETS {
                    ui.collapsing(*label, |ui| {
                        egui::Frame::new()
                            .fill(ui.visuals().extreme_bg_color)
                            .inner_margin(egui::Margin::same(6))
                            .corner_radius(egui::CornerRadius::same(4))
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(*code).monospace().size(12.0),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            });
                        if ui.button("Copy").clicked() {
                            ui.ctx().copy_text(code.to_string());
                        }
                    });
                }
            });
        });
}

const SNIPPETS: &[(&str, &str)] = &[
    ("for loop", "for (int i = 0; i < n; i++) {\n    \n}"),
    ("while loop", "while (condition) {\n    \n}"),
    ("do-while loop", "do {\n    \n} while (condition);"),
    ("switch statement", "switch (var) {\n    case 1:\n        break;\n    case 2:\n        break;\n    default:\n        break;\n}"),
    ("if / else if / else", "if (condition) {\n    \n} else if (other) {\n    \n} else {\n    \n}"),
    ("struct definition", "typedef struct {\n    int field1;\n    char field2[64];\n} MyStruct;"),
    ("malloc / free", "int *arr = malloc(n * sizeof(int));\nif (arr == NULL) {\n    fprintf(stderr, \"malloc failed\\n\");\n    return 1;\n}\n// ... use arr ...\nfree(arr);\narr = NULL;"),
    ("fopen / fclose", "FILE *fp = fopen(\"data.txt\", \"r\");\nif (fp == NULL) {\n    perror(\"fopen\");\n    return 1;\n}\n// ... use fp ...\nfclose(fp);"),
    ("fgets line loop", "char line[256];\nwhile (fgets(line, sizeof(line), fp) != NULL) {\n    // strip trailing newline\n    line[strcspn(line, \"\\n\")] = '\\0';\n    // process line\n}"),
    ("printf format cheatsheet", "// int:     %d  or  %i\n// float:   %f  (%.2f for 2 decimal places)\n// double:  %lf (%.4lf)\n// char:    %c\n// string:  %s\n// pointer: %p\n// hex:     %x  or  %X\n// width:   %-10s (left-align, width 10)\nprintf(\"%-20s %8.2f\\n\", name, value);"),
    ("string functions", "#include <string.h>\nstrlen(s)           // length\nstrcpy(dst, src)    // copy\nstrncpy(dst, src, n)// safe copy\nstrcmp(a, b)        // compare (0 = equal)\nstrcat(dst, src)    // concatenate\nstrtok(s, delim)    // tokenise\nstrchr(s, c)        // find char\nstrstr(s, sub)      // find substring"),
    ("array init (memset)", "int arr[100];\nmemset(arr, 0, sizeof(arr));"),
    ("qsort", "int cmp(const void *a, const void *b) {\n    return *(int*)a - *(int*)b;\n}\nqsort(arr, n, sizeof(int), cmp);"),
    ("function pointer", "int (*fn_ptr)(int, int);   // declare\nfn_ptr = my_function;      // assign\nint result = fn_ptr(a, b); // call"),
    ("argc/argv main", "int main(int argc, char *argv[]) {\n    if (argc < 2) {\n        fprintf(stderr, \"Usage: %s <arg>\\n\", argv[0]);\n        return 1;\n    }\n    printf(\"Arg: %s\\n\", argv[1]);\n    return 0;\n}"),
    ("enum definition", "typedef enum {\n    STATE_IDLE = 0,\n    STATE_RUNNING,\n    STATE_DONE,\n} State;"),
];
