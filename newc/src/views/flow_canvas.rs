//! Flowchart canvas — renders the `main()` composer blocks as a zoomable/pannable flowchart.

use iced::widget::canvas::{self, Canvas, Path, Stroke};
use iced::widget::{Space, button, column, row, text};
use iced::{Color, Element, Length, Point, Rectangle, Size, mouse};
use newc_core::main_builder::{MainBlock, MainBuilderState};
use newc_core::project::Project;

use crate::state::{AppState, Message, View};
use crate::theme as th;
use crate::views::call_graph::CanvasState;

const BOX_W: f32 = 180.0;
const BOX_H: f32 = 40.0;
const DIAMOND_W: f32 = 200.0;
const V_GAP: f32 = 28.0;
// Horizontal offset for YES/NO branches from the centre of a decision node.
const BRANCH_X: f32 = 130.0;

#[derive(Debug, Clone)]
struct FlowNode {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: String,
    shape: NodeShape,
}

#[derive(Debug, Clone, PartialEq)]
enum NodeShape {
    Rect,
    Diamond,
    Rounded,
}

struct FlowCanvas {
    nodes: Vec<FlowNode>,
    /// Explicit directed edges `(from_idx, to_idx)` drawn as arrows.
    edges: Vec<(usize, usize)>,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
    cache: canvas::Cache,
}

impl canvas::Program<Message> for FlowCanvas {
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
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), th::color::bg_deep());
            let cx = bounds.width / 2.0 + self.pan_x;
            let z = self.zoom;

            // Draw explicit edges
            for &(fi, ti) in &self.edges {
                let from_node = &self.nodes[fi];
                let to_node = &self.nodes[ti];
                let fx = cx + (from_node.x + from_node.w / 2.0) * z;
                let fy = self.pan_y + (from_node.y + from_node.h) * z;
                let tx = cx + (to_node.x + to_node.w / 2.0) * z;
                let ty = self.pan_y + to_node.y * z;

                // For loop-back edges (going upward), use an elbow route
                if ty < fy - 4.0 {
                    // Right-elbow: down from source, across, up to target
                    let elbow_x = cx + (from_node.x + from_node.w + 16.0) * z;
                    let mid_y = fy + 10.0 * z;
                    let path = Path::new(|b| {
                        b.move_to(Point::new(fx, fy));
                        b.line_to(Point::new(fx, mid_y));
                        b.line_to(Point::new(elbow_x, mid_y));
                        b.line_to(Point::new(elbow_x, ty));
                        b.line_to(Point::new(tx, ty));
                    });
                    frame.stroke(
                        &path,
                        Stroke::default()
                            .with_color(Color::from_rgb8(0x6A, 0x6A, 0x8A))
                            .with_width(1.5 * z),
                    );
                } else {
                    let path = Path::line(Point::new(fx, fy), Point::new(tx, ty));
                    frame.stroke(
                        &path,
                        Stroke::default()
                            .with_color(Color::from_rgb8(0x7A, 0x7A, 0x8A))
                            .with_width(1.5 * z),
                    );
                }

                // Arrowhead pointing into the target node
                let as_ = 6.0 * z;
                let ah = Path::new(|b| {
                    b.move_to(Point::new(tx, ty));
                    b.line_to(Point::new(tx - as_ * 0.4, ty - as_));
                    b.line_to(Point::new(tx + as_ * 0.4, ty - as_));
                    b.close();
                });
                frame.fill(&ah, Color::from_rgb8(0x7A, 0x7A, 0x8A));
            }

            // Draw nodes
            for node in &self.nodes {
                let nx = cx + node.x * z;
                let ny = self.pan_y + node.y * z;
                let nw = node.w * z;
                let nh = node.h * z;
                let fs = (11.0 * z).clamp(8.0, 15.0);

                let (fill, border, text_color) = match node.shape {
                    NodeShape::Rounded => (
                        Color::from_rgb8(0x44, 0x44, 0x66),
                        th::color::green(),
                        th::color::green(),
                    ),
                    NodeShape::Diamond => (
                        Color::from_rgb8(0x3A, 0x4A, 0x6A),
                        th::color::yellow(),
                        th::color::yellow(),
                    ),
                    NodeShape::Rect => (
                        Color::from_rgb8(0x3D, 0x3A, 0x3F),
                        th::color::text(),
                        Color::WHITE,
                    ),
                };

                match node.shape {
                    NodeShape::Diamond => {
                        let dcx = nx + nw / 2.0;
                        let dcy = ny + nh / 2.0;
                        let diamond = Path::new(|b| {
                            b.move_to(Point::new(dcx, ny));
                            b.line_to(Point::new(nx + nw, dcy));
                            b.line_to(Point::new(dcx, ny + nh));
                            b.line_to(Point::new(nx, dcy));
                            b.close();
                        });
                        frame.fill(&diamond, fill);
                        frame.stroke(
                            &diamond,
                            Stroke::default().with_color(border).with_width(1.5),
                        );
                    }
                    NodeShape::Rounded => {
                        let rect = Path::rectangle(Point::new(nx, ny), Size::new(nw, nh));
                        frame.fill(&rect, fill);
                        frame.stroke(&rect, Stroke::default().with_color(border).with_width(2.0));
                    }
                    NodeShape::Rect => {
                        let rect = Path::rectangle(Point::new(nx, ny), Size::new(nw, nh));
                        frame.fill(&rect, fill);
                        frame.stroke(&rect, Stroke::default().with_color(border).with_width(1.0));
                    }
                }

                let max_chars = (nw / (fs * 0.6)) as usize;
                let label = if node.label.len() > max_chars && max_chars > 3 {
                    format!("{}…", &node.label[..max_chars.saturating_sub(1)])
                } else {
                    node.label.clone()
                };

                frame.fill_text(canvas::Text {
                    content: label,
                    position: Point::new(nx + nw * 0.05, ny + nh / 2.0 - fs * 0.5),
                    size: fs.into(),
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
                }
                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.drag_start = None;
                None
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(start) = state.drag_start
                    && let Some(pos) = cursor.position_in(bounds)
                {
                    let dx = pos.x - start.x;
                    let dy = pos.y - start.y;
                    state.drag_start = Some(pos);
                    return Some(
                        canvas::Action::publish(Message::GraphPan { dx, dy }).and_capture(),
                    );
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

/// Build the node list and explicit edge list from the composer state.
fn build_flowchart(builder: &MainBuilderState) -> (Vec<FlowNode>, Vec<(usize, usize)>) {
    let mut nodes: Vec<FlowNode> = Vec::new();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut y = 0.0f32;

    nodes.push(FlowNode {
        x: -BOX_W / 2.0,
        y,
        w: BOX_W,
        h: BOX_H,
        label: "START".into(),
        shape: NodeShape::Rounded,
    });
    y += BOX_H + V_GAP;

    let exits = push_blocks(
        &builder.blocks,
        &mut nodes,
        &mut edges,
        &mut y,
        0.0,
        vec![0],
    );

    let end_idx = nodes.len();
    nodes.push(FlowNode {
        x: -BOX_W / 2.0,
        y,
        w: BOX_W,
        h: BOX_H,
        label: "return 0".into(),
        shape: NodeShape::Rounded,
    });
    for e in exits {
        edges.push((e, end_idx));
    }

    (nodes, edges)
}

/// Recursively lay out `blocks`, appending to `nodes`/`edges`.
///
/// `prev_exits` — node indices whose output arrow should connect to the first node of this sequence.
/// Returns exit node indices that need to connect to whatever comes after this sequence.
fn push_blocks(
    blocks: &[MainBlock],
    nodes: &mut Vec<FlowNode>,
    edges: &mut Vec<(usize, usize)>,
    y: &mut f32,
    indent: f32,
    mut prev_exits: Vec<usize>,
) -> Vec<usize> {
    for block in blocks {
        match block {
            MainBlock::Comment(_) | MainBlock::BlankLine => continue,

            MainBlock::IfBlock {
                condition,
                body,
                else_body,
            } => {
                // Decision diamond
                let d_x = -DIAMOND_W / 2.0 + indent;
                let diamond_idx = nodes.len();
                nodes.push(FlowNode {
                    x: d_x,
                    y: *y,
                    w: DIAMOND_W,
                    h: BOX_H,
                    label: format!("if {condition}"),
                    shape: NodeShape::Diamond,
                });
                for e in prev_exits {
                    edges.push((e, diamond_idx));
                }
                *y += BOX_H + V_GAP;

                let branch_y = *y;
                let mut all_exits: Vec<usize> = Vec::new();

                // YES branch — right side
                let yes_x_offset = indent + BRANCH_X;
                if !body.is_empty() {
                    let lbl_idx = nodes.len();
                    nodes.push(FlowNode {
                        x: yes_x_offset - BOX_W / 2.0,
                        y: *y,
                        w: BOX_W,
                        h: 20.0,
                        label: "YES →".into(),
                        shape: NodeShape::Rect,
                    });
                    edges.push((diamond_idx, lbl_idx));
                    let mut yes_y = branch_y + 20.0 + V_GAP / 2.0;
                    let yes_exits =
                        push_blocks(body, nodes, edges, &mut yes_y, yes_x_offset, vec![lbl_idx]);
                    all_exits.extend(yes_exits);
                    if yes_y > *y {
                        *y = yes_y;
                    }
                } else {
                    all_exits.push(diamond_idx);
                }

                // NO / else branch — left side
                let no_x_offset = indent - BRANCH_X;
                if !else_body.is_empty() {
                    let lbl_idx = nodes.len();
                    nodes.push(FlowNode {
                        x: no_x_offset - BOX_W / 2.0,
                        y: branch_y,
                        w: BOX_W,
                        h: 20.0,
                        label: "← NO".into(),
                        shape: NodeShape::Rect,
                    });
                    edges.push((diamond_idx, lbl_idx));
                    let mut no_y = branch_y + 20.0 + V_GAP / 2.0;
                    let no_exits = push_blocks(
                        else_body,
                        nodes,
                        edges,
                        &mut no_y,
                        no_x_offset,
                        vec![lbl_idx],
                    );
                    all_exits.extend(no_exits);
                    if no_y > *y {
                        *y = no_y;
                    }
                } else {
                    // No else: diamond itself is the "false" exit (falls through)
                    all_exits.push(diamond_idx);
                }

                *y += V_GAP;
                prev_exits = all_exits;
            }

            MainBlock::WhileLoop { condition, body } => {
                let d_x = -DIAMOND_W / 2.0 + indent;
                let diamond_idx = nodes.len();
                nodes.push(FlowNode {
                    x: d_x,
                    y: *y,
                    w: DIAMOND_W,
                    h: BOX_H,
                    label: format!("while {condition}"),
                    shape: NodeShape::Diamond,
                });
                for e in prev_exits {
                    edges.push((e, diamond_idx));
                }
                *y += BOX_H + V_GAP;

                if !body.is_empty() {
                    let body_exits =
                        push_blocks(body, nodes, edges, y, indent + BRANCH_X, vec![diamond_idx]);
                    // Loop-back: body exits → diamond
                    for e in body_exits {
                        edges.push((e, diamond_idx));
                    }
                }

                *y += V_GAP;
                // Loop exit when condition is false
                prev_exits = vec![diamond_idx];
            }

            MainBlock::ForLoop {
                init,
                condition,
                increment,
                body,
            } => {
                let d_x = -DIAMOND_W / 2.0 + indent;
                let for_label = format!("for({init}; {condition}; {increment})");
                let diamond_idx = nodes.len();
                nodes.push(FlowNode {
                    x: d_x,
                    y: *y,
                    w: DIAMOND_W + 40.0,
                    h: BOX_H,
                    label: for_label,
                    shape: NodeShape::Diamond,
                });
                for e in prev_exits {
                    edges.push((e, diamond_idx));
                }
                *y += BOX_H + V_GAP;

                if !body.is_empty() {
                    let body_exits =
                        push_blocks(body, nodes, edges, y, indent + BRANCH_X, vec![diamond_idx]);
                    for e in body_exits {
                        edges.push((e, diamond_idx));
                    }
                }

                *y += V_GAP;
                prev_exits = vec![diamond_idx];
            }

            block => {
                let (label, shape, w) = block_to_flow(block);
                let x = -w / 2.0 + indent;
                let node_idx = nodes.len();
                nodes.push(FlowNode {
                    x,
                    y: *y,
                    w,
                    h: BOX_H,
                    label,
                    shape,
                });
                for e in prev_exits {
                    edges.push((e, node_idx));
                }
                *y += BOX_H + V_GAP;
                prev_exits = vec![node_idx];
            }
        }
    }
    prev_exits
}

fn block_to_flow(block: &MainBlock) -> (String, NodeShape, f32) {
    match block {
        MainBlock::VarDecl {
            type_name,
            name,
            init,
            ..
        } => {
            let label = if init.is_empty() {
                format!("{type_name} {name}")
            } else {
                format!("{type_name} {name} = {init}")
            };
            (label, NodeShape::Rect, BOX_W)
        }
        MainBlock::FunctionCall {
            func_name,
            args,
            assign_to,
            ..
        } => {
            let call = format!("{}({})", func_name, args.join(", "));
            let label = if assign_to.is_empty() {
                call
            } else {
                format!("{assign_to} = {call}")
            };
            (label, NodeShape::Rect, BOX_W)
        }
        MainBlock::RawCode(r) => {
            let preview = if r.len() > 30 { &r[..30] } else { r };
            (preview.to_string(), NodeShape::Rect, BOX_W)
        }
        MainBlock::Comment(c) => (format!("// {c}"), NodeShape::Rect, BOX_W),
        _ => ("…".to_string(), NodeShape::Rect, BOX_W),
    }
}

/// Renders the interactive flowchart screen derived from the current `MainBuilderState`.
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let (nodes, edges) = build_flowchart(&state.main_builder);
    let block_count = state
        .main_builder
        .blocks
        .iter()
        .filter(|b| !matches!(b, MainBlock::BlankLine | MainBlock::Comment(_)))
        .count();

    let controls = row![
        button(text("← Composer")).on_press(Message::Navigate(View::MainBuilder(project.clone()))),
        text(format!("Flowchart — {}", project.name))
            .size(16)
            .color(th::color::yellow()),
        Space::new().width(Length::Fill),
        button(text("Reset").size(12)).on_press(Message::GraphReset),
        text(format!("{block_count} blocks"))
            .size(11)
            .color(th::color::text_dim()),
        text("Drag: pan  Scroll: zoom")
            .size(11)
            .color(th::color::text_dim()),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let legend = row![
        text("▬ Process").size(11).color(Color::WHITE),
        text("◇ Decision").size(11).color(th::color::yellow()),
        text("▬ Start/End").size(11).color(th::color::green()),
        text("YES → branches right   ← NO branches left")
            .size(11)
            .color(th::color::text_dim()),
    ]
    .spacing(16);

    let canvas_widget = Canvas::new(FlowCanvas {
        nodes,
        edges,
        pan_x: state.graph_pan_x,
        pan_y: state.graph_pan_y + 40.0,
        zoom: state.graph_zoom,
        cache: canvas::Cache::default(),
    })
    .width(Length::Fill)
    .height(Length::Fill);

    column![controls, legend, canvas_widget]
        .spacing(6)
        .padding(8)
        .into()
}
