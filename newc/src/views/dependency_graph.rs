//! Interactive module dependency graph canvas — modules as nodes, `#include` relationships as edges.

// Interactive graphical dependency graph using iced Canvas.
// Modules as nodes, #include dependencies as directed edges.

use std::collections::{HashMap, HashSet};

use iced::widget::canvas::{self, Canvas, Path, Stroke};
use iced::widget::{button, column, row, text, Space};
use iced::{Color, Element, Length, Point, Rectangle, Size, mouse};
use newc_core::project::Project;

use crate::state::{AppState, Message, View};
use crate::views::call_graph::{GraphNode, GraphEdge, CallGraphData, CanvasState};

// ── Canvas program ─────────────────────────────────────────────────────────────

/// iced `Canvas` program that renders the module dependency graph with pan, zoom, and selection.
pub struct DepGraphCanvas {
    pub data: CallGraphData,
    pub selected: Option<String>,
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
    pub cache: canvas::Cache,
}

impl canvas::Program<Message> for DepGraphCanvas {
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
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgb8(0x1E, 0x1C, 0x1F));

            let cx = bounds.width / 2.0 + self.pan_x;
            let cy = bounds.height / 2.0 + self.pan_y;
            let z = self.zoom;

            // Edges
            for edge in &self.data.edges {
                let from = self.data.nodes.iter().find(|n| n.id == edge.from);
                let to = self.data.nodes.iter().find(|n| n.id == edge.to);
                if let (Some(f), Some(t)) = (from, to) {
                    let fx = cx + f.x * z + (f.w * z) / 2.0;
                    let fy = cy + f.y * z + (f.h * z) / 2.0;
                    let tx = cx + t.x * z + (t.w * z) / 2.0;
                    let ty = cy + t.y * z + (t.h * z) / 2.0;

                    let path = Path::line(Point::new(fx, fy), Point::new(tx, ty));
                    frame.stroke(&path, Stroke::default()
                        .with_color(Color::from_rgba(0.471, 0.863, 0.910, 0.5))
                        .with_width(1.5 * z));

                    // Arrowhead
                    let dx = tx - fx;
                    let dy = ty - fy;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0);
                    let nx = dx / len;
                    let ny = dy / len;
                    let as_ = 6.0 * z;
                    let ax = tx - nx * as_;
                    let ay = ty - ny * as_;
                    let px = -ny * as_ * 0.4;
                    let py = nx * as_ * 0.4;
                    let arrow = Path::new(|b| {
                        b.move_to(Point::new(tx, ty));
                        b.line_to(Point::new(ax + px, ay + py));
                        b.line_to(Point::new(ax - px, ay - py));
                        b.close();
                    });
                    frame.fill(&arrow, Color::from_rgb(0.471, 0.863, 0.910));
                }
            }

            // Nodes
            for node in &self.data.nodes {
                let nx = cx + node.x * z;
                let ny = cy + node.y * z;
                let nw = node.w * z;
                let nh = node.h * z;
                let is_sel = self.selected.as_deref() == Some(&node.id);

                let fill = if is_sel {
                    Color::from_rgb8(0x4A, 0x5A, 0x8A)
                } else {
                    Color::from_rgb8(0x3D, 0x3A, 0x3F)
                };
                let border = if is_sel {
                    Color::from_rgb8(0xFF, 0xD8, 0x66)
                } else {
                    Color::from_rgb8(0xFC, 0xFC, 0xFA)
                };

                let rect = Path::rectangle(Point::new(nx, ny), Size::new(nw, nh));
                frame.fill(&rect, fill);
                frame.stroke(&rect, Stroke::default().with_color(border).with_width(1.5));

                let fs = (12.0 * z).clamp(8.0, 16.0);
                frame.fill_text(canvas::Text {
                    content: format!("◆ {}", node.id),
                    position: Point::new(nx + 6.0 * z, ny + nh / 2.0 - fs * 0.5),
                    size: fs.into(),
                    color: Color::from_rgb8(0xA9, 0xDC, 0x76),
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
                    let cy = bounds.height / 2.0 + self.pan_y;
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
                        state.drag_start = Some(pos);
                        return Some(canvas::Action::publish(Message::GraphPan { dx, dy }).and_capture());
                    }
                }
                None
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let d = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * 0.1,
                    mouse::ScrollDelta::Pixels { y, .. } => *y * 0.001,
                };
                Some(canvas::Action::publish(Message::GraphZoom(d)).and_capture())
            }
            _ => None,
        }
    }

    fn mouse_interaction(&self, state: &CanvasState, _bounds: Rectangle, _cursor: mouse::Cursor) -> mouse::Interaction {
        if state.drag_start.is_some() { mouse::Interaction::Grabbing } else { mouse::Interaction::default() }
    }
}

// ── Build dependency graph data ────────────────────────────────────────────────

pub fn build_dep_graph(project: &Project) -> CallGraphData {
    let dep_map = build_dep_map(project);
    if dep_map.is_empty() { return CallGraphData::default(); }

    let modules: Vec<String> = {
        let mut v: Vec<String> = dep_map.keys().cloned().collect();
        v.sort();
        v
    };

    // Circular layout
    const NODE_W: f32 = 150.0;
    const NODE_H: f32 = 36.0;
    let n = modules.len() as f32;
    let radius = (n * 100.0).max(200.0);
    use std::f32::consts::PI;

    let nodes: Vec<GraphNode> = modules.iter().enumerate().map(|(i, name)| {
        let angle = (i as f32 / n) * 2.0 * PI - PI / 2.0;
        let x = radius * angle.cos() - NODE_W / 2.0;
        let y = radius * angle.sin() - NODE_H / 2.0;
        GraphNode { id: name.clone(), x, y, w: NODE_W, h: NODE_H, is_main: false, is_unreachable: false }
    }).collect();

    let edges: Vec<GraphEdge> = dep_map.iter().flat_map(|(from, deps)| {
        deps.iter()
            .filter(|d| modules.contains(d))
            .map(|to| GraphEdge { from: from.clone(), to: to.clone() })
            .collect::<Vec<_>>()
    }).collect();

    CallGraphData { nodes, edges }
}

fn build_dep_map(project: &Project) -> HashMap<String, Vec<String>> {
    let module_names: HashSet<String> = project.modules.iter().map(|m| m.name.clone()).collect();
    let mut map = HashMap::new();
    for module in &project.modules {
        let mut deps = Vec::new();
        if module.source.exists() {
            if let Ok(src) = std::fs::read_to_string(&module.source) {
                for line in src.lines() {
                    let t = line.trim();
                    if let Some(rest) = t.strip_prefix("#include \"") {
                        let hdr = rest.trim_end_matches('"').trim_end_matches(".h").to_string();
                        if hdr != module.name && module_names.contains(&hdr) && !deps.contains(&hdr) {
                            deps.push(hdr);
                        }
                    }
                }
            }
        }
        map.insert(module.name.clone(), deps);
    }
    map
}

// ── SVG export ─────────────────────────────────────────────────────────────────

/// Exports the dependency graph for `project` as an SVG string.
pub fn export_svg(project: &Project) -> String {
    let data = build_dep_graph(project);
    crate::views::call_graph::graph_to_svg(&data)
}

// ── View ───────────────────────────────────────────────────────────────────────

/// Renders the interactive module dependency graph screen for the given project.
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let graph_data = build_dep_graph(project);

    let controls = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("Dependency Graph — {}", project.name))
            .size(16).color(Color::from_rgb(0.663, 0.863, 0.463)),
        Space::new().width(Length::Fill),
        button(text("Reset").size(12)).on_press(Message::GraphReset),
        button(text("Export").size(12)).on_press(Message::GraphExport),
        text("Scroll: zoom  Drag: pan  Click: select").size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let info: Element<Message> = if let Some(sel) = &state.graph_selected {
        let dep_map = build_dep_map(project);
        let deps = dep_map.get(sel).cloned().unwrap_or_default();
        let depended_by: Vec<String> = dep_map.iter()
            .filter(|(_, v)| v.contains(sel))
            .map(|(k, _)| k.clone())
            .collect();
        text(format!("{}  →  deps: {}  |  used by: {}",
            sel,
            if deps.is_empty() { "none".into() } else { deps.join(", ") },
            if depended_by.is_empty() { "none".into() } else { depended_by.join(", ") },
        ))
        .size(12).color(Color::from_rgb(0.471, 0.863, 0.910)).into()
    } else {
        text(format!("{} modules, {} dependencies", graph_data.nodes.len(), graph_data.edges.len()))
            .size(12).color(Color::from_rgb(0.5, 0.5, 0.5)).into()
    };

    let canvas_widget = Canvas::new(DepGraphCanvas {
        data: graph_data,
        selected: state.graph_selected.clone(),
        pan_x: state.graph_pan_x,
        pan_y: state.graph_pan_y,
        zoom: state.graph_zoom,
        cache: canvas::Cache::default(),
    })
    .width(Length::Fill)
    .height(Length::Fill);

    column![controls, info, canvas_widget]
        .spacing(6)
        .padding(8)
        .into()
}
