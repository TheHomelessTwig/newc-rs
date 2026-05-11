// Module dependency graph — shows which modules include which headers.
// Text-based with ASCII art. Visual graphical rendering is a future enhancement.

use iced::widget::{button, column, row, scrollable, text, Space};
#[allow(unused_imports)]
use iced::{Color, Element, Length};
use newc_core::project::Project;
use std::collections::{HashMap, HashSet};

use crate::state::{AppState, Message, View};

pub fn view<'a>(_state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("Dependency Graph — {}", project.name))
            .size(18)
            .color(Color::from_rgb(0.663, 0.863, 0.463)),
        Space::new().width(Length::Fill),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let deps = build_dependency_map(project);
    let cycles = detect_cycles(&deps);

    // Module dependency list
    let mut dep_rows: Vec<Element<Message>> = Vec::new();
    let mut module_names: Vec<String> = deps.keys().cloned().collect();
    module_names.sort();

    for module in &module_names {
        let includes = &deps[module];
        dep_rows.push(
            text(format!("◆ {module}"))
                .size(13)
                .color(Color::from_rgb(0.663, 0.863, 0.463))
                .into(),
        );
        if includes.is_empty() {
            dep_rows.push(
                text("  (no module dependencies)")
                    .size(11)
                    .color(Color::from_rgb(0.5, 0.5, 0.5))
                    .into(),
            );
        } else {
            for dep in includes {
                let arrow_color = if module_names.contains(dep) {
                    Color::from_rgb(0.471, 0.863, 0.910)
                } else {
                    Color::from_rgb(0.5, 0.5, 0.5)
                };
                dep_rows.push(
                    text(format!("  └─ #include \"{dep}.h\""))
                        .size(12)
                        .font(iced::Font::MONOSPACE)
                        .color(arrow_color)
                        .into(),
                );
            }
        }
    }

    // Circular dependency detection
    let cycle_section: Element<Message> = if cycles.is_empty() {
        text("✓ No circular dependencies detected")
            .color(Color::from_rgb(0.392, 0.863, 0.392))
            .into()
    } else {
        let rows: Vec<Element<Message>> = cycles.iter().map(|cycle| {
            text(format!("⚠ Cycle: {}", cycle.join(" → ")))
                .size(12)
                .color(Color::from_rgb(1.0, 0.376, 0.533))
                .into()
        }).collect();
        column![
            text("⚠ Circular dependencies:").color(Color::from_rgb(1.0, 0.376, 0.533)),
            column(rows).spacing(2),
        ]
        .spacing(4)
        .into()
    };

    // Build stats summary
    let total_deps: usize = deps.values().map(|v| v.len()).sum();
    let summary = text(format!(
        "{} modules, {} inter-module dependencies",
        module_names.len(), total_deps
    ))
    .size(12)
    .color(Color::from_rgb(0.5, 0.5, 0.5));

    column![
        header,
        summary,
        text("Module Dependencies").size(14).color(Color::from_rgb(0.471, 0.863, 0.910)),
        text("Cyan = module dependency  Gray = external header")
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
        scrollable(column(dep_rows).spacing(4)).height(Length::Fill),
        cycle_section,
        text("Tip: Circular dependencies between modules can cause linker issues.")
            .size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(10)
    .padding(12)
    .into()
}

fn build_dependency_map(project: &Project) -> HashMap<String, Vec<String>> {
    let _module_names: HashSet<String> = project.modules.iter().map(|m| m.name.clone()).collect();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for module in &project.modules {
        let mut deps: Vec<String> = Vec::new();
        if module.source.exists() {
            if let Ok(src) = std::fs::read_to_string(&module.source) {
                for line in src.lines() {
                    let t = line.trim();
                    if let Some(rest) = t.strip_prefix("#include \"") {
                        let header = rest.trim_end_matches('"').trim_end_matches(".h");
                        if header != module.name && !deps.contains(&header.to_string()) {
                            deps.push(header.to_string());
                        }
                    }
                }
            }
        }
        deps.sort();
        map.insert(module.name.clone(), deps);
    }

    map
}

fn detect_cycles(deps: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    let mut cycles = Vec::new();
    let names: Vec<String> = deps.keys().cloned().collect();

    for name in &names {
        // Simple 2-cycle detection (A → B → A)
        if let Some(deps_a) = deps.get(name) {
            for dep_b in deps_a {
                if let Some(deps_b) = deps.get(dep_b) {
                    if deps_b.contains(name) {
                        let cycle = vec![name.clone(), dep_b.clone(), name.clone()];
                        // Avoid duplicates
                        let rev = vec![dep_b.clone(), name.clone(), dep_b.clone()];
                        if !cycles.contains(&cycle) && !cycles.contains(&rev) {
                            cycles.push(cycle);
                        }
                    }
                }
            }
        }
    }

    cycles
}
