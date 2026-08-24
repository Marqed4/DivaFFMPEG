use std::{fs::File, io::{self, Stdout}, rc::Rc};
use ansi_to_tui::IntoText as _;
use std::time::Duration;
use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode},
};
use ratatui::prelude::*;
use ratatui::widgets::*;

use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;

mod reusable_widgets;
mod styles;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout: Stdout = io::stdout();
    /*
    The way 'execute!(stdout, EnterAlternateScreen)?;' works is by pushing invisible ascii
    characters on the screen clearing chatter that was released from previous
    use including compiling the .exe or any other related executions and. E.g., \x1b[?1049h
    */
    execute!(stdout, EnterAlternateScreen)?;

    /*
    This uses the standard write and file opening crates, as well as the chrono package to
    accurately create a .log file that can be written to during a session of this terminal application.
    */
    let filename: String = format!("log/DivaFFMPEG-{}.log", Utc::now().format("%Y-%m-%d_%H-%M-%S"));
    let mut _file: File = OpenOptions::new()
    .append(true)
    .create(true)
    .open(&filename)?;

    // Prevent log spamming.
    let mut last_logged_status = String::new();

    let backend: CrosstermBackend<Stdout> = CrosstermBackend::new(stdout);
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = Terminal::new(backend)?;
    
    let mut safe_trigger_exit_terminal: bool = false;

    loop 
    {
        terminal.draw( |f| {
            // Creates our termainal frame.
            let size: Rect = f.area();
            
            /*
            The goal of this screen is to serve as a splash screen for a user who is trying to exit the TUI.
            Directions on how to restore the state previous to pressing q outside of a text input situation
            are displayed. External links to app support and developer socials are also provided.
            */
            if safe_trigger_exit_terminal {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                        Constraint::Length(1),              // footer, directly underneath
                        Constraint::Fill(1),                // footer is exactly 1 line
                        ])
                        .split(size);

                    let exit_page: Paragraph<'_> = Paragraph::new(styles::default().render(
                        "Press e to return.
                        \nPress q again to exit! 💋").as_bytes().into_text().unwrap());

                    f.render_widget(exit_page, outer[0]);
                    f.render_widget(reusable_widgets::social_footer_hyperlinks(), outer[1]);
                
                    if last_logged_status != "Exit Screen".to_string() {
                    let _ = writeln!(_file, "{}", format!("Entered the exit screen at {}.", Utc::now().format("%H-%M-%S")));
                    last_logged_status = "Exit Screen".to_string();
                }
            } else {
                /*
                The goal of this screen is to serve as a splash screen for those who open the app.
                A welcoming text, input directions, and external links to app support and
                developer socials should be shown.
                */
                let outer: Rc<[Rect]> = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                    Constraint::Min(3),      // intro text needs at least 3 lines
                    Constraint::Length(1),   // footer is exactly 1 line
                    ])
                    .split(size);

                let introduction: Paragraph<'_> = Paragraph::new(styles::default().render(
                    "Welcome to Diva FFMPEG!
                \nStylish but functional ffmpeg wrapper 💅
                \nBy Marqed4").as_bytes().into_text().unwrap());

                f.render_widget(introduction, outer[0]);
                f.render_widget(reusable_widgets::social_footer_hyperlinks(), outer[1]);

                if last_logged_status != "Start Screen".to_string() {
                    let _ = writeln!(_file, "{}", format!("Entered the start screen at {}.", Utc::now().format("%H-%M-%S")));
                    last_logged_status = "Start Screen".to_string();
                }

            }
        })?;

        // The purpose of this block is to catch key strokes that trigger events and change code state.
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') if !safe_trigger_exit_terminal => 
                    {
                        safe_trigger_exit_terminal = true;
                        writeln!(_file, "{}", format!("State: 'safe_trigger_exit' became {} at {} because {:?} was pressed so the prompt to close the application was initiated.",
                        safe_trigger_exit_terminal, Utc::now().format("%H-%M-%S"), k.code))?;
                        let _ = event::read()?;
                    },
                    KeyCode::Char('q') if safe_trigger_exit_terminal =>
                    {
                        writeln!(_file, "{}", format!("State: 'trigger_exit' became {} at {} because {:?} was pressed so the application was closed.",
                        safe_trigger_exit_terminal, Utc::now().format("%H-%M-%S"), k.code))?;
                        break;
                    },
                    KeyCode::Char('e') if safe_trigger_exit_terminal =>
                    {
                        safe_trigger_exit_terminal = false;
                        writeln!(_file, "{}", format!("State: 'trigger_exit' became {} at {} because {:?} was pressed.",
                        safe_trigger_exit_terminal, Utc::now().format("%H-%M-%S"), k.code))?;
                    },
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}