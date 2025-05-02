use crate::progress::ProgressError;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error(transparent)]
    Progress(ProgressError),
    #[error("invalid URL on position: {0}")]
    InvalidURL(usize),
    #[error("No Download URL was provided!")]
    NoDownloadURL,
    #[error("yt-dlp binary is missing")]
    YtDlpMissing,
    #[error("Something went wrong, logging...")]
    Other,
}
