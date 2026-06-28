use lipgloss::prelude::*;

pub fn intro() -> Style {
    Style::new()
        .bold()
        .foreground("#ff00ff")
        .background("#1a1a1a")
        .padding((2, 4))
        .border(Border::rounded())
        .align(Position::Center)
        .width(60)
    }

    // println!("{}", intro.render(
    // "Welcome to Diva FFMPEG!" + 
    // "\n" + 
    // "By https://github.com/Marqed4"));

pub fn exit_screen() -> Style {
    Style::new()
        .foreground("#ff0062")
        .background("#1a1a1a")
        .padding((2, 4))
        .width(60)
}

pub fn header() -> Style {
     Style::new()
        .bold()
        .foreground("#00e1ff")
}