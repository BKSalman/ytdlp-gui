use shared_child::SharedChild;
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Stdio,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use iced::futures::channel::mpsc::UnboundedSender;
use iced_aw::modal;

use crate::{AudioFormat, AudioQuality, ModalState, Resolution, VideoFormat};

#[cfg(target_os = "windows")]
use crate::CREATE_NO_WINDOW;

#[derive(Debug, Clone)]
pub enum Message {
    Run(String),
    Stop,
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
        modal_state: &mut modal::State<ModalState>,
        placeholder: &mut String,
        active_tab: usize,
        resolution: Resolution,
        video_format: VideoFormat,
        audio_format: AudioFormat,
        audio_quality: AudioQuality,
        is_playlist: bool,
        download_folder: &mut Option<PathBuf>,
        output: &mut String,
        progress: &mut f32,
        sender: Option<UnboundedSender<String>>,
    ) {
        let mut args = Vec::new();

        self.kill_child = Arc::new(AtomicBool::new(false));
        match message {
            Message::Run(link) => {
                if link.is_empty() {
                    *placeholder = "No Download link was provided!".to_string();
                    return;
                }

                *placeholder = "Download link".to_string();

                args.push(link);

                match active_tab {
                    0 => {
                        let mut video = String::new();
                        args.push("-S".to_string());

                        match resolution {
                            Resolution::FourK => {
                                video.push_str("res:2160,");
                            }
                            Resolution::TwoK => {
                                video.push_str("res:1440,");
                            }
                            Resolution::FullHD => {
                                video.push_str("res:1080,");
                            }
                            Resolution::Hd => {
                                video.push_str("res:720,");
                            }
                            Resolution::Sd => {
                                video.push_str("res:480,");
                            }
                        }

                        match video_format {
                            VideoFormat::Mp4 => {
                                video.push_str("ext:mp4");
                            }
                            VideoFormat::ThreeGP => {
                                video.push_str("ext:3gp");
                            }
                            VideoFormat::Webm => {
                                video.push_str("ext:webm");
                            }
                        }
                        args.push(video);
                    }
                    1 => {
                        // Audio tab
                        args.push("-x".to_string());
                        args.push("--audio-format".to_string());
                        match audio_format {
                            AudioFormat::Mp3 => {
                                args.push("mp3".to_string());
                            }
                            AudioFormat::Wav => {
                                args.push("wav".to_string());
                            }
                            AudioFormat::Vorbis => {
                                args.push("vorbis".to_string());
                            }
                            AudioFormat::Opus => {
                                args.push("opus".to_string());
                            }
                            AudioFormat::M4a => {
                                args.push("m4a".to_string());
                            }
                        }

                        args.push("--audio-quality".to_string());
                        match audio_quality {
                            AudioQuality::Best => {
                                args.push("0".to_string());
                            }
                            AudioQuality::Good => {
                                args.push("2".to_string());
                            }
                            AudioQuality::Medium => {
                                args.push("4".to_string());
                            }
                            AudioQuality::Low => {
                                args.push("6".to_string());
                            }
                        }
                    }
                    _ => {}
                }

                if is_playlist {
                    args.push("--yes-playlist".to_string());
                    args.push("-P".to_string());
                    args.push(
                        download_folder
                            .clone()
                            .unwrap()
                            .to_str()
                            .expect("No Videos Directory")
                            .to_string(),
                    );
                    args.push("-o %(playlist)s/%(title)s.%(ext)s".to_string())
                } else {
                    args.push("--no-playlist".to_string());
                    args.push("-P".to_string());
                    args.push(
                        download_folder
                            .clone()
                            .unwrap()
                            .to_str()
                            .expect("No Videos Directory")
                            .to_string(),
                    );
                    args.push("-o".to_string());
                    args.push("%(title)s.%(ext)s".to_string())
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
                    modal_state.show(true);
                    if let Some(stderr) = child.take_stderr() {
                        let sender = Arc::new(Mutex::new(sender.clone().unwrap()));
                        std::thread::spawn(move || {
                            let reader = BufReader::new(stderr);
                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    (*sender.lock().unwrap())
                                        .unbounded_send(line)
                                        .unwrap_or_else(|_e| {
                                            #[cfg(debug_assertions)]
                                            println!("{_e}")
                                        });
                                }
                            }
                        });
                    }
                    if let Some(stdout) = child.take_stdout() {
                        let sender = Arc::new(Mutex::new(sender.unwrap()));
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
                                            (*sender.lock().unwrap())
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
                            (*sender.lock().unwrap())
                                .unbounded_send("Finished".to_string())
                                .unwrap_or_else(|_e| {
                                    #[cfg(debug_assertions)]
                                    println!("{_e}")
                                });
                        });
                    }
                } else {
                    modal_state.show(true);
                    *output = "yt-dlp binary is missing, add yt-dlp to your PATH and give it executable permissions `chmod +x yt-dlp`".to_string();
                }
            }
            Message::Stop => {
                match self.shared_child.clone().unwrap().kill() {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        println!("killed the child, lmao")
                    }
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        println!("{_e}")
                    }
                };
                modal_state.show(false);
                *progress = 0.;
                output.clear();
            }
        }
    }

    // fn view(&self) -> Element<Message> {
    //     ..
    // }
}
