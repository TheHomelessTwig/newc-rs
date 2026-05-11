use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Color, Element, Length, Background, Border};
use newc_core::{
    main_builder::MainBlock,
    project::Project,
};

use crate::state::{AppState, Message, View};
use crate::theme as th;

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let builder = &state.main_builder;
    let author = &state.create_author;

    let date = chrono::Local::now().format("%d/%m/%Y").to_string();
    let preview = builder.preview(author, &date);

    let header = row![
        button(text("← Project").size(12))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        text(format!("main() Composer — {}", project.name))
            .size(18)
            .color(th::color::GREEN),
        Space::new().width(Length::Fill),
        button(text("↩ Undo").size(12))
            .on_press_maybe(if !state.composer_undo.is_empty() {
                Some(Message::ComposerUndo)
            } else {
                None
            })
            .style(th::btn_secondary),
        button(text("↪ Redo").size(12))
            .on_press_maybe(if !state.composer_redo.is_empty() {
                Some(Message::ComposerRedo)
            } else {
                None
            })
            .style(th::btn_secondary),
        button(text("Write main.c").size(12))
            .on_press(Message::ComposerWriteMainC)
            .style(th::btn_primary),
        button(text("Flowchart").size(12))
            .on_press(Message::Navigate(View::FlowChart(project.clone())))
            .style(th::btn_secondary),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // ── Left: block editor ────────────────────────────────────────────────────
    let block_controls = row![
        text("Add block:").size(12),
        button(text("var").size(11)).on_press(Message::ComposerAddBlock(MainBlock::VarDecl {
            type_name: "int".into(), name: "x".into(), init: String::new(),
            is_array: false, array_size: String::new(),
        })),
        button(text("call").size(11)).on_press(Message::ComposerAddBlock(MainBlock::FunctionCall {
            func_name: "func".into(), args: Vec::new(),
            assign_to: String::new(), comment: String::new(),
        })),
        button(text("if").size(11)).on_press(Message::ComposerAddBlock(MainBlock::IfBlock {
            condition: "condition".into(), body: Vec::new(), else_body: Vec::new(),
        })),
        button(text("while").size(11)).on_press(Message::ComposerAddBlock(MainBlock::WhileLoop {
            condition: "condition".into(), body: Vec::new(),
        })),
        button(text("for").size(11)).on_press(Message::ComposerAddBlock(MainBlock::ForLoop {
            init: "int i = 0".into(), condition: "i < n".into(),
            increment: "i++".into(), body: Vec::new(),
        })),
        button(text("comment").size(11)).on_press(Message::ComposerAddBlock(MainBlock::Comment(String::new()))),
        button(text("raw").size(11)).on_press(Message::ComposerAddBlock(MainBlock::RawCode(String::new()))),
        button(text("blank").size(11)).on_press(Message::ComposerAddBlock(MainBlock::BlankLine)),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    // Block list — click to select, up/down/delete wired
    let block_count = builder.blocks.len();
    let block_rows: Vec<Element<Message>> = builder.blocks.iter().enumerate().map(|(i, block)| {
        let label = block_label(block);
        let is_sel = state.composer_selected == Some(i);
        let label_color = if is_sel {
            Color::from_rgb(0.663, 0.863, 0.463)
        } else {
            Color::WHITE
        };
        let is_this_dragging = state.composer_drag == Some(i);
        let any_drag = state.composer_drag.is_some();
        let drag_color = if is_this_dragging { Color::from_rgb(1.0, 0.847, 0.4) } else if any_drag { Color::from_rgb(0.5, 0.75, 0.5) } else { Color::from_rgb(0.35, 0.35, 0.35) };
        let block_bg = if is_this_dragging { Color::from_rgba(1.0, 0.847, 0.4, 0.15) } else { block_type_color(block) };
        // While dragging: ⣿ click = drop here; otherwise start drag
        let drag_msg = if any_drag && !is_this_dragging {
            Message::ComposerDragDrop(i)
        } else if is_this_dragging {
            Message::ComposerDragEnd
        } else {
            Message::ComposerDragStart(i)
        };
        container(row![
            button(text("⣿").size(10).color(drag_color))
                .on_press(drag_msg),
            button(text(label).size(12).font(iced::Font::MONOSPACE).color(label_color))
                .on_press(if any_drag && !is_this_dragging { Message::ComposerDragDrop(i) } else { Message::ComposerSelectBlock(i) }),
            Space::new().width(Length::Fill),
            button(text("↑").size(10)).on_press_maybe(if i > 0 { Some(Message::ComposerBlockMoveUp(i)) } else { None }),
            button(text("↓").size(10)).on_press_maybe(if i + 1 < block_count { Some(Message::ComposerBlockMoveDown(i)) } else { None }),
            button(text("✗").size(10)).on_press(Message::ComposerBlockDelete(i)),
        ]
        .spacing(4))
        .style(move |_| iced::widget::container::Style {
            background: Some(Background::Color(block_bg)),
            border: Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        })
        .padding([2, 4])
        .into()
    }).collect();

    // Block editor panel (shown when a block is selected)
    let editor_panel: Element<Message> = if let Some(idx) = state.composer_selected {
        if let Some(block) = builder.blocks.get(idx) {
            build_block_editor(block, idx)
        } else {
            text("").into()
        }
    } else {
        text("Click a block to edit its fields.").size(11)
            .color(Color::from_rgb(0.5, 0.5, 0.5))
            .into()
    };

    let block_list_height = if state.composer_selected.is_some() {
        Length::Fixed(160.0)
    } else {
        Length::Fill
    };

    let left_panel = column![
        text("Author:").size(12),
        text_input("Author name", author)
            .on_input(Message::CreateAuthor)
            .width(200),
        Space::new().height(8),
        block_controls,
        Space::new().height(4),
        scrollable(
            if block_rows.is_empty() {
                column![text("No blocks yet. Add a block above.").size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5))]
            } else {
                column(block_rows).spacing(4)
            }
        ).height(block_list_height),
        editor_panel,
    ]
    .spacing(4)
    .width(360);

    // ── Right: code preview ────────────────────────────────────────────────────
    let right_panel = container(
        column![
            th::section_title("Preview"),
            scrollable(
                text(preview.clone()).font(iced::Font::MONOSPACE).size(12).color(th::color::TEXT)
            ).height(Length::Fill),
        ]
        .spacing(4)
        .padding(8),
    )
    .style(th::card_style)
    .height(Length::Fill);

    column![
        header,
        row![left_panel, right_panel].spacing(12).height(Length::Fill),
    ]
    .spacing(8)
    .padding(12)
    .into()
}

fn block_type_color(block: &MainBlock) -> Color {
    match block {
        MainBlock::VarDecl { .. }      => Color::from_rgba(0.25, 0.47, 0.65, 0.35),
        MainBlock::FunctionCall { .. } => Color::from_rgba(0.25, 0.55, 0.30, 0.35),
        MainBlock::IfBlock { .. }      => Color::from_rgba(0.65, 0.40, 0.20, 0.35),
        MainBlock::WhileLoop { .. }
        | MainBlock::ForLoop { .. }    => Color::from_rgba(0.55, 0.30, 0.55, 0.35),
        MainBlock::Comment { .. }      => Color::from_rgba(0.30, 0.30, 0.30, 0.30),
        MainBlock::RawCode { .. }      => Color::from_rgba(0.45, 0.40, 0.25, 0.35),
        MainBlock::BlankLine           => Color::from_rgba(0.0, 0.0, 0.0, 0.0),
    }
}

fn build_block_editor<'a>(block: &MainBlock, idx: usize) -> Element<'a, Message> {
    let field_row = |label: &'static str, val: &str, field: &'static str| -> Element<'a, Message> {
        let val = val.to_string();
        row![
            text(label).size(11).width(80),
            text_input("", &val)
                .on_input(move |v| Message::ComposerEditField {
                    idx,
                    field: field.to_string(),
                    value: v,
                })
                .size(12)
                .width(Length::Fill),
        ]
        .spacing(4)
        .into()
    };

    let title = text(format!("Edit block {} — {}", idx + 1, block.label()))
        .size(12)
        .color(Color::from_rgb(0.471, 0.863, 0.910));

    let fields: Vec<Element<Message>> = match block {
        MainBlock::VarDecl { type_name, name, init, .. } => vec![
            field_row("Type:", type_name, "type"),
            field_row("Name:", name, "name"),
            field_row("Init:", init, "init"),
        ],
        MainBlock::FunctionCall { func_name, args, assign_to, .. } => vec![
            field_row("Function:", func_name, "func_name"),
            field_row("Args:", &args.join(", "), "args"),
            field_row("Assign to:", assign_to, "assign_to"),
        ],
        MainBlock::IfBlock { condition, .. } | MainBlock::WhileLoop { condition, .. } => vec![
            field_row("Condition:", condition, "condition"),
        ],
        MainBlock::ForLoop { init, condition, increment, .. } => vec![
            field_row("Init:", init, "init"),
            field_row("Condition:", condition, "condition"),
            field_row("Increment:", increment, "increment"),
        ],
        MainBlock::Comment(c) => vec![field_row("Text:", c, "text")],
        MainBlock::RawCode(c) => vec![field_row("Code:", c, "text")],
        MainBlock::BlankLine => vec![text("Blank line — no fields.").size(11).into()],
    };

    column(std::iter::once(title.into()).chain(fields).collect::<Vec<_>>())
        .spacing(4)
        .into()
}

fn block_label(block: &MainBlock) -> String {
    match block {
        MainBlock::VarDecl { type_name, name, init, .. } => {
            if init.trim().is_empty() {
                format!("var {} {}", type_name, name)
            } else {
                format!("var {} {} = {}", type_name, name, init)
            }
        }
        MainBlock::FunctionCall { func_name, args, assign_to, .. } => {
            if !assign_to.is_empty() {
                format!("{} = {}({})", assign_to, func_name, args.join(", "))
            } else {
                format!("{}({})", func_name, args.join(", "))
            }
        }
        MainBlock::IfBlock { condition, .. } => format!("if ({})", condition),
        MainBlock::WhileLoop { condition, .. } => format!("while ({})", condition),
        MainBlock::ForLoop { init, condition, increment, .. } => {
            format!("for ({init}; {condition}; {increment})")
        }
        MainBlock::Comment(c) => format!("// {c}"),
        MainBlock::RawCode(r) => {
            let preview = if r.len() > 40 { &r[..40] } else { r };
            format!("raw: {preview}")
        }
        MainBlock::BlankLine => "blank".into(),
    }
}
