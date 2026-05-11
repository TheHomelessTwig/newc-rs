// Flowchart canvas — renders MainBuilderState blocks as an interactive flowchart.
// Start → blocks with decision diamonds for if/while/for → End.

use iced::widget::canvas::{self, Canvas, Path, Stroke};
use iced::widget::{button, column, row, text, Space};
use iced::{Color, Element, Length, Point, Rectangle, Size, mouse};
use newc_core::main_builder::{MainBlock, MainBuilderState};
use newc_core::project::Project;

use crate::state::{AppState, Message, View};
use crate::views::call_graph::CanvasState;

const BOX_W: f32 = 180.0;
const BOX_H: f32 = 40.0;
const DIAMOND_W: f32 = 160.0;
const _DIAMOND_H: f32 = 48.0;
const V_GAP: f32 = 30.0;

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
    Rect,       // process / statement
    Diamond,    // condition
    Rounded,    // start / end
}

struct FlowCanvas {
    nodes: Vec<FlowNode>,
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
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgb8(0x1E, 0x1C, 0x1F));
            let cx = bounds.width / 2.0 + self.pan_x;
            let z = self.zoom;

            // Draw connectors between nodes
            for i in 0..self.nodes.len().saturating_sub(1) {
                let n = &self.nodes[i];
                let m = &self.nodes[i + 1];
                let fx = cx + n.x * z + n.w * z / 2.0;
                let fy = self.pan_y + n.y * z + n.h * z;
                let tx = cx + m.x * z + m.w * z / 2.0;
                let ty = self.pan_y + m.y * z;
                let path = Path::line(Point::new(fx, fy), Point::new(tx, ty));
                frame.stroke(&path, Stroke::default()
                    .with_color(Color::from_rgb8(0x7A, 0x7A, 0x8A))
                    .with_width(1.5 * z));
                // Arrowhead
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
                        Color::from_rgb8(0xA9, 0xDC, 0x76),
                        Color::from_rgb8(0xA9, 0xDC, 0x76),
                    ),
                    NodeShape::Diamond => (
                        Color::from_rgb8(0x3A, 0x4A, 0x6A),
                        Color::from_rgb8(0xFF, 0xD8, 0x66),
                        Color::from_rgb8(0xFF, 0xD8, 0x66),
                    ),
                    NodeShape::Rect => (
                        Color::from_rgb8(0x3D, 0x3A, 0x3F),
                        Color::from_rgb8(0xFC, 0xFC, 0xFA),
                        Color::WHITE,
                    ),
                };

                match node.shape {
                    NodeShape::Diamond => {
                        let cx = nx + nw / 2.0;
                        let cy = ny + nh / 2.0;
                        let diamond = Path::new(|b| {
                            b.move_to(Point::new(cx, ny));
                            b.line_to(Point::new(nx + nw, cy));
                            b.line_to(Point::new(cx, ny + nh));
                            b.line_to(Point::new(nx, cy));
                            b.close();
                        });
                        frame.fill(&diamond, fill);
                        frame.stroke(&diamond, Stroke::default().with_color(border).with_width(1.5));
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

                // Truncate label to fit
                let max_chars = (nw / (fs * 0.6)) as usize;
                let label = if node.label.len() > max_chars && max_chars > 3 {
                    format!("{}…", &node.label[..max_chars - 1])
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

fn build_flowchart(builder: &MainBuilderState) -> Vec<FlowNode> {
    let mut nodes = Vec::new();
    let mut y = 0.0;
    nodes.push(FlowNode { x: -BOX_W / 2.0, y, w: BOX_W, h: BOX_H, label: "START".into(), shape: NodeShape::Rounded });
    y += BOX_H + V_GAP;
    push_blocks(&builder.blocks, &mut nodes, &mut y, 0.0);
    nodes.push(FlowNode { x: -BOX_W / 2.0, y, w: BOX_W, h: BOX_H, label: "return 0".into(), shape: NodeShape::Rounded });
    nodes
}

fn push_blocks(blocks: &[MainBlock], nodes: &mut Vec<FlowNode>, y: &mut f32, indent: f32) {
    let x_center = -BOX_W / 2.0 + indent;
    let d_center = -DIAMOND_W / 2.0 + indent;
    for block in blocks {
        match block {
            MainBlock::Comment(_) | MainBlock::BlankLine => continue, // skip decorative
            MainBlock::IfBlock { condition, body, else_body } => {
                nodes.push(FlowNode { x: d_center, y: *y, w: DIAMOND_W, h: BOX_H,
                    label: format!("if {condition}"), shape: NodeShape::Diamond });
                *y += BOX_H + V_GAP;
                // true branch (indented right)
                if !body.is_empty() {
                    nodes.push(FlowNode { x: d_center + 20.0, y: *y, w: BOX_W, h: 20.0,
                        label: "YES →".into(), shape: NodeShape::Rect });
                    *y += 20.0 + V_GAP / 2.0;
                    push_blocks(body, nodes, y, indent + 20.0);
                }
                // false/else branch
                if !else_body.is_empty() {
                    nodes.push(FlowNode { x: d_center - 20.0, y: *y, w: BOX_W, h: 20.0,
                        label: "NO →".into(), shape: NodeShape::Rect });
                    *y += 20.0 + V_GAP / 2.0;
                    push_blocks(else_body, nodes, y, indent - 20.0);
                }
            }
            MainBlock::WhileLoop { condition, body } => {
                nodes.push(FlowNode { x: d_center, y: *y, w: DIAMOND_W, h: BOX_H,
                    label: format!("while {condition}"), shape: NodeShape::Diamond });
                *y += BOX_H + V_GAP;
                if !body.is_empty() {
                    push_blocks(body, nodes, y, indent + 16.0);
                    // loop-back marker
                    nodes.push(FlowNode { x: d_center + 30.0, y: *y, w: 100.0, h: 18.0,
                        label: "↩ loop".into(), shape: NodeShape::Rect });
                    *y += 18.0 + V_GAP;
                }
            }
            MainBlock::ForLoop { init, condition, increment, body } => {
                nodes.push(FlowNode { x: d_center, y: *y, w: DIAMOND_W, h: BOX_H,
                    label: format!("for {init}; {condition}; {increment}"), shape: NodeShape::Diamond });
                *y += BOX_H + V_GAP;
                if !body.is_empty() {
                    push_blocks(body, nodes, y, indent + 16.0);
                    nodes.push(FlowNode { x: d_center + 30.0, y: *y, w: 100.0, h: 18.0,
                        label: "↩ loop".into(), shape: NodeShape::Rect });
                    *y += 18.0 + V_GAP;
                }
            }
            block => {
                let (label, shape, w) = block_to_flow(block);
                nodes.push(FlowNode { x: x_center, y: *y, w, h: BOX_H, label, shape });
                *y += BOX_H + V_GAP;
            }
        }
    }
}

fn block_to_flow(block: &MainBlock) -> (String, NodeShape, f32) {
    match block {
        MainBlock::VarDecl { type_name, name, init, .. } => {
            let label = if init.is_empty() { format!("{type_name} {name}") } else { format!("{type_name} {name} = {init}") };
            (label, NodeShape::Rect, BOX_W)
        }
        MainBlock::FunctionCall { func_name, args, assign_to, .. } => {
            let call = format!("{}({})", func_name, args.join(", "));
            let label = if assign_to.is_empty() { call } else { format!("{assign_to} = {call}") };
            (label, NodeShape::Rect, BOX_W)
        }
        MainBlock::RawCode(r) => {
            let preview = if r.len() > 30 { &r[..30] } else { r };
            (preview.to_string(), NodeShape::Rect, BOX_W)
        }
        // Loops/ifs/blanks/comments handled in push_blocks
        _ => ("…".to_string(), NodeShape::Rect, BOX_W),
    }
}

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let nodes = build_flowchart(&state.main_builder);
    let block_count = state.main_builder.blocks.len();

    let controls = row![
        button(text("← Composer"))
            .on_press(Message::Navigate(View::MainBuilder(project.clone()))),
        text(format!("Flowchart — {}", project.name))
            .size(16).color(Color::from_rgb(1.0, 0.847, 0.4)),
        Space::new().width(Length::Fill),
        button(text("Reset").size(12)).on_press(Message::GraphReset),
        text(format!("{block_count} blocks")).size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
        text("Drag: pan  Scroll: zoom").size(11).color(Color::from_rgb(0.5, 0.5, 0.5)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let legend = row![
        text("▬ Process").size(11).color(Color::WHITE),
        text("◇ Decision").size(11).color(Color::from_rgb8(0xFF, 0xD8, 0x66)),
        text("▬ Start/End").size(11).color(Color::from_rgb8(0xA9, 0xDC, 0x76)),
    ]
    .spacing(16);

    let canvas_widget = Canvas::new(FlowCanvas {
        nodes,
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
