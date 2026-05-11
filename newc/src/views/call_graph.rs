// Call flow graph — text-based BFS/DFS call tree from main()
// Phase 2: visual graphical rendering is a future enhancement.

use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{Color, Element, Length};
use newc_core::{project::Project, sync::extract_function_implementations};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::state::{AppState, Message, View};

pub fn view<'a>(_state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("Call Graph — {}", project.name))
            .size(18)
            .color(Color::from_rgb(0.671, 0.616, 0.949)),
        Space::new().width(Length::Fill),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Build call map: function → [called functions]
    let call_map = build_call_map(project);
    let reachable = reachable_from_main(&call_map);

    // Generate ASCII call tree
    let tree_lines = render_call_tree(&call_map, "main");
    let unreachable_funcs: Vec<String> = call_map.keys()
        .filter(|f| !reachable.contains(*f) && *f != "main")
        .cloned()
        .collect();

    // Call tree panel
    let tree_rows: Vec<Element<Message>> = if tree_lines.is_empty() {
        vec![text("No functions found in project.").color(Color::from_rgb(0.5, 0.5, 0.5)).into()]
    } else {
        tree_lines.into_iter().map(|(depth, name, is_leaf)| {
            let indent = "  ".repeat(depth);
            let icon = if depth == 0 { "◆" } else if is_leaf { "├─" } else { "├┬" };
            let color = if depth == 0 {
                Color::from_rgb(0.663, 0.863, 0.463)
            } else if is_leaf {
                Color::WHITE
            } else {
                Color::from_rgb(0.471, 0.863, 0.910)
            };
            text(format!("{indent}{icon} {name}"))
                .size(12)
                .font(iced::Font::MONOSPACE)
                .color(color)
                .into()
        }).collect()
    };

    // Legend
    let legend = column![
        text("Legend:").size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
        text("◆ = entry point   ├┬ = calls other functions   ├─ = leaf function")
            .size(11).font(iced::Font::MONOSPACE).color(Color::from_rgb(0.5, 0.5, 0.5)),
    ].spacing(2);

    // Unreachable section
    let dead_section: Element<Message> = if unreachable_funcs.is_empty() {
        text("✓ All functions reachable from main()")
            .color(Color::from_rgb(0.392, 0.863, 0.392))
            .into()
    } else {
        let rows: Vec<Element<Message>> = unreachable_funcs.iter().map(|f| {
            text(format!("  ⚠ {f}")).size(12).font(iced::Font::MONOSPACE)
                .color(Color::from_rgb(1.0, 0.376, 0.533)).into()
        }).collect();
        column![
            text(format!("⚠ {} unreachable function(s):", unreachable_funcs.len()))
                .color(Color::from_rgb(1.0, 0.376, 0.533)),
            column(rows).spacing(2),
        ]
        .spacing(4)
        .into()
    };

    column![
        header,
        text("Call Tree from main()").size(14).color(Color::from_rgb(0.471, 0.863, 0.910)),
        legend,
        scrollable(column(tree_rows).spacing(2)).height(Length::Fill),
        dead_section,
        text("Tip: Use Health → Dead Code to remove unreachable functions.")
            .size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(10)
    .padding(12)
    .into()
}

fn build_call_map(project: &Project) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for module in &project.modules {
        if !module.source.exists() { continue; }
        let Ok(src) = std::fs::read_to_string(&module.source) else { continue };
        let funcs = extract_function_implementations(&src);
        let func_names: Vec<String> = funcs.iter().map(|f| f.name.clone()).collect();
        for func in funcs {
            let calls = extract_calls_from_body(&func.body, &func_names);
            map.insert(func.name.clone(), calls);
        }
    }

    // Add main.c
    let main_c = project.root.join("src").join("main.c");
    if let Ok(src) = std::fs::read_to_string(&main_c) {
        let funcs = extract_function_implementations(&src);
        for func in &funcs {
            let all_names: Vec<String> = map.keys().cloned().chain(funcs.iter().map(|f| f.name.clone())).collect();
            let calls = extract_calls_from_body(&func.body, &all_names);
            map.insert(func.name.clone(), calls);
        }
    }

    map
}

fn extract_calls_from_body(body: &str, known_funcs: &[String]) -> Vec<String> {
    let mut calls = Vec::new();
    for fname in known_funcs {
        if fname == "main" { continue; }
        if body.contains(&format!("{fname}(")) {
            calls.push(fname.clone());
        }
    }
    calls.sort();
    calls.dedup();
    calls
}

fn reachable_from_main(call_map: &HashMap<String, Vec<String>>) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back("main".to_string());
    while let Some(func) = queue.pop_front() {
        if visited.contains(&func) { continue; }
        visited.insert(func.clone());
        if let Some(calls) = call_map.get(&func) {
            for callee in calls {
                queue.push_back(callee.clone());
            }
        }
    }
    visited
}

fn render_call_tree(call_map: &HashMap<String, Vec<String>>, root: &str) -> Vec<(usize, String, bool)> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    render_subtree(call_map, root, 0, &mut visited, &mut result);
    result
}

fn render_subtree(
    call_map: &HashMap<String, Vec<String>>,
    func: &str,
    depth: usize,
    visited: &mut HashSet<String>,
    result: &mut Vec<(usize, String, bool)>,
) {
    if depth > 8 { return; } // cap depth to avoid cycles
    let calls = call_map.get(func).map(|v| v.as_slice()).unwrap_or(&[]);
    let is_leaf = calls.is_empty() || visited.contains(func);
    result.push((depth, func.to_string(), is_leaf));

    if !visited.contains(func) && !calls.is_empty() {
        visited.insert(func.to_string());
        for callee in calls {
            render_subtree(call_map, callee, depth + 1, visited, result);
        }
    }
}
