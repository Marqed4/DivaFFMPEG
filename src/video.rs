use std::{rc::Rc};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use ansi_to_tui::IntoText as _;

use ratatui_textarea::TextArea;
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::explanations;
use crate::background;
use crate::component;
use crate::styles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoProcessingState {
    Default,
    Convert,
    ExplainConvert,
    Compress,
    ExplainCompress,
    Trim,
    ExplainTrim,
    Merge,
    ExplainMerge,
}

//                      <-- SELECTABLES -->
impl VideoProcessingState {
    const SELECTABLES: [VideoProcessingState; 5] = [
        VideoProcessingState::Default,
        VideoProcessingState::Convert,
        VideoProcessingState::Compress,
        VideoProcessingState::Trim,
        VideoProcessingState::Merge,
    ];

    const EXPLANATIONS: [VideoProcessingState; 4] = [
        VideoProcessingState::ExplainConvert,
        VideoProcessingState::ExplainCompress,
        VideoProcessingState::ExplainTrim,
        VideoProcessingState::ExplainMerge,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Convert => "   Convert   ",
            Self::ExplainConvert => "Convert Guide",
            Self::Compress => "   Compress   ",
            Self::ExplainCompress => "Compress Guide",
            Self::Trim => "   Trim   ",
            Self::ExplainTrim => "Trim Guide",
            Self::Merge => "   Merge   ",
            Self::ExplainMerge => "Merge Guide",
        }
    }
}

#[derive(Debug)]
pub struct DirectionsMenuState {
    selected_row: usize,
    selected_column: usize,
}

impl DirectionsMenuState {
    pub fn new() -> Self {
        Self { selected_row: 0, selected_column: 0 }
    }

    pub fn next(&mut self) {
        self.selected_column = (self.selected_column + 1) % 4;
    }

    pub fn previous(&mut self) {
        self.selected_column = if self.selected_column == 0 { 3 } else { self.selected_column - 1 };
    }

    pub fn above(&mut self) {
        self.selected_row = if self.selected_row == 0 { 1 } else { self.selected_row - 1 };
    }

    pub fn below(&mut self) {
        self.selected_row = (self.selected_row + 1) % 2;
    }

    pub fn selected_state(&self) -> VideoProcessingState {
        if self.selected_row == 0 {
            VideoProcessingState::SELECTABLES[self.selected_column + 1]
        } else {
            VideoProcessingState::EXPLANATIONS[self.selected_column]
        }
    }
}

//                      <-- CONVERT OPTIONS -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertFormat { Mp4, Mkv, WebM, Mov, Avi, Ts, Mpd, Mxf, Flv, Gp3, Mpeg, Wmv, Gif }

impl ConvertFormat {
    const ALL: [ConvertFormat; 13] = [
        ConvertFormat::Mp4, ConvertFormat::Mkv, ConvertFormat::WebM, ConvertFormat::Mov,
        ConvertFormat::Avi, ConvertFormat::Ts, ConvertFormat::Mpd, ConvertFormat::Mxf,
        ConvertFormat::Flv, ConvertFormat::Gp3, ConvertFormat::Mpeg, ConvertFormat::Wmv,
        ConvertFormat::Gif,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Mp4 => "🎬 .mp4", Self::Mkv => ".mkv", Self::WebM => "🌐 .webm",
            Self::Mov => ".mov", Self::Avi => ".avi", Self::Ts => ".ts",
            Self::Mpd => ".mpd", Self::Mxf => ".mxf", Self::Flv => ".flv",
            Self::Gp3 => "📱 .3gp", Self::Mpeg => ".mpeg", Self::Wmv => ".wmv",
            Self::Gif => "🖼 .gif",
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4", Self::Mkv => "mkv", Self::WebM => "webm", Self::Mov => "mov",
            Self::Avi => "avi", Self::Ts => "ts", Self::Mpd => "mpd", Self::Mxf => "mxf",
            Self::Flv => "flv", Self::Gp3 => "3gp", Self::Mpeg => "mpeg", Self::Wmv => "wmv",
            Self::Gif => "gif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertCodec { H264, H265 }

impl ConvertCodec {
    const ALL: [ConvertCodec; 2] = [ConvertCodec::H264, ConvertCodec::H265];

    fn label(&self) -> &'static str {
        match self {
            Self::H264 => "🏃 H.264", Self::H265 => "🐌 H.265",
        }
    }

    fn ffmpeg_flag(&self) -> &'static str {
        match self {
            Self::H264 => "libx264", Self::H265 => "libx265",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertResolution { Original, R1080, R720, R480, R360 }

impl ConvertResolution {
    const ALL: [ConvertResolution; 5] = [
        ConvertResolution::Original, ConvertResolution::R1080, ConvertResolution::R720,
        ConvertResolution::R480, ConvertResolution::R360,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Original => "🔳 original", Self::R1080 => "1080p", Self::R720 => "720p",
            Self::R480 => "480p", Self::R360 => "360p",
        }
    }

    fn scale(&self) -> Option<(u32, u32)> {
        match self {
            Self::Original => None,
            Self::R1080 => Some((1920, 1080)),
            Self::R720 => Some((1280, 720)),
            Self::R480 => Some((854, 480)),
            Self::R360 => Some((640, 360)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertFps { Original, Fps60, Fps30, Fps24 }

impl ConvertFps {
    const ALL: [ConvertFps; 4] = [
        ConvertFps::Original, ConvertFps::Fps60, ConvertFps::Fps30, ConvertFps::Fps24,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::Original => "🔳 original", Self::Fps60 => "60fps", Self::Fps30 => "30fps",
            Self::Fps24 => "24fps",
        }
    }

    fn value(&self) -> Option<u32> {
        match self {
            Self::Original => None, Self::Fps60 => Some(60), Self::Fps30 => Some(30), Self::Fps24 => Some(24),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertCrf { Auto, High, Medium, Low }

impl ConvertCrf {
    const ALL: [ConvertCrf; 4] = [ConvertCrf::Auto, ConvertCrf::High, ConvertCrf::Medium, ConvertCrf::Low];

    fn label(&self) -> &'static str {
        match self {
            Self::Auto => "🔳 auto", Self::High => "✨ high", Self::Medium => "quality", Self::Low => "📦 small",
        }
    }

    fn value(&self) -> Option<u8> {
        match self {
            Self::Auto => None, Self::High => Some(18), Self::Medium => Some(23), Self::Low => Some(28),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertField { InputPath, Format, Codec, Resolution, Fps, Crf, OutputPath, Run }

impl ConvertField {
    const ALL: [ConvertField; 8] = [
        ConvertField::InputPath, ConvertField::Format, ConvertField::Codec,
        ConvertField::Resolution, ConvertField::Fps, ConvertField::Crf,
        ConvertField::OutputPath, ConvertField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::Format => "format", Self::Codec => "codec",
            Self::Resolution => "res", Self::Fps => "fps", Self::Crf => "quality",
            Self::OutputPath => "💾 output", Self::Run => "▶ convert",
        }
    }
}

#[derive(Debug)]
pub struct ConvertMenuState {
    focus: usize,
    pub editing: bool,
}

impl ConvertMenuState {
    pub fn new() -> Self {
        Self { focus: 0, editing: false }
    }

    pub fn focus_field(&self) -> ConvertField {
        ConvertField::ALL[self.focus]
    }

    pub fn next(&mut self) {
        self.focus = (self.focus + 1) % ConvertField::ALL.len();
    }

    pub fn previous(&mut self) {
        self.focus = if self.focus == 0 { ConvertField::ALL.len() - 1 } else { self.focus - 1 };
    }
}

//                      <-- CONVERT JOB (background ffmpeg process) -->

enum ConvertMsg {
    Duration(f64),
    Time(f64),
    Done(Result<(), String>),
}

pub struct ConvertJob {
    rx: Receiver<ConvertMsg>,
    duration: f64,
    elapsed: f64,
    finished: Option<Result<(), String>>,
}

impl ConvertJob {
    fn poll(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(ConvertMsg::Duration(d)) => self.duration = d,
                Ok(ConvertMsg::Time(t)) => self.elapsed = t,
                Ok(ConvertMsg::Done(r)) => self.finished = Some(r),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    fn ratio(&self) -> f64 {
        if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else if self.finished.is_some() {
            1.0
        } else {
            0.0
        }
    }

    fn status_text(&self) -> String {
        match &self.finished {
            Some(Ok(())) => "✅ done!".to_string(),
            Some(Err(e)) => format!("❌ {e}"),
            None => format!("⏳ converting... {:.0}%", self.ratio() * 100.0),
        }
    }
}

fn parse_timestamp(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 { return None; }
    let hours: f64 = parts[0].parse().ok()?;
    let minutes: f64 = parts[1].parse().ok()?;
    let seconds: f64 = parts[2].parse().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

fn parse_tagged_timestamp(line: &str, tag: &str) -> Option<f64> {
    let start = line.find(tag)? + tag.len();
    let rest = &line[start..];
    let end = rest.find(|c: char| c != ':' && c != '.' && !c.is_ascii_digit()).unwrap_or(rest.len());
    parse_timestamp(&rest[..end])
}

//                      <-- CONVERT SCREEN STATE -->

pub struct ConvertState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub menu: ConvertMenuState,
    format: ConvertFormat,
    codec: ConvertCodec,
    resolution: ConvertResolution,
    fps: ConvertFps,
    crf: ConvertCrf,
    job: Option<ConvertJob>,
}

impl ConvertState {
    pub fn new() -> Self {
        Self {
            input_file_path: TextArea::default(),
            output_file_path: TextArea::default(),
            menu: ConvertMenuState::new(),
            format: ConvertFormat::Mp4,
            codec: ConvertCodec::H264,
            resolution: ConvertResolution::Original,
            fps: ConvertFps::Original,
            crf: ConvertCrf::Auto,
            job: None,
        }
    }

    pub fn cycle_value(&mut self, forward: bool) {
        match self.menu.focus_field() {
            ConvertField::Format => self.format = Self::cycle(&ConvertFormat::ALL, self.format, forward),
            ConvertField::Codec => self.codec = Self::cycle(&ConvertCodec::ALL, self.codec, forward),
            ConvertField::Resolution => self.resolution = Self::cycle(&ConvertResolution::ALL, self.resolution, forward),
            ConvertField::Fps => self.fps = Self::cycle(&ConvertFps::ALL, self.fps, forward),
            ConvertField::Crf => self.crf = Self::cycle(&ConvertCrf::ALL, self.crf, forward),
            ConvertField::InputPath | ConvertField::OutputPath | ConvertField::Run => {},
        }
    }

    fn cycle<T: PartialEq + Copy>(all: &[T], current: T, forward: bool) -> T {
        let idx = all.iter().position(|v| *v == current).unwrap_or(0);
        let next_idx = if forward {
            (idx + 1) % all.len()
        } else if idx == 0 {
            all.len() - 1
        } else {
            idx - 1
        };
        all[next_idx]
    }

    pub fn start_convert(&mut self) {
        let input = self.input_file_path.lines().join("");
        let mut output = self.output_file_path.lines().join("");
        if input.trim().is_empty() || output.trim().is_empty() {
            return;
        }

        // Swap in the chosen format's extension, replacing whatever the user typed (if any).
        if let Some(dot) = output.rfind('.') {
            output.truncate(dot);
        }
        output.push('.');
        output.push_str(self.format.extension());

        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), input];

        if let Some((w, h)) = self.resolution.scale() {
            args.push("-vf".into());
            args.push(format!("scale={w}:{h}"));
        }
        if let Some(fps) = self.fps.value() {
            args.push("-r".into());
            args.push(fps.to_string());
        }
        args.push("-c:v".into());
        args.push(self.codec.ffmpeg_flag().into());
        if let Some(crf) = self.crf.value() {
            args.push("-crf".into());
            args.push(crf.to_string());
        }
        args.push(output);

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let spawned = Command::new("ffmpeg")
                .args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn();

            match spawned {
                Ok(mut child) => {
                    if let Some(stderr) = child.stderr.take() {
                        for line in BufReader::new(stderr).lines().flatten() {
                            if let Some(d) = parse_tagged_timestamp(&line, "Duration: ") {
                                let _ = tx.send(ConvertMsg::Duration(d));
                            }
                            if let Some(t) = parse_tagged_timestamp(&line, "time=") {
                                let _ = tx.send(ConvertMsg::Time(t));
                            }
                        }
                    }

                    let result = match child.wait() {
                        Ok(status) if status.success() => Ok(()),
                        Ok(status) => Err(format!("ffmpeg exited with {status}")),
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = tx.send(ConvertMsg::Done(result));
                },
                Err(e) => {
                    let _ = tx.send(ConvertMsg::Done(Err(format!("couldn't launch ffmpeg: {e}"))));
                },
            }
        });

        self.job = Some(ConvertJob { rx, duration: 0.0, elapsed: 0.0, finished: None });
    }
}

pub fn render(
    frame: &mut Frame<'_>,
    state: VideoProcessingState,
    menu: &DirectionsMenuState,
    convert_state: &mut ConvertState,
) {
    let size: Rect = frame.area();

    let outer: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
        ])
        .split(size);

    let video_block = Block::default()
        .borders(Borders::ALL)
        .title("| Diva FFMPEG: Video Processing |")
        .title_alignment(Alignment::Center)
        .bold();

    let video_inner: Rect = video_block.inner(outer[0]);
    frame.render_widget(video_block, outer[0]);

    let inner: Rc<[Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // inner[0]: empty/unused space
            Constraint::Length(3),  // inner[1]: selection guide
            Constraint::Length(1),  // inner[2]: new line gap
            Constraint::Length(1),  // inner[3]: SELECTABLES tabs
            Constraint::Length(1),  // inner[4]: EXPLANATIONS tabs
            Constraint::Length(3),  // inner[5]: EXPLANATIONS/USER INTERFACE
            Constraint::Fill(1),    // inner[6]: art & UI
            Constraint::Length(1),  // inner[7]: footer links
        ])
        .split(video_inner);

    let selection_guide: Paragraph<'_> = Paragraph::new(
        styles::center_directions(inner[1].width, inner[1].height)
            .render(
                "Press '\x1b[38;5;218m\x1b[1mA\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mleft arrow-key\x1b[22m\x1b[39m' OR '\x1b[38;5;218m\x1b[1mD\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mright arrow-key\x1b[22m\x1b[39m' \n \
                Press '\x1b[38;5;218m\x1b[1mW\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mup arrow-key\x1b[22m\x1b[39m' OR '\x1b[38;5;218m\x1b[1mS\x1b[22m\x1b[39m' or '\x1b[38;5;218m\x1b[1mdown arrow-key\x1b[22m\x1b[39m' to MOVE THE SELECTION HIGHLIGHT. \n \
                Press '\x1b[38;5;205m\x1b[1mESC\x1b[22m\x1b[39m' to return to the PREVIOUS SCREEN or '\x1b[38;5;205m\x1b[1mQ\x1b[22m\x1b[39m' to EXIT! 💋"
            )
            .as_bytes()
            .into_text()
            .unwrap()
    );

    //                      <-- TAB TITLES -->
    let selectable_titles: Vec<Line> = VideoProcessingState::SELECTABLES
        .iter()
        .filter(|s| **s != VideoProcessingState::Default)
        .map(|s| Line::from(s.label()))
        .collect();

    let explanation_titles: Vec<Line> = VideoProcessingState::EXPLANATIONS
        .iter()
        .map(|s| Line::from(s.label()))
        .collect();

    //                      <-- TAB ELEMENTS -->
    let selectable_tabs = Tabs::new(selectable_titles)
        .select(if menu.selected_row == 0 { Some(menu.selected_column) } else { None })
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .divider(" | ");

    let explanation_tabs = Tabs::new(explanation_titles)
        .select(if menu.selected_row == 1 { Some(menu.selected_column) } else { None })
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .divider(" | ");

    //                      <-- DIRECTIONS -->
    // Explanations run down the LEFT side of inner[6], sharing rows with the
    // art on the right — art's own rect/rendering stays untouched.
    match state {
        VideoProcessingState::Default => {},
        VideoProcessingState::Convert => {
            if let Some(job) = convert_state.job.as_mut() {
                job.poll();
            }

            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line
                Constraint::Length(1),  // rows[1]: status line
                Constraint::Length(1),  // rows[2]: empty/unused space
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(80), // cols[0]: fields + progress
                Constraint::Percentage(20), // cols[1]: art
            ])
            .split(inner[6]);

            let intro_text = "WASD to move between fields below \u{2022} A/D changes focus, W/S cycles a value \u{2022} ENTER edits a path or runs convert.";
            frame.render_widget(
                Paragraph::new(styles::left_directions(rows[0].width, rows[0].height).render(intro_text).as_bytes().into_text().unwrap()),
                rows[0],
            );

            let status_text: String = match convert_state.job.as_ref() {
                Some(job) => job.status_text(),
                None => "Idle \u{2014} fill in the fields, then focus ▶ convert and press ENTER.".to_string(),
            };
            frame.render_widget(
                Paragraph::new(styles::left_directions(rows[1].width, rows[1].height).render(&status_text).as_bytes().into_text().unwrap()),
                rows[1],
            );

            let field_area: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // field_area[0]: focusable field boxes
                Constraint::Length(3),  // field_area[1]: progress gauge
            ])
            .split(cols[0]);

            let field_cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(2),    // input path
                Constraint::Length(12), // format
                Constraint::Length(12), // codec
                Constraint::Length(12), // resolution
                Constraint::Length(10), // fps
                Constraint::Length(12), // quality/crf
                Constraint::Fill(2),    // output path
                Constraint::Length(12), // run
            ])
            .split(field_area[0]);

            let focused = convert_state.menu.focus_field();

            let field_block = |field: ConvertField| -> Block<'static> {
                let focused_here = focused == field;
                let style = if focused_here && convert_state.menu.editing {
                    Style::default().fg(Color::Rgb(255, 133, 200)).add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else if focused_here {
                    Style::default().fg(Color::Rgb(255, 133, 200)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(150, 150, 150))
                };

                Block::default()
                    .borders(Borders::ALL)
                    .border_style(style)
                    .title(field.label())
            };

            let input_block = field_block(ConvertField::InputPath);
            let input_inner = input_block.inner(field_cols[0]);
            frame.render_widget(input_block, field_cols[0]);
            frame.render_widget(&convert_state.input_file_path, input_inner);

            let value_field = |field: ConvertField, value: &str, area: Rect, frame: &mut Frame<'_>| {
                let block = field_block(field);
                let inner = block.inner(area);
                frame.render_widget(block, area);
                frame.render_widget(
                    Paragraph::new(value).alignment(Alignment::Center).style(Style::default().fg(Color::White)),
                    inner,
                );
            };

            value_field(ConvertField::Format, convert_state.format.label(), field_cols[1], frame);
            value_field(ConvertField::Codec, convert_state.codec.label(), field_cols[2], frame);
            value_field(ConvertField::Resolution, convert_state.resolution.label(), field_cols[3], frame);
            value_field(ConvertField::Fps, convert_state.fps.label(), field_cols[4], frame);
            value_field(ConvertField::Crf, convert_state.crf.label(), field_cols[5], frame);

            let output_block = field_block(ConvertField::OutputPath);
            let output_inner = output_block.inner(field_cols[6]);
            frame.render_widget(output_block, field_cols[6]);
            frame.render_widget(&convert_state.output_file_path, output_inner);

            let run_block = field_block(ConvertField::Run);
            let run_inner = run_block.inner(field_cols[7]);
            frame.render_widget(run_block, field_cols[7]);
            frame.render_widget(
                Paragraph::new(ConvertField::Run.label()).alignment(Alignment::Center).style(Style::default().fg(Color::White)),
                run_inner,
            );

            let ratio = convert_state.job.as_ref().map(|j| j.ratio()).unwrap_or(0.0);
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("progress"))
                .gauge_style(Style::default().fg(Color::Rgb(255, 133, 200)))
                .ratio(ratio);
            frame.render_widget(gauge, field_area[1]);
        },
        VideoProcessingState::ExplainConvert => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(80), // cols[0]: format list
                Constraint::Percentage(20), // cols[1]: art
            ])
            .split(inner[6]);

            let explain_rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(14), // explain_rows[0]: format list
                Constraint::Length(3),  // explain_rows[1]: codec list
                Constraint::Fill(1),    // explain_rows[2]: unused space
            ])
            .split(cols[0]);

            frame.render_widget(explanations::explain_convert_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_convert_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_convert_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_convert_format_list(explain_rows[0].width, explain_rows[0].height), explain_rows[0]);
            frame.render_widget(explanations::explain_convert_codec_list(explain_rows[1].width, explain_rows[1].height), explain_rows[1]);
        },
        VideoProcessingState::Compress => {},
        VideoProcessingState::ExplainCompress => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line (black bg)
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: format list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_compress_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_compress_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_compress_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_compress_format_list(cols[0].width, cols[0].height), cols[0]);
        },
        VideoProcessingState::Trim => {},
        VideoProcessingState::ExplainTrim => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line (black bg)
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: format list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_trim_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_trim_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_trim_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_trim_format_list(cols[0].width, cols[0].height), cols[0]);
        },
        VideoProcessingState::Merge => {},
        VideoProcessingState::ExplainMerge => {
            let rows: Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // rows[0]: intro line 1
                Constraint::Length(1),  // rows[1]: intro line 2
                Constraint::Length(1),  // rows[2]: outro line (black bg)
            ])
            .split(inner[5]);

            let cols: Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // cols[0]: format list
                Constraint::Percentage(40), // cols[1]: art
            ])
            .split(inner[6]);

            frame.render_widget(explanations::explain_merge_intro_line_1(rows[0].width, rows[0].height), rows[0]);
            frame.render_widget(explanations::explain_merge_intro_line_2(rows[1].width, rows[1].height), rows[1]);
            frame.render_widget(explanations::explain_merge_outro_line(rows[2].width, rows[2].height), rows[2]);
            frame.render_widget(explanations::explain_merge_format_list(cols[0].width, cols[0].height), cols[0]);
        },
    }

    frame.render_widget(selection_guide, inner[1]);
    frame.render_widget(selectable_tabs, inner[3]);
    frame.render_widget(explanation_tabs, inner[4]);
    background::render_bottom_right(frame, inner[6]);
    frame.render_widget(component::social_footer_hyperlinks(inner[7].width, inner[7].height), inner[7]);
}
