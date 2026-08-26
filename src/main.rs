use std::{fs::File, io::{self, Stdout}, rc::Rc};
use ansi_to_tui::IntoText as _;
use lipgloss::Position::Center;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode::{self, Enter}}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{widgets::LegendPosition::Top};
use ratatui::prelude::*;
use ratatui::widgets::*;

use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;

mod home;
mod video;
mod reusable_widgets;
mod styles;

use video::{VideoProcessingState, DirectionsMenuState};

fn main() -> Result<(), io::Error> {

    //                      <-- ENABLE TERMINAL -->

    enable_raw_mode()?;
    let mut stdout: Stdout = io::stdout();

    // The way 'execute!(stdout, EnterAlternateScreen)?;' works is by pushing invisible ascii
    // characters on the screen clearing chatter that was released from previous
    // use including compiling the .exe or any other related executions and. E.g., \x1b[?1049h
    execute!(stdout, EnterAlternateScreen)?;

    //                      <-- LOGGING APPLICATION STATE -->

    // This uses the standard write and file opening crates, as well as the chrono package to
    // accurately create a .log file that can be written to during a session of this terminal application.
    let filename: String = format!("log/DivaFFMPEG-{}.log", Utc::now().format("%Y-%m-%d_%H-%M-%S"));
    let mut _file: File = OpenOptions::new()
    .append(true)
    .create(true)
    .open(&filename)?;

    // Logging Var
    let mut last_logged_state = String::new();

    let backend: CrosstermBackend<Stdout> = CrosstermBackend::new(stdout);
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = Terminal::new(backend)?;
    
    //                      <-- APPLICATION STATE -->
    
    let mut safe_trigger_exit_terminal: bool = false;
    let mut home_scene: bool = false;
    let mut image_processing_scene: bool = false;
    let mut video_processing_scene: bool = false;
    let mut audio_processing_scene: bool = false;

    // Video Processing State
    let mut video_state: VideoProcessingState = VideoProcessingState::Default;
    let mut video_menu: DirectionsMenuState = DirectionsMenuState::new();

    loop
    {
        terminal.draw( |frame| {
            // Creates our termainal frame.
            let size: Rect = frame.area();
            
            // The goal of this block is to render a splash screen for a user who is trying to exit the TUI.
            // Directions on how to restore the state previous to pressing q outside of a text input situation
            // are displayed. External links to app support and developer socials are also provided.
            if safe_trigger_exit_terminal {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Fill(1),
                        ])
                        .split(size);

                    let intro_block = Block::default()
                        .borders(Borders::ALL)
                        .title("Diva FFMPEG")
                        .title_alignment(Alignment::Center)
                        .bold();

                    let intro_inner = intro_block.inner(outer[0]);

                    frame.render_widget(intro_block, outer[0]);

                    let inner: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                        Constraint::Fill(1),
                        Constraint::Length(1),
                        Constraint::Fill(1),
                        Constraint::Length(1),
                        ])
                        .split(intro_inner);

                    let exit_page: Paragraph<'_> = Paragraph::new(styles::title(inner[0].width, inner[0].height).render(
                        "Press 'esc' to return to the previous screen. \nPress 'q' again to exit! 💋").as_bytes().into_text().unwrap());

                    frame.render_widget(exit_page, inner[0]);
                    frame.render_widget(reusable_widgets::social_footer_hyperlinks(inner[0].width, inner[0].height), inner[3]);
                
                    if last_logged_state != "Exit Screen".to_string() {
                    let _ = writeln!(_file, "{}", format!("Entered the exit screen at {}.", Utc::now().format("%H-%M-%S")));
                    last_logged_state = "Exit Screen".to_string();
                }
            } else {
                // The goal of this block is to render a splash screen for those who open the app.
                // A welcoming text, input directions, and external links to app support and
                // developer socials should be shown.
                let outer: Rc<[Rect]> = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Fill(1),
                    ])
                    .split(size);

                let intro_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Diva FFMPEG")
                    .title_alignment(Alignment::Center)
                    .bold();

                let intro_inner = intro_block.inner(outer[0]);

                frame.render_widget(intro_block, outer[0]);

                let inner: Rc<[Rect]> = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                    Constraint::Fill(1),
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    ])
                    .split(intro_inner);

                let introduction: Paragraph<'_> = Paragraph::new(styles::title(inner[0].width, inner[0].height).render(
                    "Welcome to Diva FFMPEG! \nStylish but functional ffmpeg wrapper 💅 \nBy Marqed").as_bytes().into_text().unwrap());

                let directions: Paragraph<'_> = Paragraph::new(styles::directions(inner[1].width, inner[1].height).blink().render(
                    "Press 'enter' to get started. \nPress 'q' again to quit! 💋").as_bytes().into_text().unwrap());

                frame.render_widget(introduction, inner[0]);
                frame.render_widget(directions, inner[1]);
                frame.render_widget(reusable_widgets::social_footer_hyperlinks(inner[0].width, inner[0].height), inner[3]);

                if last_logged_state != "Start Screen".to_string() {
                    let _ = writeln!(_file, "{}", format!("Entered the start screen at {}.", Utc::now().format("%H-%M-%S")));
                    last_logged_state = "Start Screen".to_string();
                }
            }


            // The goal of this block is to render the portition of this TUI that displays information
            // on how to reach and enter abstracted scenes involving image_processing, video_processing, audio_processing, etc.
            if home_scene {
                todo!()
            }

            // The goal of this block is to render the portition of this TUI that displays information
            // and control used for rendering.
            if video_processing_scene {
               video::render(frame, video_state, &video_menu);
            }
        })?;

        // The purpose of this block is to catch key strokes that trigger events and change code state.
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    //                      <-- APPLICATION KEY EVENTS STATE -->
                    KeyCode::Char('q') if !safe_trigger_exit_terminal => 
                    {
                        safe_trigger_exit_terminal = true;
                        writeln!(_file, "{}", format!("State: 'safe_trigger_exit' became {} at {} because {:?} was pressed so the prompt to close the application was initiated.",
                        safe_trigger_exit_terminal, Utc::now().format("%H-%M-%S"), k.code))?;
                        let _ = event::read()?;
                    },
                    KeyCode::Char('q') if safe_trigger_exit_terminal =>
                    {
                        writeln!(_file, "{}", format!("State: 'safe_trigger_exit_terminal' became {} at {} because {:?} was pressed so the application was closed.",
                        safe_trigger_exit_terminal, Utc::now().format("%H-%M-%S"), k.code))?;
                        break;
                    },
                    KeyCode::Esc if safe_trigger_exit_terminal =>
                    {
                        safe_trigger_exit_terminal = false;
                        writeln!(_file, "{}", format!("State: 'safe_trigger_exit_terminal' became {} at {} because {:?} was pressed.",
                        safe_trigger_exit_terminal, Utc::now().format("%H-%M-%S"), k.code))?;
                    },
                    // Check if any of the other alternate screens are not activated. If not
                    // the app has just started, we're entering the home_menu.
                    KeyCode::Enter if !image_processing_scene && !video_processing_scene && !audio_processing_scene =>
                    {
                        home_scene = true;
                        writeln!(_file, "{}", format!("Entered 'home_scene' at {} because {:?} was pressed.",
                        Utc::now().format("%H-%M-%S"), k.code))?;
                    },

                    //                      <-- VIDEO PROCESSING SCENE KEY EVENTS -->
                    
                    KeyCode::Left if video_processing_scene => video_menu.previous(),
                    KeyCode::Right if video_processing_scene => video_menu.next(),
                    KeyCode::Esc if video_processing_scene => {
                        video_state = video_menu.selected_state();
                    },
                    KeyCode::Esc if video_processing_scene && video_state == VideoProcessingState::Default => {
                        todo!()
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