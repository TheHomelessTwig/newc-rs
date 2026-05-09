use crate::main_builder::{GlobalVar, MainBlock, MainBuilderState};

#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub modules: &'static [&'static str],
    pub builder: fn() -> MainBuilderState,
}

pub fn all_templates() -> &'static [ProjectTemplate] {
    &[
        ProjectTemplate {
            name: "Calculator",
            description: "Double-precision calculator with menu, input validation and formatted output",
            modules: &["input", "math", "display"],
            builder: calculator_builder,
        },
        ProjectTemplate {
            name: "Array Processor",
            description: "Read a user-defined int array, then sort, search and display statistics",
            modules: &["input", "math", "array", "display"],
            builder: array_processor_builder,
        },
        ProjectTemplate {
            name: "Grade Manager",
            description: "Store and analyse student grades; compute average, min, max and pass/fail",
            modules: &["input", "math", "display"],
            builder: grade_manager_builder,
        },
    ]
}

// ── Template builders ─────────────────────────────────────────────────────────

fn calculator_builder() -> MainBuilderState {
    let blocks = vec![
        MainBlock::FunctionCall {
            func_name: "print_header".into(),
            args: vec!["\"Calculator\"".into()],
            assign_to: String::new(),
            comment: String::new(),
        },
        MainBlock::BlankLine,
        MainBlock::VarDecl {
            type_name: "double".into(), name: "a".into(),
            init: String::new(), is_array: false, array_size: String::new(),
        },
        MainBlock::VarDecl {
            type_name: "double".into(), name: "b".into(),
            init: String::new(), is_array: false, array_size: String::new(),
        },
        MainBlock::VarDecl {
            type_name: "char".into(), name: "op".into(),
            init: String::new(), is_array: false, array_size: String::new(),
        },
        MainBlock::BlankLine,
        MainBlock::FunctionCall {
            func_name: "prompt_for_double".into(),
            args: vec!["\"Enter first number: \"".into()],
            assign_to: "a".into(),
            comment: String::new(),
        },
        MainBlock::FunctionCall {
            func_name: "prompt_for_double".into(),
            args: vec!["\"Enter second number: \"".into()],
            assign_to: "b".into(),
            comment: String::new(),
        },
        MainBlock::FunctionCall {
            func_name: "prompt_for_char".into(),
            args: vec!["\"Operator (+, -, *, /): \"".into()],
            assign_to: "op".into(),
            comment: String::new(),
        },
        MainBlock::BlankLine,
        MainBlock::Comment("TODO: switch(op) to compute result".into()),
        MainBlock::BlankLine,
        MainBlock::FunctionCall {
            func_name: "press_enter_to_exit".into(),
            args: vec![],
            assign_to: String::new(),
            comment: String::new(),
        },
    ];
    MainBuilderState { blocks, globals: Vec::new(), includes: vec!["input".into(), "math".into(), "display".into()] }
}

fn array_processor_builder() -> MainBuilderState {
    let globals = vec![
        GlobalVar {
            type_name: "int".into(), name: "SIZE".into(),
            init: "10".into(), is_array: false, array_size: String::new(), is_static: false,
        },
    ];
    let blocks = vec![
        MainBlock::FunctionCall {
            func_name: "print_header".into(),
            args: vec!["\"Array Processor\"".into()],
            assign_to: String::new(),
            comment: String::new(),
        },
        MainBlock::BlankLine,
        MainBlock::VarDecl {
            type_name: "int".into(), name: "arr".into(),
            init: String::new(), is_array: true, array_size: "SIZE".into(),
        },
        MainBlock::BlankLine,
        MainBlock::FunctionCall {
            func_name: "array_input_int".into(),
            args: vec!["arr".into(), "SIZE".into(), "\"Enter value\"".into(), "1".into()],
            assign_to: String::new(),
            comment: "Fill array".into(),
        },
        MainBlock::FunctionCall {
            func_name: "array_sort_asc".into(),
            args: vec!["arr".into(), "SIZE".into()],
            assign_to: String::new(),
            comment: String::new(),
        },
        MainBlock::FunctionCall {
            func_name: "array_print".into(),
            args: vec!["arr".into(), "SIZE".into()],
            assign_to: String::new(),
            comment: String::new(),
        },
        MainBlock::BlankLine,
        MainBlock::Comment("Statistics".into()),
        MainBlock::VarDecl {
            type_name: "int".into(), name: "sum".into(),
            init: String::new(), is_array: false, array_size: String::new(),
        },
        MainBlock::FunctionCall {
            func_name: "array_sum".into(),
            args: vec!["arr".into(), "SIZE".into()],
            assign_to: "sum".into(),
            comment: String::new(),
        },
    ];
    MainBuilderState { blocks, globals, includes: vec!["input".into(), "math".into(), "array".into(), "display".into()] }
}

fn grade_manager_builder() -> MainBuilderState {
    let globals = vec![
        GlobalVar {
            type_name: "int".into(), name: "NUM_STUDENTS".into(),
            init: "5".into(), is_array: false, array_size: String::new(), is_static: false,
        },
    ];
    let blocks = vec![
        MainBlock::FunctionCall {
            func_name: "print_header".into(),
            args: vec!["\"Grade Manager\"".into()],
            assign_to: String::new(),
            comment: String::new(),
        },
        MainBlock::BlankLine,
        MainBlock::VarDecl {
            type_name: "double".into(), name: "grades".into(),
            init: String::new(), is_array: true, array_size: "NUM_STUDENTS".into(),
        },
        MainBlock::BlankLine,
        MainBlock::FunctionCall {
            func_name: "array_input_double".into(),
            args: vec!["grades".into(), "NUM_STUDENTS".into(), "\"Enter grade\"".into(), "1".into()],
            assign_to: String::new(),
            comment: "Collect grades".into(),
        },
        MainBlock::BlankLine,
        MainBlock::VarDecl {
            type_name: "double".into(), name: "avg".into(),
            init: String::new(), is_array: false, array_size: String::new(),
        },
        MainBlock::FunctionCall {
            func_name: "array_average_double".into(),
            args: vec!["grades".into(), "NUM_STUDENTS".into()],
            assign_to: "avg".into(),
            comment: String::new(),
        },
        MainBlock::Comment("TODO: print avg, min, max, pass/fail count".into()),
        MainBlock::BlankLine,
        MainBlock::FunctionCall {
            func_name: "press_enter_to_exit".into(),
            args: vec![],
            assign_to: String::new(),
            comment: String::new(),
        },
    ];
    MainBuilderState { blocks, globals, includes: vec!["input".into(), "math".into(), "display".into()] }
}
