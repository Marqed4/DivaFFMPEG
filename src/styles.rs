use lipgloss::prelude::*;

pub fn default() -> Style {
    Style::new()
        .bold()
        .foreground("#ff00ff")
        .background("#1a1a1a")
        .padding((2, 4))
        .border(Border::rounded())
        .border_style(Border::thick())
        .align(Position::Center)
        .width(150)
        .height(20)
    }

pub fn link() -> Style {
     Style::new()
        .bold()
        .foreground("#00e1ff")
}