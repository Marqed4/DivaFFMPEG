use std::io;
use ansi_to_tui::IntoText as _;
use std::time::Duration;
use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode},
};
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::text::Span;
use ratatui::prelude::Stylize;
mod styles;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // start splash page

    // mid business logic
    let mut trigger_exit_terminal = false;

    loop 
    {
        terminal.draw(|f| {
            let size = f.area();

            let credit_area = Rect {
                x: size.x,
                y: size.y + size.height - 1, // last line
                width: size.width,
                height: 1,
            };

            if trigger_exit_terminal {
                let styled_content = styles::exit_screen().render("Press q again to exit! 💋");
                let ansi_content = styled_content.as_bytes().into_text().unwrap();
                f.render_widget(Paragraph::new(ansi_content), size);
            } else {
                // Introduction
                let raw_string = "Welcome to Diva FFMPEG!\nStylish but functional ffmpeg wrapper 💅\nBy Marqed4";
                let styled_content = styles::intro().render(&raw_string);
                let ansi_content = styled_content.as_bytes().into_text().unwrap();
                f.render_widget(Paragraph::new(ansi_content), size);

                // Introduction Options
                let raw_string = "Press Q to escape\nPress Enter/↵";

                // Credits: TRYING TO ACHIEVE HYPERTEXT...
                // let link = Link::new(
                //     Span::from("By Marqed4").blue().underlined(),
                //     "https://github.com/Marqed4",
                // );
                // f.render_widget(link, credit_area);
            }
        })?;

        // Exit Terminal Logic
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') if !trigger_exit_terminal => trigger_exit_terminal = true,
                    KeyCode::Char('q') if trigger_exit_terminal => break, // second press exits
                    KeyCode::Esc => break, // or just exit immediately
                    _ => {}
                }
            }
        }
    }

    // end
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}