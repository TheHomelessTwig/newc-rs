// Interactive graphical call graph using iced Canvas.
// Nodes = function boxes, edges = call arrows, BFS layout from main().

use std::collections::{HashMap, HashSet, VecDeque};

use iced::widget::canvas::{self, Canvas, Path, Stroke};
use iced::widget::{button, column, row, text, Space};
use iced::{Color, Element, Length, Point, Rectangle, Size, mouse};
use newc_core::{project::Project, sync::extract_function_implementations};

use crate::state::{AppState, Message, View};

// ── Graph data ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub is_main: bool,
    pub is_unreachable: bool,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Default)]
pub struct CallGraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

// ── Canvas program ─────────────────────────────────────────────────────────────

pub struct CallGraphCanvas {
    pub data: CallGraphData,
    pub selected: Option<String>,
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
    pub cache: canvas::Cache,
}

#[derive(Debug, Clone, Default)]
pub struct CanvasState {
    pub drag_start: Option<Point>,
}

impl canvas::Program<Message> for CallGraphCanvas {
    type State = CanvasState;

    fn draw(
        &self,
        _state: &CanvasState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<iced::Renderer>> {
        let geo = self.cache.draw(renderer, bounds.size(), |frame| {
            // Background
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Color::from_rgb8(0x1E, 0x1C, 0x1F),
            );

            let cx = bounds.width / 2.0 + self.pan_x;
            let cy = self.pan_y + 20.0;
            let z = self.zoom;

            // Draw edges first
            for edge in &self.data.edges {
                let from = self.data.nodes.iter().find(|n| n.id == edge.from);
                let to = self.data.nodes.iter().find(|n| n.id == edge.to);
                if let (Some(f), Some(t)) = (from, to) {
                    let fx = cx + f.x * z + (f.w * z) / 2.0;
                    let fy = cy + f.y * z + f.h * z;
                    let tx = cx + t.x * z + (t.w * z) / 2.0;
                    let ty = cy + t.y * z;

                    let path = Path::line(Point::new(fx, fy), Point::new(tx, ty));
                    frame.stroke(
                        &path,
                        Stroke::default()
                            .with_color(Color::from_rgba(0.471, 0.863, 0.910, 0.6))
                            .with_width(1.5 * z),
                    );

                    // Arrowhead
                    let dx = tx - fx;
                    let dy = ty - fy;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0);
                    let nx = dx / len;
                    let ny = dy / len;
                    let arrow_size = 6.0 * z;
                    let ax = tx - nx * arrow_size;
                    let ay = ty - ny * arrow_size;
                    let perp_x = -ny * arrow_size * 0.4;
                    let perp_y = nx * arrow_size * 0.4;
                    let arrow = Path::new(|b| {
                        b.move_to(Point::new(tx, ty));
                        b.line_to(Point::new(ax + perp_x, ay + perp_y));
                        b.line_to(Point::new(ax - perp_x, ay - perp_y));
                        b.close();
                    });
                    frame.fill(&arrow, Color::from_rgb(0.471, 0.863, 0.910));
                }
            }

            // Draw nodes
            for node in &self.data.nodes {
                let nx = cx + node.x * z;
                let ny = cy + node.y * z;
                let nw = node.w * z;
                let nh = node.h * z;

                let is_sel = self.selected.as_deref() == Some(&node.id);
                let fill = if node.is_main {
                    Color::from_rgb8(0x44, 0x8B, 0x3A)
                } else if node.is_unreachable {
                    Color::from_rgb8(0x7A, 0x2A, 0x3A)
                } else if is_sel {
                    Color::from_rgb8(0x4A, 0x5A, 0x8A)
                } else {
                    Color::from_rgb8(0x3D, 0x3A, 0x3F)
                };
                let border = if is_sel {
                    Color::from_rgb8(0xA9, 0xDC, 0x76)
                } else if node.is_main {
                    Color::from_rgb8(0xA9, 0xDC, 0x76)
                } else {
                    Color::from_rgba(0.6, 0.6, 0.6, 0.5)
                };

                let rect = Path::rectangle(Point::new(nx, ny), Size::new(nw, nh));
                frame.fill(&rect, fill);
                frame.stroke(&rect, Stroke::default().with_color(border).with_width(1.5));

                // Label
                let label_size = (12.0 * z).clamp(8.0, 16.0);
                let text_color = if node.is_unreachable {
                    Color::from_rgb(0.8, 0.5, 0.5)
                } else {
                    Color::WHITE
                };
                frame.fill_text(canvas::Text {
                    content: node.id.clone(),
                    position: Point::new(nx + 6.0 * z, ny + nh / 2.0 - label_size * 0.5),
                    size: label_size.into(),
                    color: text_color,
                    ..canvas::Text::default()
                });
            }
        });
        vec![geo]
    }

    fn update(
        &self,
        state: &mut CanvasState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        use iced::Event;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    state.drag_start = Some(pos);
                    let cx = bounds.width / 2.0 + self.pan_x;
                    let cy = self.pan_y + 20.0;
                    let z = self.zoom;
                    for node in &self.data.nodes {
                        let nx = cx + node.x * z;
                        let ny = cy + node.y * z;
                        let nw = node.w * z;
                        let nh = node.h * z;
                        if pos.x >= nx && pos.x <= nx + nw && pos.y >= ny && pos.y <= ny + nh {
                            return Some(canvas::Action::publish(Message::GraphNodeSelect(node.id.clone())).and_capture());
                        }
                    }
                }
                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.drag_start = None;
                None
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(start) = state.drag_start {
                    if let Some(pos) = cursor.position_in(bounds) {
                        let dx = pos.x - start.x;
                        let dy = pos.y - start.y;
                        state.drag_start = Some(pos); // frame-to-frame delta, not total
                        return Some(canvas::Action::publish(Message::GraphPan { dx, dy }).and_capture());
                    }
                }
                None
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let zoom_delta = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * 0.1,
                    mouse::ScrollDelta::Pixels { y, .. } => *y * 0.001,
                };
                Some(canvas::Action::publish(Message::GraphZoom(zoom_delta)).and_capture())
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        state: &CanvasState,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.drag_start.is_some() {
            mouse::Interaction::Grabbing
        } else {
            mouse::Interaction::default()
        }
    }
}

// ── Layout ─────────────────────────────────────────────────────────────────────

pub fn build_graph(project: &Project) -> CallGraphData {
    let call_map = build_call_map(project);
    if call_map.is_empty() {
        return CallGraphData::default();
    }

    let reachable = reachable_from(&call_map, "main");

    // BFS levels from main
    let mut levels: HashMap<String, usize> = HashMap::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    queue.push_back(("main".to_string(), 0));
    while let Some((func, level)) = queue.pop_front() {
        if levels.contains_key(&func) { continue; }
        levels.insert(func.clone(), level);
        if let Some(calls) = call_map.get(&func) {
            for callee in calls {
                queue.push_back((callee.clone(), level + 1));
            }
        }
    }

    // Add unreachable functions at bottom
    let max_level = levels.values().copied().max().unwrap_or(0);
    for func in call_map.keys() {
        if !levels.contains_key(func) {
            levels.insert(func.clone(), max_level + 2);
        }
    }

    // Group by level
    let mut by_level: HashMap<usize, Vec<String>> = HashMap::new();
    for (func, level) in &levels {
        by_level.entry(*level).or_default().push(func.clone());
    }
    for funcs in by_level.values_mut() {
        funcs.sort(); // initial stable sort
    }

    const NODE_W: f32 = 140.0;
    const NODE_H: f32 = 32.0;
    const H_GAP: f32 = 24.0;
    const V_GAP: f32 = 60.0;

    let max_lvl = by_level.keys().copied().max().unwrap_or(0);

    // Barycenter crossing reduction: order each level by average caller position
    // Do 3 downward passes then 2 upward passes
    for _ in 0..3 {
        for lvl in 1..=max_lvl {
            let prev = by_level.get(&(lvl - 1)).cloned().unwrap_or_default();
            let prev_pos: HashMap<String, f32> = prev.iter().enumerate()
                .map(|(i, f)| (f.clone(), i as f32)).collect();
            if let Some(funcs) = by_level.get_mut(&lvl) {
                funcs.sort_by(|a, b| {
                    let score = |f: &str| {
                        let callers: Vec<f32> = call_map.iter()
                            .filter(|(_, callees)| callees.iter().any(|c| c == f))
                            .filter_map(|(caller, _)| prev_pos.get(caller))
                            .cloned().collect();
                        if callers.is_empty() { f32::MAX } else { callers.iter().sum::<f32>() / callers.len() as f32 }
                    };
                    score(a).partial_cmp(&score(b)).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
    }

    let mut nodes = Vec::new();
    for level in 0..=max_lvl {
        let funcs = by_level.get(&level).cloned().unwrap_or_default();
        let count = funcs.len() as f32;
        let total_w = count * NODE_W + (count - 1.0) * H_GAP;
        for (i, func) in funcs.into_iter().enumerate() {
            let x = i as f32 * (NODE_W + H_GAP) - total_w / 2.0;
            let y = level as f32 * (NODE_H + V_GAP);
            nodes.push(GraphNode {
                is_main: func == "main",
                is_unreachable: !reachable.contains(&func),
                x,
                y,
                w: NODE_W,
                h: NODE_H,
                id: func,
            });
        }
    }

    // Build edges
    let mut edges = Vec::new();
    for (from, calls) in &call_map {
        for to in calls {
            if nodes.iter().any(|n| &n.id == to) {
                edges.push(GraphEdge { from: from.clone(), to: to.clone() });
            }
        }
    }

    CallGraphData { nodes, edges }
}

// ── View ───────────────────────────────────────────────────────────────────────

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let graph_data = build_graph(project);

    let controls = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("Call Graph — {}", project.name))
            .size(16).color(Color::from_rgb(0.671, 0.616, 0.949)),
        Space::new().width(Length::Fill),
        button(text("Reset View").size(12)).on_press(Message::GraphReset),
        button(text("Export").size(12)).on_press(Message::GraphExport),
        text("Scroll: zoom  Drag: pan  Click: select").size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let legend = row![
        colored_badge("main()", Color::from_rgb8(0x44, 0x8B, 0x3A)),
        colored_badge("reachable", Color::from_rgb8(0x3D, 0x3A, 0x3F)),
        colored_badge("unreachable", Color::from_rgb8(0x7A, 0x2A, 0x3A)),
    ]
    .spacing(12);

    let selected_info: Element<Message> = if let Some(sel) = &state.graph_selected {
        let calls = graph_data.edges.iter()
            .filter(|e| &e.from == sel)
            .map(|e| e.to.clone())
            .collect::<Vec<_>>();
        let called_by = graph_data.edges.iter()
            .filter(|e| &e.to == sel)
            .map(|e| e.from.clone())
            .collect::<Vec<_>>();
        text(format!("Selected: {}  |  Calls: {}  |  Called by: {}",
            sel,
            if calls.is_empty() { "none".to_string() } else { calls.join(", ") },
            if called_by.is_empty() { "none".to_string() } else { called_by.join(", ") },
        ))
        .size(12)
        .color(Color::from_rgb(0.471, 0.863, 0.910))
        .into()
    } else {
        text(format!("{} functions | {} calls", graph_data.nodes.len(), graph_data.edges.len()))
            .size(12).color(Color::from_rgb(0.5, 0.5, 0.5)).into()
    };

    let canvas_widget = Canvas::new(CallGraphCanvas {
        data: graph_data,
        selected: state.graph_selected.clone(),
        pan_x: state.graph_pan_x,
        pan_y: state.graph_pan_y,
        zoom: state.graph_zoom,
        cache: canvas::Cache::default(),
    })
    .width(Length::Fill)
    .height(Length::Fill);

    column![controls, legend, selected_info, canvas_widget]
        .spacing(6)
        .padding(8)
        .into()
}

fn colored_badge<'a>(label: &'a str, badge_color: Color) -> Element<'a, Message> {
    text(label).size(11).color(badge_color).into()
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn build_call_map(project: &Project) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for module in &project.modules {
        if !module.source.exists() { continue; }
        let Ok(src) = std::fs::read_to_string(&module.source) else { continue };
        let funcs = extract_function_implementations(&src);
        let names: Vec<String> = funcs.iter().map(|f| f.name.clone()).collect();
        for func in funcs {
            let calls = calls_in(&func.body, &names);
            map.insert(func.name, calls);
        }
    }

    let main_c = project.root.join("src").join("main.c");
    if let Ok(src) = std::fs::read_to_string(&main_c) {
        let funcs = extract_function_implementations(&src);
        let all: Vec<String> = map.keys().cloned()
            .chain(funcs.iter().map(|f| f.name.clone()))
            .collect();
        for func in &funcs {
            let calls = calls_in(&func.body, &all);
            map.insert(func.name.clone(), calls);
        }
    }

    map
}

fn calls_in(body: &str, known: &[String]) -> Vec<String> {
    let mut out: Vec<String> = known.iter()
        .filter(|n| *n != "main" && body.contains(&format!("{n}(")))
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}

fn reachable_from(map: &HashMap<String, Vec<String>>, root: &str) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(root.to_string());
    while let Some(f) = queue.pop_front() {
        if !visited.insert(f.clone()) { continue; }
        if let Some(calls) = map.get(&f) {
            for c in calls { queue.push_back(c.clone()); }
        }
    }
    visited
}

// ── SVG export ─────────────────────────────────────────────────────────────────

pub fn export_svg(project: &Project) -> String {
    let data = build_graph(project);
    graph_to_svg(&data)
}

pub fn graph_to_svg(data: &CallGraphData) -> String {
    if data.nodes.is_empty() {
        return "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"60\"><text x=\"10\" y=\"30\" fill=\"#fff\">No functions found.</text></svg>".to_string();
    }

    let margin = 40.0f32;
    let min_x = data.nodes.iter().map(|n| n.x).fold(f32::MAX, f32::min);
    let min_y = data.nodes.iter().map(|n| n.y).fold(f32::MAX, f32::min);
    let max_x = data.nodes.iter().map(|n| n.x + n.w).fold(f32::MIN, f32::max);
    let max_y = data.nodes.iter().map(|n| n.y + n.h).fold(f32::MIN, f32::max);

    let ox = margin - min_x;
    let oy = margin - min_y;
    let svg_w = max_x - min_x + 2.0 * margin;
    let svg_h = max_y - min_y + 2.0 * margin;

    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.0}\" height=\"{:.0}\" style=\"background:#1E1C1F\">",
        svg_w, svg_h
    );

    for edge in &data.edges {
        let f = data.nodes.iter().find(|n| n.id == edge.from);
        let t = data.nodes.iter().find(|n| n.id == edge.to);
        if let (Some(f), Some(t)) = (f, t) {
            let fx = ox + f.x + f.w / 2.0;
            let fy = oy + f.y + f.h;
            let tx = ox + t.x + t.w / 2.0;
            let ty = oy + t.y;
            out.push_str(&format!(
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#78DCE8\" stroke-width=\"1.5\" opacity=\"0.6\"/>",
                fx, fy, tx, ty
            ));
        }
    }

    for node in &data.nodes {
        let nx = ox + node.x;
        let ny = oy + node.y;
        let fill = if node.is_main { "#448B3A" } else if node.is_unreachable { "#7A2A3A" } else { "#3D3A3F" };
        let text_color = if node.is_unreachable { "#CC8080" } else { "#FCFCFA" };
        out.push_str(&format!(
            "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\" stroke=\"#888\" stroke-width=\"1.5\"/>",
            nx, ny, node.w, node.h, fill
        ));
        let escaped = node.id.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        out.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"12\" font-family=\"monospace\">{}</text>",
            nx + 6.0, ny + node.h / 2.0 + 4.0, text_color, escaped
        ));
    }

    out.push_str("</svg>");
    out
}
