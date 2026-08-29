use lipgloss::prelude::*;

// https://en.wikipedia.org/wiki/ANSI_escape_code brief reading on how ansi escape codes work, and some origin.
// Thank you to project owner for https://ansi.gabebanks.net/ allowing me to easily generate stylish ansi escape code for this project.

pub fn title(width: u16, height: u16) -> Style {
    return Style::new()
        .bold()
        .foreground("#ff00ff")
        .padding((2, 4))
        .align(Position::Center)
        .width(width)
        .height(height)
    }

pub fn link(width: u16, height: u16) -> Style {
    return Style::new()
        .bold()
        .underline()
        .foreground("#ff85c8")
        .align(Position::Center)
        .width(width)
        .height(height)
}

#[allow(unused)]
pub fn left_directions(width: u16, height: u16) -> Style {
    return Style::new()
    .bold()
    .background("#383838")
    .align(Position::Left)
    .width(width)
    .height(height)
}

pub fn left_directions_transparent(width: u16, height: u16) -> Style {
    return Style::new()
    .bold()
    .align(Position::Left)
    .width(width)
    .height(height)
}

pub fn center_directions(width: u16, height: u16) -> Style {
    return Style::new()
    .bold()
    .background("#383838")
    .align(Position::Center)
    .width(width)
    .height(height)
}
#[allow(unused)]
pub fn descriptions() -> Style {
    return Style::new()
}

#[allow(unused)]
pub fn centered_menu(width: u16, height: u16) -> Style {
    return Style::new()
    .bold()
    .align(Position::Center)
    .width(width)
    .height(height)
}

pub fn dir_highlight() -> Style {
    return Style::new()
    .bold()
    .foreground("#E9D502")
}