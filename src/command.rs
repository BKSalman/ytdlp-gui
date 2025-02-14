use shared_child::SharedChild;
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
};

use iced::futures::channel::mpsc::UnboundedSender;

use crate::error::DownloadError;
#[cfg(target_os = "windows")]
use crate::CREATE_NO_WINDOW;

#[derive(Default)]
pub struct Command {
    pub shared_child: Option<Arc<SharedChild>>,
    videos_num: usize,
}

impl Command {
    // #[allow(clippy::too_many_arguments)]

    // fn view(&self) -> Element<Message> {
    //     ..
    // }

    pub fn is_multiple_videos(&self) -> bool {
        self.videos_num > 1
    }

    pub fn finished_single_video(&mut self) {
        self.videos_num -= 1;
    }

    pub fn kill(&mut self) {
        if let Some(child) = self.shared_child.take() {
            match child.kill() {
                Ok(_) => {
                    tracing::debug!("killed child process");
                }
                Err(e) => {
                    tracing::error!("failed to kill child process {e}");
                }
            };
            match child.wait() {
                Ok(status) => {
                    tracing::debug!("waited for child process: {status}");
                }
                Err(e) => {
                    tracing::error!("failed to to wait for child process {e}");
                }
            };
        }
    }

    pub fn start(
        &mut self,
        mut args: Vec<&str>,
        bin_dir: Option<PathBuf>,
        sender: UnboundedSender<crate::Message>,
        videos_num: usize,
    ) -> Option<Result<String, DownloadError>> {
        let _ = self.kill();

        self.videos_num = videos_num;

        let mut command = std::process::Command::new(bin_dir.unwrap_or_default().join("yt-dlp"));

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let print = [
            "--print",
            r#"before_dl:__{"type": "pre_download", "video_id": "%(id)s"}"#,
            "--print",
            r#"playlist:__{"type": "end_of_playlist"}"#,
            "--print",
            r#"after_video:__{"type": "end_of_video"}"#,
        ];

        // reference: https://github.com/yt-dlp/yt-dlp/blob/351dc0bc334c4e1b5f00c152818c3ec0ed71f788/yt_dlp/YoutubeDL.py#L364
        // NOTE: sometimes some fields are missing like:
        // - total_bytes: Size of the whole file, None if unknown
        // - total_bytes_estimate: Guess of the eventual file size, None if unavailable.
        // - eta: The estimated time in seconds, None if unknown
        // - speed: The download speed in bytes/second, None if unknown
        // we need to compensate for it as much as possible (sometimes we can't)

        // format progress as a simple json
        let template = concat!(
            r#"__{"type": "downloading","#,
            r#""eta": %(progress.eta)s, "downloaded_bytes": %(progress.downloaded_bytes)s,"#,
            r#""total_bytes": %(progress.total_bytes)s, "total_bytes_estimate": %(progress.total_bytes_estimate)s,"#,
            r#""elapsed": %(progress.elapsed)s, "speed": %(progress.speed)s, "playlist_count": %(info.playlist_count)s,"#,
            r#""playlist_index": %(info.playlist_index)s }"#
        );

        let progess_template = [
            "--progress-template",
            template,
            // "--progress-template",
            // r#"postprocess:__{"type": "post_processing", "status": "%(progress.status)s"}"#,
        ];

        args.extend_from_slice(&print);
        args.extend_from_slice(&progess_template);

        args.push("--no-quiet");

        let Ok(shared_child) = SharedChild::spawn(
            command
                .args(args)
                .stderr(Stdio::piped())
                .stdout(Stdio::piped()),
        ) else {
            tracing::error!("Spawning child process failed");
            return Some(Err(DownloadError::YtDlpMissing));
        };

        self.shared_child = Some(Arc::new(shared_child));

        let Some(child) = self.shared_child.clone() else {
            tracing::error!("No child process");
            return Some(Err(DownloadError::Other));
        };

        if let Some(stderr) = child.take_stderr() {
            let sender = sender.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    sender
                        .unbounded_send(crate::Message::ProgressEvent(format!("stderr:{line}")))
                        .unwrap_or_else(|e| tracing::error!("failed to send stderr: {e}"));
                }
            });
        }

        if let Some(stdout) = child.take_stdout() {
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut buffer = vec![];
                loop {
                    let Ok(bytes_read) = reader.read_until(b'\r', &mut buffer) else {
                        panic!("failed to read buffer");
                    };

                    if bytes_read == 0 {
                        break;
                    }

                    sender
                        .unbounded_send(crate::Message::ProgressEvent(
                            String::from_utf8_lossy(&buffer).to_string(),
                        ))
                        .unwrap_or_else(|e| tracing::error!("failed to send stdout: {e}"));

                    buffer.clear();
                }
                // sender
                //     .unbounded_send(String::from("Finished"))
                //     .unwrap_or_else(|e| tracing::error!(r#"failed to send "Finished": {e}"#));
            });
        }

        Some(Ok(String::from("Initializing...")))
    }

    pub fn is_running(&self) -> bool {
        self.shared_child.is_some()
    }
}
