//! Git panel — branch switcher, staged/unstaged file list, commit form, and recent log.

use iced::widget::{button, column, row, scrollable, text, text_input, Space};
use iced::{Element, Length};
use newc_core::{git, project::Project};

use crate::theme as th;
use crate::state::{AppState, Message, View};

/// Renders the Git panel screen for the given project.
pub fn view<'a>(state: &'a AppState, project: &'a Project) -> Element<'a, Message> {
    let header = row![
        button(text("← Back"))
            .on_press(Message::Navigate(View::ProjectDetail(project.clone())))
            .style(th::btn_ghost),
        text(format!("◆ Git — {}", project.name))
            .size(18)
            .color(th::color::GREEN),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let is_repo = git::is_repo(&project.root);

    if !is_repo {
        return column![
            header,
            text("No git repository found.").color(th::color::YELLOW),
            button(text("Init repository")).on_press(Message::GitInit).style(th::btn_primary),
        ]
        .spacing(10)
        .padding(16)
        .into();
    }

    // Branch row
    let branches = git::branches(&project.root);
    let cur_branch = git::current_branch(&project.root);
    let mut branch_btns: Vec<Element<Message>> = vec![
        text(format!("⎇ {}", cur_branch)).color(th::color::GREEN).into(),
    ];
    for b in &branches {
        let b = b.clone();
        let _b2 = b.clone();
        if b != cur_branch {
            branch_btns.push(
                button(text(b.clone()).size(11)).on_press(Message::GitCheckout(b)).into(),
            );
        }
    }
    let branch_row = row(branch_btns).spacing(4).align_y(iced::Alignment::Center);

    let new_branch_row = row![
        text_input("New branch…", &state.git_new_branch)
            .on_input(Message::GitNewBranch)
            .width(180),
        button(text("Create").size(12)).on_press(Message::GitCreateBranch).style(th::btn_secondary),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    // Status
    let status_block: Element<Message> = if let Some(status) = git::status(&project.root) {
        column![
            row![
                text("Branch:").color(th::color::TEXT),
                text(status.branch.clone()).color(th::color::GREEN).font(iced::Font::MONOSPACE),
                text(format!("staged: {}  unstaged: {}  untracked: {}",
                    status.staged, status.unstaged, status.untracked))
                    .size(12).color(th::color::TEXT_DIM),
                Space::new().width(Length::Fill),
                button(text("Pull")).on_press(Message::GitPull).style(th::btn_secondary),
                button(text("Push")).on_press(Message::GitPush).style(th::btn_secondary),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(4)
        .padding(8)
        .into()
    } else {
        text("Unable to get git status").color(th::color::TEXT_DIM).into()
    };

    // Staged / unstaged files
    let changed = git::changed_files(&project.root);
    let (staged_files, unstaged_files): (Vec<_>, Vec<_>) = changed.iter().partition(|f| f.staged);

    let staged_section: Element<Message> = if staged_files.is_empty() {
        text("Nothing staged.").size(12).color(th::color::TEXT_DIM).into()
    } else {
        let rows: Vec<Element<Message>> = staged_files.iter().map(|f| {
            let path_str = f.path.clone();
            row![
                text(f.path.clone()).size(12).font(iced::Font::MONOSPACE).color(th::color::GREEN),
                button(text("Unstage").size(10)).on_press(Message::GitUnstage(path_str)).style(th::btn_ghost),
            ]
            .spacing(8)
            .into()
        }).collect();
        column(rows).spacing(2).into()
    };

    let unstaged_section: Element<Message> = if unstaged_files.is_empty() {
        text("No changes.").size(12).color(th::color::TEXT_DIM).into()
    } else {
        let rows: Vec<Element<Message>> = unstaged_files.iter().map(|f| {
            let path_str = f.path.clone();
            row![
                text(f.path.clone()).size(12).font(iced::Font::MONOSPACE).color(th::color::YELLOW),
                button(text("Stage").size(10)).on_press(Message::GitStage(path_str)).style(th::btn_secondary),
            ]
            .spacing(8)
            .into()
        }).collect();
        column(rows).spacing(2).into()
    };

    // Commit
    let commit_row = row![
        text_input("Commit message…", &state.git_commit_msg)
            .on_input(Message::GitCommitMsg)
            .on_submit(Message::GitCommit)
            .width(320),
        button(text("Commit")).on_press(Message::GitCommit).style(th::btn_primary),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Stash
    let stash_entries = git::stash_list(&project.root);
    let stash_btn_row = row![
        button(text("Stash").size(12)).on_press(Message::GitStash).style(th::btn_secondary),
        button(text("Pop").size(12)).on_press(Message::GitStashPop).style(th::btn_secondary),
    ]
    .spacing(6);
    let stash_rows: Vec<Element<Message>> = stash_entries.iter().map(|s| {
        row![
            text(s.reference.clone()).size(11).font(iced::Font::MONOSPACE).color(th::color::YELLOW),
            text(s.description.clone()).size(11).color(th::color::TEXT_DIM),
        ]
        .spacing(8)
        .into()
    }).collect();
    let stash_section: Element<Message> = if stash_rows.is_empty() {
        text("No stashes.").size(12).color(th::color::TEXT_DIM).into()
    } else {
        column(stash_rows).spacing(2).into()
    };

    // Log
    let log_entries = git::log(&project.root, 20);
    let log_rows: Vec<Element<Message>> = log_entries.iter().map(|entry| {
        row![
            text(entry.hash.clone()).size(11).font(iced::Font::MONOSPACE)
                .color(th::color::YELLOW).width(70),
            text(entry.author.clone()).size(11).width(120).color(th::color::TEXT_DIM),
            text(entry.message.clone()).size(11),
        ]
        .spacing(8)
        .into()
    }).collect();

    column![
        header,
        branch_row,
        new_branch_row,
        status_block,
        text("Staged").size(13).color(th::color::CYAN),
        staged_section,
        text("Unstaged").size(13).color(th::color::CYAN),
        unstaged_section,
        commit_row,
        text("Stash").size(13).color(th::color::CYAN),
        stash_btn_row,
        stash_section,
        text("Recent commits").size(13).color(th::color::CYAN),
        scrollable(column(log_rows).spacing(2)).height(200),
    ]
    .spacing(8)
    .padding(12)
    .into()
}
