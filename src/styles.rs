use lipgloss::prelude::*;

pub fn title(width: u16, height: u16) -> Style {
    return Style::new()
        .bold()
        .foreground("#ff00ff")
        .background("#241e24")
        .padding((2, 4))
        .align(Position::Center)
        .width(width)
        .height(height)
    }

pub fn link(width: u16, height: u16) -> Style {
    return Style::new()
        .bold()
        .unset_underline()
        .foreground("#f1d7f1")
        .foreground("#00e1ff")
        .align(Position::Center)
        .width(width)
        .height(height)
}

pub fn directions(width: u16, height: u16) -> Style {
    return Style::new()
    .bold()
    .background("#383838")
    .align(Position::Left)
    .width(width)
    .height(height)
}

pub fn descriptions() -> Style {
    return Style::new()
}