pub mod convert_explanation;
pub mod compress_explanation;
pub mod trim_explanation;
pub mod merge_explanation;

pub use convert_explanation::{
    explain_intro_line_1 as explain_convert_intro_line_1,
    explain_intro_line_2 as explain_convert_intro_line_2,
    explain_outro_line as explain_convert_outro_line,
    explain_format_list as explain_convert_format_list,
    explain_codec_list as explain_convert_codec_list,
};
pub use compress_explanation::{
    explain_intro_line_1 as explain_compress_intro_line_1,
    explain_intro_line_2 as explain_compress_intro_line_2,
    explain_outro_line as explain_compress_outro_line,
    explain_format_list as explain_compress_format_list,
};
pub use trim_explanation::{
    explain_intro_line_1 as explain_trim_intro_line_1,
    explain_intro_line_2 as explain_trim_intro_line_2,
    explain_outro_line as explain_trim_outro_line,
    explain_format_list as explain_trim_format_list,
};
pub use merge_explanation::{
    explain_intro_line_1 as explain_merge_intro_line_1,
    explain_intro_line_2 as explain_merge_intro_line_2,
    explain_outro_line as explain_merge_outro_line,
    explain_format_list as explain_merge_format_list,
};
