use std::{fs::File, io::{self, Stdout}, rc::Rc};
use ansi_to_tui::IntoText as _;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use ratatui::widgets::*;

use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;

mod home;
mod video;
mod reusable_widgets;
mod styles;
mod background; 

use home::{HomeState, HomeMenuState};
use video::{VideoProcessingState, DirectionsMenuState};

//                      <-- SCENE STATE -->

// Tracks exactly which screen is live at any given time. Using a single enum
// instead of several independent bools makes it structurally impossible for
// two scenes to render (or receive key input) in the same frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scene {
    Start,
    Home,
    ImageProcessing, // W.I.P.
    VideoProcessing,
    AudioProcessing, // W.I.P.
    Exit,
}

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
    let mut current_scene: Scene = Scene::Start;
    // Remembers which scene to return to when the user backs out of the exit prompt.
    let mut previous_scene: Scene = Scene::Start;

    // Home Menu States
    let mut home_state: HomeState = HomeState::Default;
    let mut home_menu: HomeMenuState = HomeMenuState::new();

    // Video Processing State
    let mut video_state: VideoProcessingState = VideoProcessingState::Default;
    let mut video_menu: DirectionsMenuState = DirectionsMenuState::new();

    // Drain any stray input events left over from launching the process
    // (e.g. the Enter keystroke used to run the binary) so they don't
    // get misinterpreted as real user input once the app starts.
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }

    loop
    {
        terminal.draw(|frame| {
            // Creates our terminal frame.
            let size: Rect = frame.area();

            match current_scene {
                //                      <-- EXIT SCENE -->
                // The goal of this block is to render a splash screen for a user who is trying to exit the TUI.
                // Directions on how to restore the state previous to pressing q outside of a text input situation
                // are displayed. External links to app support and developer socials are also provided.
                Scene::Exit => {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Fill(1),
                        ])
                        .split(size);

                    let intro_block = Block::default()
                        .borders(Borders::ALL)
                        .title("| Diva FFMPEG: Exit |")
                        .title_alignment(Alignment::Center)
                        .bold();

                    let intro_inner = intro_block.inner(outer[0]);

                    frame.render_widget(intro_block, outer[0]);

                    let inner: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Fill(1),
                            Constraint::Length(1),
                            Constraint::Min(1),
                            Constraint::Length(1),
                        ])
                        .split(intro_inner);

                    let exit_page: Paragraph<'_> = Paragraph::new(styles::title(inner[0].width, inner[0].height).render(
                        "Press 'ESC' to return to the PREVIOUS SCREEN.\nPress 'Q' again to EXIT! 💋").as_bytes().into_text().unwrap());
                    
                    frame.render_widget(exit_page, inner[0]);
                    frame.render_widget(reusable_widgets::social_footer_hyperlinks(inner[3].width, inner[3].height), inner[3]);

                    if last_logged_state != "Exit Screen" {
                        let _ = writeln!(_file, "Entered the exit screen at {}.", Utc::now().format("%H-%M-%S"));
                        last_logged_state = "Exit Screen".to_string();
                    }
                },

                //                      <-- START SCENE -->
                // The goal of this block is to render a splash screen for those who open the app.
                // A welcoming text, input directions, and external links to app support and
                // developer socials should be shown.
                Scene::Start => {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Fill(1)])
                        .split(size);

                    let intro_block = Block::default()
                        .borders(Borders::ALL)
                        .title("| Diva FFMPEG |")
                        .title_alignment(Alignment::Center)
                        .bold();

                    let intro_inner = intro_block.inner(outer[0]);
                    frame.render_widget(intro_block, outer[0]);

                    let inner: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(6),    // inner[0]: welcome text
                            Constraint::Length(16),   // inner[1]: art
                            Constraint::Length(1),    // inner[2]: directions bar
                            Constraint::Length(16),   // inner[3]: art
                            Constraint::Length(1),    // inner[4]: footer links
                        ])
                        .split(intro_inner);

                    background::render_diva_top(frame, inner[1]); // pass the middle chunk, not `size`
                    background::render_diva_bottom(frame, inner[3]); // pass the middle chunk, not `size`

                    let introduction: Paragraph<'_> = Paragraph::new(styles::title(inner[0].width, inner[0].height).render(
                        "Welcome to Diva FFMPEG! \nSlay your multimedia pipeline 💅 \nBy Marqed").as_bytes().into_text().unwrap());

                    let directions: Paragraph<'_> = Paragraph::new(styles::center_directions(inner[1].width, inner[1].height).blink().render(
                        "Press 'ENTER' to get started OR 'Q' to EXIT! 💋").as_bytes().into_text().unwrap());

                    frame.render_widget(introduction, inner[0]);
                    frame.render_widget(directions, inner[2]);
                    frame.render_widget(reusable_widgets::social_footer_hyperlinks(inner[4].width, inner[4].height), inner[4]);

                    if last_logged_state != "Start Screen" {
                        let _ = writeln!(_file, "Entered the start screen at {}.", Utc::now().format("%H-%M-%S"));
                        last_logged_state = "Start Screen".to_string();
                    }
                },

                //                      <-- HOME SCENE -->
                // The goal of this block is to render the portion of this TUI that displays information
                // on how to reach and enter abstracted scenes involving image_processing, video_processing, audio_processing, etc.
                Scene::Home => {
                    home::render(frame, home_state, &home_menu);

                    if last_logged_state != "Home Screen" {
                        let _ = writeln!(_file, "Entered the home screen at {}.", Utc::now().format("%H-%M-%S"));
                        last_logged_state = "Home Screen".to_string();
                    }
                },

                //                      <-- VIDEO PROCESSING SCENE -->
                // The goal of this block is to render the portion of this TUI that displays information
                // and control used for video processing.
                Scene::VideoProcessing => {
                    video::render(frame, video_state, &video_menu);

                    if last_logged_state != "Video Processing Screen" {
                        let _ = writeln!(_file, "Entered the video processing screen at {}.", Utc::now().format("%H-%M-%S"));
                        last_logged_state = "Video Processing Screen".to_string();
                    }
                },

                //                      <-- IMAGE PROCESSING SCENE (W.I.P.) -->
                Scene::ImageProcessing => {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Fill(1),
                        ])
                        .split(size);

                    let wip_block = Block::default()
                        .borders(Borders::ALL)
                        .title("Diva FFMPEG: Image Processing")
                        .title_alignment(Alignment::Center)
                        .bold();

                    let wip_inner = wip_block.inner(outer[0]);
                    frame.render_widget(wip_block, outer[0]);

                    let wip_text = Paragraph::new(
                        styles::title(wip_inner.width, wip_inner.height).render("Work in progress...").as_bytes().into_text().unwrap()
                    ).alignment(Alignment::Center);

                    frame.render_widget(wip_text, wip_inner);

                    if last_logged_state != "Image Processing Screen" {
                        let _ = writeln!(_file, "Entered the image processing screen at {}.", Utc::now().format("%H-%M-%S"));
                        last_logged_state = "Image Processing Screen".to_string();
                    }
                },

                //                      <-- AUDIO PROCESSING SCENE (W.I.P.) -->
                Scene::AudioProcessing => {
                    let outer: Rc<[Rect]> = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Fill(1),
                        ])
                        .split(size);

                    let wip_block = Block::default()
                        .borders(Borders::ALL)
                        .title("Diva FFMPEG: Audio Processing")
                        .title_alignment(Alignment::Center)
                        .bold();

                    let wip_inner = wip_block.inner(outer[0]);
                    frame.render_widget(wip_block, outer[0]);

                    let wip_text = Paragraph::new(
                        styles::title(wip_inner.width, wip_inner.height).render("Work in progress...").as_bytes().into_text().unwrap()
                    ).alignment(Alignment::Center);

                    frame.render_widget(wip_text, wip_inner);

                    if last_logged_state != "Audio Processing Screen" {
                        let _ = writeln!(_file, "Entered the audio processing screen at {}.", Utc::now().format("%H-%M-%S"));
                        last_logged_state = "Audio Processing Screen".to_string();
                    }
                },
            }
        })?;

        // The purpose of this block is to catch key strokes that trigger events and change code state.
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(k) = event::read()? {
                if k.kind == event::KeyEventKind::Press {
                    match (current_scene, k.code) {
                        //                      <-- GLOBAL EXIT KEY EVENTS -->
                        (scene, KeyCode::Char('q')) if scene != Scene::Exit => {
                            previous_scene = scene;
                            current_scene = Scene::Exit;
                            writeln!(_file, "State: entered the exit prompt at {} from {:?} because {:?} was pressed.",
                                Utc::now().format("%H-%M-%S"), previous_scene, k.code)?;
                        },
                        (Scene::Exit, KeyCode::Char('q')) => {
                            writeln!(_file, "State: application closed at {} because {:?} was pressed.",
                                Utc::now().format("%H-%M-%S"), k.code)?;
                            break;
                        },
                        (Scene::Exit, KeyCode::Esc) => {
                            current_scene = previous_scene;
                            writeln!(_file, "State: exited the exit prompt at {}, returning to {:?}.",
                                Utc::now().format("%H-%M-%S"), current_scene)?;
                        },

                        //                      <-- START SCENE KEY EVENTS -->
                        // If the app has just started, entering advances us to the home menu.
                        (Scene::Start, KeyCode::Enter) => {
                            current_scene = Scene::Home;
                            writeln!(_file, "Entered 'Scene::Home' at {} because {:?} was pressed.",
                                Utc::now().format("%H-%M-%S"), k.code)?;
                        },

                        //                      <-- HOME SCENE KEY EVENTS -->
                        (Scene::Home, KeyCode::Left) | (Scene::Home, KeyCode::Char('a')) => home_menu.previous(),
                        (Scene::Home, KeyCode::Right) | (Scene::Home, KeyCode::Char('d')) => home_menu.next(),
                        (Scene::Home, KeyCode::Enter) => {
                            home_state = home_menu.selected_state();

                            // Image and audio scenes are still W.I.P. placeholders, but they're
                            // still reachable so their in-progress layouts can be seen and iterated on.
                            current_scene = match home_state {
                                HomeState::Image => Scene::ImageProcessing,
                                HomeState::Video => Scene::VideoProcessing,
                                HomeState::Audio => Scene::AudioProcessing,
                                HomeState::Default => Scene::Home,
                            };

                            writeln!(_file, "State: 'home_state' became {:?} at {}, transitioning to {:?}.",
                                home_state, Utc::now().format("%H-%M-%S"), current_scene)?;
                        },

                        //                      <-- VIDEO PROCESSING SCENE KEY EVENTS -->
                        (Scene::VideoProcessing, KeyCode::Enter) => {
                            video_state = video_menu.selected_state();
                        },
                        (Scene::VideoProcessing, KeyCode::Esc) => {
                            video_state = VideoProcessingState::Default;
                            current_scene = Scene::Home;
                        },

                        //                      <-- IMAGE / AUDIO PROCESSING SCENE KEY EVENTS (W.I.P.) -->
                        (Scene::ImageProcessing, KeyCode::Esc) | (Scene::AudioProcessing, KeyCode::Esc) => {
                            current_scene = Scene::Home;
                        },

                        _ => {}
                    }
                }

                if k.kind == event::KeyEventKind::Repeat {
                    if k.kind == event::KeyEventKind::Press {
                        match (current_scene, k.code) {
                            //                      <-- VIDEO PROCESSING SCENE KEY EVENTS -->
                            (Scene::VideoProcessing, KeyCode::Left) | (Scene::VideoProcessing, KeyCode::Char('a')) => video_menu.previous(),
                            (Scene::VideoProcessing, KeyCode::Right) | (Scene::VideoProcessing, KeyCode::Char('d')) => video_menu.next(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}