use shared_child::SharedChild;
use std::{
    io::{self, BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
};

use iced::futures::channel::mpsc::UnboundedSender;

#[cfg(target_os = "windows")]
use crate::CREATE_NO_WINDOW;

#[derive(Debug, Clone)]
pub enum Message {
    Run(String),
    Stop,
    Finished,
}

#[derive(Default)]
pub struct Command {
    pub shared_child: Option<Arc<SharedChild>>,
}

impl Command {
    // #[allow(clippy::too_many_arguments)]

    // fn view(&self) -> Element<Message> {
    //     ..
    // }

    pub fn kill(&self) -> io::Result<()> {
        if let Some(child) = &self.shared_child {
            return child.kill();
        }
        Ok(())
    }

    pub fn start(
        &mut self,
        mut args: Vec<&str>,
        show_modal: &mut bool,
        ui_message: &mut String,
        bin_dir: Option<PathBuf>,
        sender: Option<UnboundedSender<String>>,
    ) {
        let mut command = std::process::Command::new(bin_dir.unwrap_or_default().join("yt-dlp"));

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let print_before_dl = [
            "--print",
            r#"before_dl:__{"type": "pre_download", "video_id": "%(id)s"}"#,
        ];
        let progess_template = [
            "--progress-template",
            // format progress as a simple json
            r#"__{"type": "downloading", "video_title": "%(info.title)s", "eta": %(progress.eta)s, "downloaded_bytes": %(progress.downloaded_bytes)s, "total_bytes": %(progress.total_bytes)s, "elapsed": %(progress.elapsed)s, "speed": %(progress.speed)s, "percent_str": "%(progress._percent_str)s"}"#,
            "--progress-template",
            r#"postprocess:__{"type": "post_processing", "status": "%(progress.status)s"}"#,
        ];

        args.extend_from_slice(&print_before_dl);
        args.extend_from_slice(&progess_template);

        args.push("--progress");

        let Ok(shared_child) = SharedChild::spawn(
            command
                .args(args)
                .stderr(Stdio::piped())
                .stdout(Stdio::piped()),
        ) else {
            tracing::error!("Spawning child process failed");
            *show_modal = true;
            *ui_message = String::from("yt-dlp binary is missing");
            return;
        };

        self.shared_child = Some(Arc::new(shared_child));

        let Some(child) = self.shared_child.clone() else {
                        *show_modal = true;
                        *ui_message = String::from("Something went wrong");
                        tracing::error!("No child process");
                        return;
                    };

        *show_modal = true;
        *ui_message = String::from("Initializing...");

        if let Some(stderr) = child.take_stderr() {
            let Some(sender) = sender.clone() else {
                *show_modal = true;
                *ui_message = String::from("Something went wrong");
                return;
            };
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    sender
                        .unbounded_send(line)
                        .unwrap_or_else(|e| tracing::error!("{e}"));
                }
            });
        }

        if let Some(stdout) = child.take_stdout() {
            let Some(sender) = sender else {
                *show_modal = true;
                *ui_message = String::from("Something went wrong");
                return;
            };
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

                    match std::str::from_utf8(&buffer) {
                        Ok(str) => {
                            sender
                                .unbounded_send(str.to_string())
                                .unwrap_or_else(|e| tracing::debug!("{e}"));
                        }
                        Err(err) => {
                            tracing::debug!("{err}");
                        }
                    }
                    buffer.clear();
                }
                sender
                    .unbounded_send(String::from("Finished"))
                    .unwrap_or_else(|_e| tracing::error!("{_e}"));
            });
        }
    }
}
