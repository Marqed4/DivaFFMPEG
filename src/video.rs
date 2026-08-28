use std::{fs::File, io::{self, Stdout}, rc::Rc};
use ansi_to_tui::IntoText as _;
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::reusable_widgets;
use crate::styles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoProcessingState {
    Default,
    ExplainCreate,
    ExplainCompress,
    ExplainTrim,
    ExplainMerge,
}

impl VideoProcessingState {
    const ALL: [VideoProcessingState; 5] = [
        VideoProcessingState::Default,
        VideoProcessingState::ExplainCreate,
        VideoProcessingState::ExplainCompress,
        VideoProcessingState::ExplainTrim,
        VideoProcessingState::ExplainMerge,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::ExplainCreate => "explain_create",
            Self::ExplainCompress => "explain_compress",
            Self::ExplainTrim => "explain_trim",
            Self::ExplainMerge => "explain_merge",
        }
    }
}

pub struct DirectionsMenuState {
    selected: usize,
}

impl DirectionsMenuState {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn next(&mut self) {
        let len = VideoProcessingState::ALL.len() - 1;
        self.selected = (self.selected + 1) % len;
    }

    pub fn previous(&mut self) {
        let len = VideoProcessingState::ALL.len() - 1;
        self.selected = if self.selected == 0 { len - 1 } else { self.selected - 1 };
    }

    pub fn selected_state(&self) -> VideoProcessingState {
        VideoProcessingState::ALL[self.selected + 1]
    }
}

pub fn render(f: &mut Frame<'_>, state: VideoProcessingState, menu: &DirectionsMenuState) {
    let size = f.area();

    let outer: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
        ])
        .split(size);

    let video_block = Block::default()
        .borders(Borders::ALL)
        .title("| Diva FFMPEG: Video Processing |")
        .title_alignment(Alignment::Center)
        .bold();

    let video_inner = video_block.inner(outer[0]);

    f.render_widget(video_block, outer[0]);

    let inner: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(video_inner);

    let body: Rc<[Rect]> = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(inner[0]);

    let titles: Vec<Line> = VideoProcessingState::ALL
        .iter()
        .filter(|s| **s != VideoProcessingState::Default)
        .map(|s| Line::from(s.label()))
        .collect();

    let directions_tabs = Tabs::new(titles)
        .select(menu.selected)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .divider(" | ");

    //                      <-- DIRECTIONS -->
    match state {
        VideoProcessingState::Default => {},
        VideoProcessingState::ExplainCreate => {
            let explain_create_text = Paragraph::new(
                styles::center_directions(body[0].width, body[0].height).render(
                    "To create a new .mp4, provide a source file and output path. \
                    Diva FFMPEG will wrap ffmpeg's -c:v libx264 encode for you."
                ).as_bytes().into_text().unwrap()
            );
            f.render_widget(explain_create_text, inner[0]);
        },
        VideoProcessingState::ExplainCompress => {
            let explain_compress_text = Paragraph::new(
                styles::center_directions(body[0].width, body[0].height).render(
                    "Compression re-encodes your video at a lower bitrate. \
                    Choose a target size or a CRF value to control quality vs. file size."
                ).as_bytes().into_text().unwrap()
            );
            f.render_widget(explain_compress_text, inner[0]);
        },
        VideoProcessingState::ExplainTrim => {
            let explain_trim_text = Paragraph::new(
                styles::center_directions(body[0].width, body[0].height).render(
                    "Trimming cuts your video down to a start and end timestamp \
                    without re-encoding, so it's fast and lossless."
                ).as_bytes().into_text().unwrap()
            );
            f.render_widget(explain_trim_text, inner[0]);
        },
        VideoProcessingState::ExplainMerge => {
            let explain_merge_text = Paragraph::new(
                styles::center_directions(body[0].width, body[0].height).render(
                    "Merging concatenates multiple video files into one, provided \
                    they share compatible codecs and resolution."
                ).as_bytes().into_text().unwrap()
            );
            f.render_widget(explain_merge_text, inner[0]);
        },
    }

    f.render_widget(directions_tabs, inner[1]);
    f.render_widget(reusable_widgets::social_footer_hyperlinks(inner[2].width, inner[2].height), inner[2]);
}