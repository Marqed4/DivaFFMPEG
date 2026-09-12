use std::{rc::Rc, usize};
use ansi_to_tui::IntoText as _;
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::background;
use crate::component;
use crate::styles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeState {
    Default,
    Image,
    Video,
    Audio,
}

//                      <-- SELECTABLES -->
impl HomeState {
    const ALL: [HomeState; 4] = [
        HomeState::Default,
        HomeState::Image,
        HomeState::Video,
        HomeState::Audio,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Image => "🖼️ Image Processing",
            Self::Video => "🎬 Video Processing",
            Self::Audio => "🎵 Audio Processing",
        }
    }
}

pub struct HomeMenuState {
    selected: usize,
}

impl HomeMenuState {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn next(&mut self) {
        let len = HomeState::ALL.len() - 1;
        self.selected = (self.selected + 1) % len;
    }

    pub fn previous(&mut self) {
        let len = HomeState::ALL.len() - 1;
        self.selected = if self.selected == 0 { len - 1 } else { self.selected - 1 };
    }

    pub fn selected_state(&self) -> HomeState {
        HomeState::ALL[self.selected + 1]
    }
}

pub fn render(frame: &mut Frame<'_>, _state: HomeState, menu: &HomeMenuState) {
    let size = frame.area();

    let outer: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
        ])
        .split(size);

    let home_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("✦  Diva FFMPEG: Home Menu  ✦")
        .title_alignment(Alignment::Center)
        .bold();

    let home_inner = home_block.inner(outer[0]);

    frame.render_widget(home_block, outer[0]);

    // Split off the footer row first, then give Felix his own right-hand
    // column for the rest - his art no longer shares rows with the centered
    // guide/tabs text, so there's no width to reserve or "climb" threshold
    // to fight over: he's always full height and they never collide.
    let rows: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // rows[0] content area (text column + mascot column)
            Constraint::Length(1), // rows[1] footer links
        ])
        .split(home_inner);

    let columns: Rc<[Rect]> = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),                              // columns[0] text column
            Constraint::Length(background::felix_art_width()), // columns[1] mascot column
        ])
        .split(rows[0]);

    let inner: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // inner[0] unused/empty space
            Constraint::Length(3), // inner[1] selection guide
            Constraint::Length(1), // inner[2] new line gap
            Constraint::Length(1), // inner[3] direction tabs
            Constraint::Fill(1),   // inner[4] unused/empty space
        ])
        .split(columns[0]);

    let titles: Vec<Line> = HomeState::ALL
        .iter()
        .filter(|s| **s != HomeState::Default)
        .map(|s| Line::from(s.label()))
        .collect();

    // width of titles + " | " dividers between them, plus a safety margin since
    // emoji glyph width is often undercounted, which was truncating the last label.
    let divider_width = 3; // " | "
    let safety_margin = 2;
    let content_width: u16 = titles.iter()
        .map(|l| l.width() as u16 + safety_margin)
        .sum::<u16>()
        + divider_width * (titles.len().saturating_sub(1)) as u16;

    let selection_guide: Paragraph<'_> = Paragraph::new(styles::center_directions_transparent(inner[1].width, inner[1].height).render(
        "Press '\x1b[38;5;218m\x1b[1mA\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mleft arrow-key\x1b[22m\x1b[39m' & '\x1b[38;5;218m\x1b[1mD\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mright arrow-key\x1b[22m\x1b[39m' \n \
        to MOVE THE SELECTION HIGHLIGHT. \n \
        Press '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m' to CONFIRM the SELECTION, '\x1b[38;5;205m\x1b[1mESC\x1b[22m\x1b[39m' to return to the PREVIOUS SCREEN, or '\x1b[38;5;205m\x1b[1mQ\x1b[22m\x1b[39m' to EXIT! 💋").as_bytes().into_text().unwrap());

    let centered_area_below = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Min(content_width),
            Constraint::Fill(1),
        ])
        .split(inner[3])[1];

    let directions_tabs = Tabs::new(titles)
        .select(menu.selected)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .divider(" | ");

    frame.render_widget(selection_guide, inner[1]);
    frame.render_widget(directions_tabs, centered_area_below);
    background::render_felix_bottom_right(frame, columns[1]);
    frame.render_widget(component::social_footer_hyperlinks(rows[1].width, rows[1].height), rows[1]);
}