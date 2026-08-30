use std::{rc::Rc, usize};
use ansi_to_tui::IntoText as _;
use ratatui::prelude::*;
use ratatui::widgets::*;

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
            Self::Image => "Image Processing",
            Self::Video => "Video Processing",
            Self::Audio => "Audio Processing",
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
        .title("| Diva FFMPEG: Home Menu |")
        .title_alignment(Alignment::Center)
        .bold();

    let home_inner = home_block.inner(outer[0]);

    frame.render_widget(home_block, outer[0]);

    let inner: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),    // inner[0] unused/empty space
            Constraint::Length(2),  // inner[1] selection guide
            Constraint::Length(1),  // inner[2] new line gap
            Constraint::Length(2),  // inner[3] unused/empty space
            Constraint::Length(20), // inner[4] unused/empty space
            Constraint::Length(1),  // inner[5] footer links
        ])
        .split(home_inner);

    let titles: Vec<Line> = HomeState::ALL
        .iter()
        .filter(|s| **s != HomeState::Default)
        .map(|s| Line::from(s.label()))
        .collect();

    // width of titles + " | " dividers between them
    let divider_width = 6; // Changed this from 3, 6 because it was truncating the last String in our array.
    let content_width: u16 = titles.iter()
        .map(|l| l.width() as u16)
        .sum::<u16>()
        + divider_width * (titles.len().saturating_sub(1)) as u16;

    let selection_guide: Paragraph<'_> = Paragraph::new(styles::center_directions(inner[1].width, inner[1].height).render(
        "Press '\x1b[38;5;218m\x1b[1mA\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mleft arrow-key\x1b[22m\x1b[39m' & '\x1b[38;5;218m\x1b[1mD\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mright arrow-key\x1b[22m\x1b[39m' to MOVE THE SELECTION HIGHLIGHT. \nPress '\x1b[38;5;205m\x1b[1mENTER\x1b[22m\x1b[39m' to CONFIRM the SELECTION or '\x1b[38;5;205m\x1b[1mQ\x1b[22m\x1b[39m' to EXIT! 💋").as_bytes().into_text().unwrap());

    let centered_area_below = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(content_width),
            Constraint::Fill(1),
        ])
        .split(inner[3])[1];

    let directions_tabs = Tabs::new(titles)
        .select(menu.selected)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .divider(" | ");

    frame.render_widget(selection_guide, inner[1]);
    frame.render_widget(directions_tabs, centered_area_below);
    frame.render_widget(component::social_footer_hyperlinks(inner[5].width, inner[5].height), inner[5]);
}