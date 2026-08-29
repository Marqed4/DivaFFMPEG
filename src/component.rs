use ansi_to_tui::IntoText;
use ratatui::widgets::Paragraph;

use crate::styles;

static DOC: &str = "https://marqed.it/DivaFFMPEG";
static GITHUB: &str = "https://GitHub.com/Marqed4";
static YOUTUBE: &str = "https://Youtube.com/Marqed";

fn footer_raw() -> String {
    format!(
        "\x1b[38;5;213m\x1b[1m🐙 {}\x1b[22m\x1b[39m   \x1b[38;5;219m\x1b[1m📖 {}\x1b[22m\x1b[39m   \x1b[38;5;205m\x1b[1m▶ {}\x1b[22m\x1b[39m",
        GITHUB, DOC, YOUTUBE
    )
}

pub fn social_footer_hyperlinks(width: u16, height: u16) -> Paragraph<'static> {
    return Paragraph::new(styles::link(width, height).render(&footer_raw()).as_bytes().into_text().unwrap());
}

// Ratatui redraws the buffer cell-by-cell and has no concept of OSC-8, so a real
// clickable link can't be handed to it through Paragraph/into_text and that's why it gets stripped.
// This renders the identical, already-centered line and wraps each URL in a raw OSC-8
// sequence so it's printed straight to the terminal plz see 'main.rs' instead of through ratatui.
pub fn social_footer_overlay(width: u16, height: u16) -> String {
    let rendered: String = styles::link(width, height).render(&footer_raw());

    rendered
        .replace(GITHUB, &hyperlink(GITHUB))
        .replace(DOC, &hyperlink(DOC))
        .replace(YOUTUBE, &hyperlink(YOUTUBE))
}

fn hyperlink(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{url}\x1b]8;;\x1b\\")
}