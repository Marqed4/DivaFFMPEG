use ansi_to_tui::IntoText;
use ratatui::widgets::Paragraph;

use crate::styles;

#[allow(unused)]
static DOC: &str = "https://marqed.it/DivaFFMPEG";
static GITHUB: &str = "https://GitHub.com/Marqed4";
#[allow(unused)]
static YOUTUBE: &str = "https://Youtube.com/Marqed";

pub fn social_footer_hyperlinks(width: u16, height: u16) -> Paragraph<'static> {
    let raw = format!("{}", GITHUB);

    return Paragraph::new(styles::link(width, height).render(&raw).as_bytes().into_text().unwrap());
}