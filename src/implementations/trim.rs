use ratatui_textarea::TextArea;

use super::shared::{default_output_path, new_path_field, spawn_ffmpeg_job, FfmpegJob, FieldSet, MenuState};

//                      <-- TRIM FIELDS (cut a range, no re-encode) -->

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimField { InputPath, OutputPath, Start, End, Run }

impl FieldSet for TrimField {
    const ALL: &'static [TrimField] = &[
        TrimField::InputPath, TrimField::OutputPath, TrimField::Start, TrimField::End, TrimField::Run,
    ];

    fn label(&self) -> &'static str {
        match self {
            Self::InputPath => "📂 input", Self::OutputPath => "💾 output",
            Self::Start => "⏱ start", Self::End => "⏱ end", Self::Run => "▶ trim",
        }
    }
}

pub type TrimMenuState = MenuState<TrimField>;

//                      <-- TRIM SCREEN STATE (cut a range, no re-encode) -->
pub struct TrimState {
    pub input_file_path: TextArea<'static>,
    pub output_file_path: TextArea<'static>,
    pub start_time: TextArea<'static>,
    pub end_time: TextArea<'static>,
    pub menu: TrimMenuState,
    job: Option<FfmpegJob>,
}

impl TrimState {
    pub fn new() -> Self {
        Self {
            input_file_path: new_path_field("C:/Users/you/video.mov"),
            output_file_path: new_path_field("(optional) C:/Users/you/output.mov"),
            start_time: new_path_field("00:00:00"),
            end_time: new_path_field("00:00:10"),
            menu: TrimMenuState::new(),
            job: None,
        }
    }

    pub fn job(&self) -> Option<&FfmpegJob> {
        self.job.as_ref()
    }

    pub fn poll_job(&mut self) {
        if let Some(job) = self.job.as_mut() {
            job.poll();
        }
    }

    /// Trim only ever cuts a time range: stream-copied, no re-encode, no format/quality change.
    pub fn start_trim(&mut self, log_path: &str) {
        let input = super::shared::strip_quotes(&self.input_file_path.lines().join(""));
        let start = self.start_time.lines().join("");
        let end = self.end_time.lines().join("");
        if input.trim().is_empty() || start.trim().is_empty() || end.trim().is_empty() {
            return;
        }
        let typed_output = super::shared::strip_quotes(&self.output_file_path.lines().join(""));
        let output = if typed_output.trim().is_empty() {
            default_output_path(&input, "trimmed")
        } else {
            typed_output
        };

        // -ss/-to before -i: fast seek on the input side, -c copy skips re-encoding entirely.
        let args: Vec<String> = vec![
            "-y".into(), "-ss".into(), start, "-to".into(), end,
            "-i".into(), input, "-c".into(), "copy".into(), output,
        ];

        self.job = Some(spawn_ffmpeg_job(args, log_path, "trim"));
    }
}
