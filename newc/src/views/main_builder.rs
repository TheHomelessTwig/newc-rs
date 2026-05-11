use iced::widget::{button, column, row, scrollable, text, text_input, Space};
use iced::{Color, Element, Length};
use newc_core::{
    main_builder::MainBlock,
    project::Project,
    sync::extract_function_implementations,
};

use crate::state::{AppState, Message, View};

pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let builder = &state.main_builder;
    let author = &state.create_author;

    let date = chrono::Local::now().format("%d/%m/%Y").to_string();
    let preview = builder.preview(author, &date);

    let header = row![
        button(text("← Project"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone()))),
        text(format!("main() Composer — {}", project.name))
            .size(18)
            .color(Color::from_rgb(0.663, 0.863, 0.463)),
        Space::new().width(Length::Fill),
        button(text("↩ Undo").size(12))
            .on_press_maybe(if !state.composer_undo.is_empty() {
                Some(Message::ComposerUndo)
            } else {
                None
            }),
        button(text("↪ Redo").size(12))
            .on_press_maybe(if !state.composer_redo.is_empty() {
                Some(Message::ComposerRedo)
            } else {
                None
            }),
        button(text("Write main.c"))
            .on_press(Message::ComposerWriteMainC),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Collect available functions
    let _func_sigs: Vec<String> = project.modules.iter().flat_map(|m| {
        if m.source.exists() {
            let src = std::fs::read_to_string(&m.source).unwrap_or_default();
            extract_function_implementations(&src)
                .into_iter()
                .filter(|f| f.name != "main")
                .map(|f| f.signature)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    }).collect();

    // ── Left: block editor ────────────────────────────────────────────────────
    let block_controls = row![
        text("Add block:").size(12),
        button(text("var").size(11)).on_press(Message::None),
        button(text("call").size(11)).on_press(Message::None),
        button(text("if").size(11)).on_press(Message::None),
        button(text("while").size(11)).on_press(Message::None),
        button(text("for").size(11)).on_press(Message::None),
        button(text("comment").size(11)).on_press(Message::None),
        button(text("raw").size(11)).on_press(Message::None),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    // Block list — up/down/delete wired; drag-and-drop planned
    let block_count = builder.blocks.len();
    let block_rows: Vec<Element<Message>> = builder.blocks.iter().enumerate().map(|(i, block)| {
        let label = block_label(block);
        row![
            text(label).size(12).font(iced::Font::MONOSPACE),
            Space::new().width(Length::Fill),
            button(text("↑").size(10)).on_press_maybe(if i > 0 { Some(Message::ComposerBlockMoveUp(i)) } else { None }),
            button(text("↓").size(10)).on_press_maybe(if i + 1 < block_count { Some(Message::ComposerBlockMoveDown(i)) } else { None }),
            button(text("✗").size(10)).on_press(Message::ComposerBlockDelete(i)),
        ]
        .spacing(4)
        .into()
    }).collect();

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
        ).height(Length::Fill),
    ]
    .spacing(4)
    .width(360);

    // ── Right: code preview ────────────────────────────────────────────────────
    let right_panel = column![
        text("Preview").size(13).color(Color::from_rgb(0.471, 0.863, 0.910)),
        scrollable(
            text(preview.clone()).font(iced::Font::MONOSPACE).size(12)
        ).height(Length::Fill),
    ]
    .spacing(4);

    column![
        header,
        row![left_panel, right_panel].spacing(12).height(Length::Fill),
    ]
    .spacing(8)
    .padding(12)
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
