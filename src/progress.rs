use iced::futures::{channel::mpsc, StreamExt};
use iced::{subscription, Subscription};
use serde::{Deserialize, Serialize};

use crate::{command, Message};

pub enum ProgressState {
    Starting,
    Ready(mpsc::UnboundedReceiver<String>),
}

pub fn bind() -> Subscription<Message> {
    struct Progress;

    subscription::unfold(
        std::any::TypeId::of::<Progress>(),
        ProgressState::Starting,
        |state| async move {
            match state {
                ProgressState::Starting => {
                    let (sender, receiver) = mpsc::unbounded();

                    (Message::Ready(sender), ProgressState::Ready(receiver))
                }
                ProgressState::Ready(mut progress_receiver) => {
                    let received = progress_receiver.next().await;
                    if let Some(progress) = received {
                        tracing::debug!("received progress from yt-dlp: {progress}");
                        if progress.contains("has already been downloaded") {
                            progress_receiver.close();
                            return (
                                Message::Command(command::Message::AlreadyExists),
                                ProgressState::Starting,
                            );
                        } else if progress.contains("entry does not pass filter (!playlist)") {
                            progress_receiver.close();
                            return (
                                Message::Command(command::Message::PlaylistNotChecked),
                                ProgressState::Starting,
                            );
                        } else if let Some(progress) = progress.strip_prefix("stderr:ERROR") {
                            return (
                                Message::Command(command::Message::Error(progress.to_string())),
                                ProgressState::Ready(progress_receiver),
                            );
                        } else {
                            return (
                                Message::ProgressEvent(progress),
                                ProgressState::Ready(progress_receiver),
                            );
                        }
                    }

                    (Message::None, ProgressState::Ready(progress_receiver))
                }
            }
        },
    )
}

#[derive(Debug, Deserialize, Serialize)]
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

pub fn parse_progress(input: String) -> Vec<Progress> {
    input.lines().filter_map(|line| {
        if let Some(progress) = line.strip_prefix("__") {
            let progress = progress.replace("NA", "null");

            Some(
                serde_json::from_str::<Progress>(&progress).unwrap_or_else(|e| {
                    tracing::error!(
                        "failed to parse yt-dlp progress: \noriginal-input: {input}\nstripped-input: {progress}\n{e:#?}"
                    );
                    panic!("failed to parse yt-dlp progress");
                }),
            )
        } else {
            None
        }
    }).collect::<Vec<Progress>>()
}
