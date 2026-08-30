pub mod shared;
pub mod convert;
pub mod compress;
pub mod trim;
pub mod merge;

pub use shared::{FfmpegJob, FieldSet};

pub use convert::{ConvertField, ConvertState};
pub use compress::{CompressField, CompressState, crf_quality_label, crf_ratio};
pub use trim::{TrimField, TrimState};
pub use merge::{MergeField, MergeState};
