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
                        } else if progress.contains("Finished") {
                            progress_receiver.close();
                            return (
                                Message::Command(command::Message::Finished),
                                ProgressState::Starting,
                            );
                        } else if progress.contains("entry does not pass filter (!playlist)") {
                            progress_receiver.close();
                            return (
                                Message::Command(command::Message::PlaylistNotChecked),
                                ProgressState::Starting,
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
        video_title: String,
        eta: i32,
        downloaded_bytes: f32,
        total_bytes: f32,
        elapsed: f32,
        speed: f32,
        playlist_count: Option<i32>,
        playlist_index: Option<i32>,
    },
    EndOfVideo,
    EndOfPlaylist,
    PostProcessing {
        status: String,
    },
}

pub fn parse_progress(input: String) -> Option<Progress> {
    for line in input.lines() {
        if let Some(progress) = line.strip_prefix("__") {
            let progress = progress.replace(r#""playlist_count": NA"#, r#""#);
            let progress = progress.replace(r#""playlist_index": NA"#, r#""#);
            let progress = progress.replace("NA", "0");

            return Some(
                serde_json::from_str::<Progress>(&progress).unwrap_or_else(|e| {
                    tracing::error!(
                        "failed to parse yt-dlp progress: {progress}::{input} -- {e:#?}"
                    );
                    panic!("failed to parse yt-dlp progress: {progress} -- {e:#?}");
                }),
            );
        }
    }

    None
}
