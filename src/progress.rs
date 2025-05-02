use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Progress {
    PreProcessing,
    PreDownload {
        video_id: String,
    },
    Downloading {
        eta: Option<f64>,
        downloaded_bytes: f32,
        total_bytes: Option<f32>,
        total_bytes_estimate: Option<f32>,
        elapsed: f32,
        speed: Option<f32>,
        playlist_count: Option<i32>,
        playlist_index: Option<i32>,
    },
    EndOfVideo,
    EndOfPlaylist,
    PostProcessing {
        status: String,
    },
    Error(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProgressError {
    #[error("File already exists")]
    AlreadyExists,
    #[error("Playlist checkbox not checked")]
    PlaylistNotChecked,
    #[error("Private video, skipping...")]
    PrivateVideo,
    #[error("Video unavailable, skipping...")]
    VideoUnavailable,
    #[error("Playlist does not exist")]
    NoPlaylist,
    #[error("{0}")]
    Other(String),
}

pub fn parse_progress(progress: &str) -> Result<Vec<Progress>, ProgressError> {
    if progress.contains("has already been downloaded") {
        return Err(ProgressError::AlreadyExists);
    } else if progress.contains("entry does not pass filter (!playlist)") {
        return Err(ProgressError::PlaylistNotChecked);
    } else if progress.contains("Private video. Sign in if you've been granted access to this video") {
        return Err(ProgressError::PrivateVideo);
    } else if progress.contains("Video unavailable. This video contains content") ||
        progress.contains("Video unavailable. This video is no longer available because the YouTube account associated with this video has been terminated.") {
        return Err(ProgressError::VideoUnavailable);
    } else if progress.contains("YouTube said: The playlist does not exist.") {
        return Err(ProgressError::NoPlaylist);
    } else if let Some(error) = progress.strip_prefix("stderr:ERROR: ") {
        return Err(ProgressError::Other(error.to_string()));
    }

    tracing::debug!("received progress from yt-dlp: {progress}");

    let mut progresses = Vec::new();

    for line in progress.lines() {
        if line.starts_with("__") {
            for object in line.split("__") {
                let progress = object.replace("NA", "null");

                if let Ok(progress) = serde_json::from_str::<Progress>(&progress) {
                    progresses.push(progress);
                }
            }
        }
    }

    Ok(progresses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_progress() {
        let progress = r#"__{"type": "downloading","eta": 10, "downloaded_bytes": 62444041,"total_bytes": 198896641, "total_bytes_estimate": NA,"elapsed": 3.448781967163086, "speed": 12773016.258777222, "playlist_count": NA,"playlist_index": NA }"#;
        let parsed_progress = parse_progress(progress).unwrap();

        assert_eq!(
            parsed_progress,
            vec![Progress::Downloading {
                eta: Some(10.),
                downloaded_bytes: 62444041.,
                total_bytes: Some(198896641.),
                total_bytes_estimate: None,
                elapsed: 3.448781967163086,
                speed: Some(12773016.258777222),
                playlist_count: None,
                playlist_index: None
            }]
        );
    }
}
