use charmed_lipgloss::{Style, Border, Position};

fn main() {
    let style = Style::new()
        .bold()
        .foreground("#ff00ff")
        .background("#1a1a1a")
        .padding((1, 2))
        .border(Border::rounded())
        .align(Position::Center);

    println!("{}", style.render("Hello, Lipgloss!"));
}
