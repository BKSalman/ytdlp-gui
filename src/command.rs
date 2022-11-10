use shared_child::SharedChild;
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use iced::futures::channel::mpsc::UnboundedSender;

use crate::video_options::Options;

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
    kill_child: Arc<AtomicBool>,
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
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        message: Message,
        show_modal: &mut bool,
        placeholder: &mut String,
        active_tab: usize,
        options: Options,
        is_playlist: bool,
        download_folder: &mut Option<PathBuf>,
        ui_message: &mut String,
        progress: &mut f32,
        progress_state: &mut ProgressState,
        sender: Option<UnboundedSender<String>>,
    ) {
        let mut args = Vec::new();

        self.kill_child = Arc::new(AtomicBool::new(false));
        match message {
            Message::Run(link) => {
                if link.is_empty() {
                    *placeholder = String::from("No Download link was provided!");
                    return;
                }

                *placeholder = String::from("Download link");

                args.push(link);

                match active_tab {
                    0 => {
                        let mut video = String::new();
                        args.push(String::from("-S"));

                        video.push_str(options.video_resolution.options());
                        
                        video.push_str(options.video_format.options());

                        args.push(video);
                    }
                    1 => {
                        // Audio tab
                        
                        // Extract audio from Youtube video
                        args.push(String::from("-x"));

                        // set audio format
                        args.push(String::from("--audio-format"));
                        args.push(options.audio_format.options());
                        
                        // set audio quality
                        args.push(String::from("--audio-quality"));
                        args.push(options.audio_quality.options());
                    }
                    _ => {}
                }

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
                    args.push(String::from("-o %(playlist)s/%(title)s.%(ext)s"))
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
                    args.push(String::from("%(title)s.%(ext)s"))
                }

                let mut command = std::process::Command::new("yt-dlp");

                #[cfg(target_os = "windows")]
                use std::os::windows::process::CommandExt;
                #[cfg(target_os = "windows")]
                command.creation_flags(CREATE_NO_WINDOW);

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
                if let Some(child) = self.shared_child.clone() {

                    *show_modal = true;

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
                    *ui_message = String::from("Initializing...");
                    if let Some(stdout) = child.take_stdout() {
                        let sender = Arc::new(Mutex::new(sender.expect("Sender")));
                        std::thread::spawn(move || {
                            let mut reader = BufReader::new(stdout);
                            let mut buffer: Vec<u8> = Vec::new();
                            loop {
                                if let Ok(bytes_read) = reader.read_until(b'\r', &mut buffer) {
                                    if bytes_read == 0 {
                                        break;
                                    }
                                    match std::str::from_utf8(&buffer) {
                                        Ok(str) => {
                                            (*sender.lock().expect("Sender lock"))
                                                .unbounded_send(str.to_string())
                                                .unwrap_or_else(|_e| {
                                                    #[cfg(debug_assertions)]
                                                    println!("{_e}")
                                                });
                                        }
                                        Err(err) => {
                                            println!("{err}");
                                        }
                                    }
                                    buffer.clear();
                                } else {
                                    panic!("failed to read buffer")
                                }
                            }
                            (*sender.lock().expect("Sender lock"))
                                .unbounded_send(String::from("Finished"))
                                .unwrap_or_else(|_e| {
                                    #[cfg(debug_assertions)]
                                    println!("{_e}")
                                });
                        });
                    }
                } else {
                    *show_modal = true;
                    *ui_message = String::from("yt-dlp binary is missing, add yt-dlp to your PATH and give it executable permissions `chmod +x yt-dlp`");
                }
            }
            Message::Stop => {
                match self.shared_child.clone().expect("Shared child").kill() {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        println!("killed the child, lmao")
                    }
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        println!("{_e}")
                    }
                };
                *show_modal = false;
                *progress_state = ProgressState::Hide;
                *progress = 0.;
                ui_message.clear();
            }
            Message::Finished => {
                match self.shared_child.clone().expect("Shared child").kill() {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        println!("killed the child, lmao")
                    }
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        println!("{_e}")
                    }
                };
                *progress = 0.;
                *progress_state = ProgressState::Hide;
                if ui_message.contains("Already") {
                    return;
                }
                *ui_message = String::from("Finished!");
            }
        }
    }

    // fn view(&self) -> Element<Message> {
    //     ..
    // }
}
