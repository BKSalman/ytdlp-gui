use shared_child::SharedChild;
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    sync::{atomic::AtomicBool, Arc, Mutex},
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

pub enum ProgressState {
    Show,
    Hide,
}

pub struct Command {
    pub kill_child: Arc<AtomicBool>,
    pub shared_child: Option<Arc<SharedChild>>,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            kill_child: Arc::new(AtomicBool::new(false)),
            shared_child: None,
        }
    }
}

impl Command {
    // #[allow(clippy::too_many_arguments)]

    // fn view(&self) -> Element<Message> {
    //     ..
    // }

    pub fn command(&mut self, args: Vec<String>, show_modal: bool, ui_message: String, sender: Option<UnboundedSender<String>>) {
        let mut command = std::process::Command::new("yt-dlp");

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        self.shared_child = match SharedChild::spawn(
            command
                .args(args)
                .stderr(Stdio::piped())
                .stdout(Stdio::piped()),
        ) {
            Ok(child) => Some(Arc::new(child)),
            Err(e) => {
                println!("{e}");
                None
            }
        };
        let Some(child) = self.shared_child.clone() else {
                        show_modal = true;
                        ui_message = String::from("yt-dlp binary is missing, add yt-dlp to your PATH and give it executable permissions `chmod +x yt-dlp`");
                        return;
                    };
        show_modal = true;

        if let Some(stderr) = child.take_stderr() {
            let sender = Arc::new(Mutex::new(sender.clone().expect("Sender clone")));
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    (*sender.lock().expect("Sender lock"))
                        .unbounded_send(line)
                        .unwrap_or_else(|_e| {
                            #[cfg(debug_assertions)]
                            println!("{_e}")
                        });
                }
            });
        }
        ui_message = String::from("Initializing...");
        if let Some(stdout) = child.take_stdout() {
            let sender = Arc::new(Mutex::new(sender.expect("Sender")));
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut buffer: Vec<u8> = Vec::new();
                loop {
                    let Ok(bytes_read) = reader.read_until(b'\r', &mut buffer) else {
                                        panic!("failed to read buffer");
                                };

                    if bytes_read == 0 {
                        break;
                    }

                    match std::str::from_utf8(&buffer) {
                        Ok(str) => {
                            (*sender.lock().expect("Sender lock"))
                                .unbounded_send(str.to_string())
                                .unwrap_or_else(|_e| {
                                    #[cfg(debug_assertions)]
                                    eprintln!("{_e}")
                                });
                        }
                        Err(err) => {
                            #[cfg(debug_assertions)]
                            eprintln!("{err}");
                        }
                    }
                    buffer.clear();
                }
                (*sender.lock().expect("Sender lock"))
                    .unbounded_send(String::from("Finished"))
                    .unwrap_or_else(|_e| {
                        #[cfg(debug_assertions)]
                        eprintln!("{_e}")
                    });
            });
        }
    }
}

pub fn playlist_options(is_playlist: bool, download_folder: Option<PathBuf>) -> Vec<String> {
    let mut args = Vec::new();
    if is_playlist {
        args.push(String::from("--yes-playlist"));
        args.push(String::from("-P"));
        args.push(
            download_folder
                .clone()
                .expect("No Videos Directory")
                .to_str()
                .expect("No Videos Directory")
                .to_string(),
        );
        args.push(String::from("-o %(playlist)s/%(title)s.%(ext)s"));
    } else {
        args.push(String::from("--break-on-reject"));
        args.push(String::from("--match-filter"));
        args.push(String::from("!playlist"));
        args.push(String::from("--no-playlist"));
        args.push(String::from("-P"));
        args.push(
            download_folder
                .clone()
                .expect("No Videos Directory")
                .to_str()
                .expect("No Videos Directory")
                .to_string(),
        );
        args.push(String::from("-o"));
        args.push(String::from("%(title)s.%(ext)s"));
    }

    args
}
