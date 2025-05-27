use std::path::PathBuf;

use crate::progress::ProgressError;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error(transparent)]
    Progress(ProgressError),
    #[error("invalid URL on position: {0}")]
    InvalidURL(usize),
    #[error(r#"Directory "{0}" does not exist, please create it then start the download"#)]
    DownloadDir(PathBuf),
    #[error("No Download URL was provided!")]
    NoDownloadURL,
    #[error("yt-dlp binary is missing")]
    YtDlpMissing,
    #[error("Something went wrong, logging...")]
    Other,
}
