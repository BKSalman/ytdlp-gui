use crate::DownloadError;
use iced::widget::{button, column, horizontal_space, progress_bar, row, text};
use iced::Length;

use serde::{Deserialize, Serialize};

use crate::{app::SPACING, Message, YtGUI};

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

impl YtGUI {
    pub fn show_download_progress<'a>(
        &'a self,
        download_message: &'a Result<String, DownloadError>,
    ) -> iced::widget::Column<'a, Message> {
        match download_message {
            Ok(download_message) => column![
                row![
                    text(download_message).align_x(iced::alignment::Horizontal::Center),
                    horizontal_space(),
                    text(self.playlist_progress.as_deref().unwrap_or_default()),
                    button("X").on_press(Message::StopDownload).padding([5, 25]),
                ]
                .spacing(SPACING)
                .width(iced::Length::Fill)
                .align_y(iced::Alignment::Center)
                .padding(12),
                if let Some(progress) = self.progress {
                    row![progress_bar(0.0..=100., progress)]
                        .spacing(SPACING)
                        .width(iced::Length::Fill)
                        .align_y(iced::Alignment::Center)
                        .padding(12)
                } else {
                    row![]
                }
            ]
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
            Err(e) => {
                column![
                    row![text(e.to_string()).align_x(iced::alignment::Horizontal::Center)]
                        .spacing(SPACING)
                        .width(iced::Length::Fill)
                        .align_y(iced::Alignment::Center)
                        .padding(12),
                ]
            }
        }
    }

    pub fn handle_progress_event(&mut self, progress: &str) {
        if !self.command.is_running() {
            return;
        }

        match parse_progress(progress) {
            Ok(progress) => {
                for progress in progress {
                    match progress {
                        Progress::Downloading {
                            eta,
                            downloaded_bytes,
                            total_bytes,
                            total_bytes_estimate,
                            elapsed: _,
                            speed,
                            playlist_count,
                            playlist_index,
                        } => {
                            self.progress = Some(
                                (downloaded_bytes
                                    / total_bytes.unwrap_or(total_bytes_estimate.unwrap_or(0.)))
                                    * 100.,
                            );

                            if let Some((playlist_count, playlist_index)) =
                                playlist_count.zip(playlist_index)
                            {
                                self.playlist_progress = Some(format!(
                                    "Downloading {}/{}",
                                    playlist_index, playlist_count
                                ));
                            }

                            // `eta as i64` rounds it
                            // for examlpe: 12.368520936129604 as i64 = 12
                            let eta = chrono::Duration::seconds(eta.unwrap_or(0.) as i64);

                            let downloaded_megabytes = downloaded_bytes / 1024_f32.powi(2);
                            let total_downloaded = if downloaded_megabytes > 1024. {
                                format!("{:.2}GB", downloaded_megabytes / 1024.)
                            } else {
                                format!("{:.2}MB", downloaded_megabytes)
                            };

                            self.download_message = Some(Ok(format!(
                                                        "{total_downloaded} | {speed:.2}MB/s | ETA {eta_mins:02}:{eta_secs:02}",
                                                        speed = speed.unwrap_or(0.) / 1024_f32.powi(2),
                                                        eta_mins = eta.num_minutes(),
                                                        eta_secs = eta.num_seconds() - (eta.num_minutes() * 60),
                                                    )));
                        }
                        Progress::PostProcessing { status: _ } => {
                            self.download_message = Some(Ok(String::from("Processing...")));
                        }
                        Progress::EndOfPlaylist => {
                            tracing::info!("end of playlist");
                            self.end_download(Some(Ok(String::from("Finished playlist!"))));
                        }
                        Progress::EndOfVideo => {
                            if !self.is_playlist {
                                if self.command.is_multiple_videos() {
                                    self.command.finished_single_video();
                                } else {
                                    self.end_download(Some(Ok(String::from("Finished!"))));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                self.end_download(Some(Err(DownloadError::Progress(e))));
            }
        }
    }
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
