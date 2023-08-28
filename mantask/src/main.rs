use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use anyhow::anyhow;
use walkdir::WalkDir;
use zip::write::FileOptions;

enum Task {
    PackageRPM,
    PackageDEB,
    PackageLinux,
    PackageLinuxAll,
    PackageWindows,
}

impl FromStr for Task {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "package_linux" => Ok(Self::PackageLinux),
            "package_rpm" => Ok(Self::PackageRPM),
            "package_deb" => Ok(Self::PackageDEB),
            "package_linux_all" => Ok(Self::PackageLinuxAll),
            "package_windows" => Ok(Self::PackageWindows),
            _ => Err(anyhow!("Invalid task")),
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::PackageRPM => writeln!(f, "package_rpm"),
            Task::PackageDEB => writeln!(f, "package_deb"),
            Task::PackageLinux => writeln!(f, "package_linux"),
            Task::PackageLinuxAll => writeln!(f, "package_linux_all"),
            Task::PackageWindows => writeln!(f, "package_windows"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let flags = xflags::parse_or_exit! {
        /// List all available tasks.
        optional -l,--list
        /// The task to run.
        optional task: String
    };

    if !flags.list && flags.task.is_none() {
        return Err(anyhow!(
            "No arguments were passed\nuse --help to see options"
        ));
    }

    if let Some(task) = flags.task {
        let task = Task::from_str(&task)?;

        std::fs::create_dir_all("packages")?;

        match task {
            Task::PackageWindows => package_windows()?,
            Task::PackageRPM => {
                package_rpm()?;
            }
            Task::PackageDEB => package_deb()?,
            Task::PackageLinux => {
                package_linux()?;
            }
            Task::PackageLinuxAll => package_linux_all()?,
        }
    } else if flags.list {
        println!("\nAvailable tasks:\n");

        [
            Task::PackageRPM,
            Task::PackageDEB,
            Task::PackageLinux,
            Task::PackageWindows,
            Task::PackageLinuxAll,
        ]
        .iter()
        .for_each(|t| {
            println!("    {t}");
        });
    }

    Ok(())
}

fn package_linux() -> anyhow::Result<()> {
    cargo("build")
        .with_arg("--release")
        .run("Building for linux")?;

    std::fs::copy(
        "target/release/ytdlp-gui",
        "packages/ytdlp-gui-linux-x64-86",
    )?;

    println!("Finished building for linux");

    Ok(())
}

fn package_rpm() -> anyhow::Result<()> {
    package_linux()?;

    cargo("install")
        .with_args(["--locked", "cargo-generate-rpm"])
        .run("Installing cargo-generate-rpm")?;

    cargo("generate-rpm").run("Generating RPM package")?;

    for entry in glob::glob("target/generate-rpm/*.rpm").expect("Failed to read glob pattern") {
        let entry = entry?;
        std::fs::rename(
            entry.to_string_lossy().to_string(),
            format!(
                "packages/{}",
                entry
                    .file_name()
                    .ok_or_else(|| anyhow!("No file name"))?
                    .to_string_lossy()
            ),
        )?;
    }

    println!("Finished generating RPM");
    Ok(())
}

fn package_deb() -> anyhow::Result<()> {
    package_linux()?;

    cargo("install")
        .with_arg("cargo-deb")
        .run("Installing cargo-deb")?;

    cargo("deb").run("Generating DEB package")?;

    std::fs::create_dir_all("packages")?;

    for entry in glob::glob("target/debian/*.deb").expect("Failed to read glob pattern") {
        let entry = entry?;
        std::fs::rename(
            entry.to_string_lossy().to_string(),
            format!(
                "packages/{}",
                entry
                    .file_name()
                    .ok_or_else(|| anyhow!("No file name"))?
                    .to_string_lossy()
            ),
        )?;
    }

    Ok(())
}

fn package_linux_all() -> anyhow::Result<()> {
    package_linux()?;
    package_rpm()?;
    package_deb()?;

    Ok(())
}

fn package_windows() -> anyhow::Result<()> {
    std::fs::create_dir_all("windows")?;
    std::fs::create_dir_all("packages")?;

    println!("Downloading ffmpeg");
    let ffmpeg_zip = minreq::get("https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip").send()?.into_bytes();

    File::create("windows/ffmpeg-master-latest-win64-gpl.zip")?.write_all(&ffmpeg_zip)?;

    println!("Unzipping ffmpeg");
    unzip("windows/ffmpeg-master-latest-win64-gpl.zip", "windows")?;

    std::fs::rename(
        "windows/ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe",
        "windows/ffmpeg.exe",
    )?;

    println!("Downloading ytdlp");
    let ytdlp = minreq::get("https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe")
        .send()?
        .into_bytes();

    File::create("windows/yt-dlp.exe")?.write_all(&ytdlp)?;
    println!("Saved yt-dlp to windows/yt-dlp.exe");

    #[cfg(target_os = "windows")]
    {
        cargo("build")
            .with_arg("--release")
            .run("Building for Windows")?;

        std::fs::rename("target/release/ytdlp-gui.exe", "windows/ytdlp-gui.exe")?;
    }

    #[cfg(unix)]
    {
        cargo("build")
            .with_args(["--release", "--target", "x86_64-pc-windows-gnu"])
            .run("Building for Windows")?;

        std::fs::rename(
            "target/x86_64-pc-windows-gnu/release/ytdlp-gui.exe",
            "windows/ytdlp-gui.exe",
        )?;
    }

    println!("Zipping windows package");
    zip_dir("windows", "ytdlp-gui-windows-64.zip")?;

    Ok(())
}

fn cargo(subcommand: &str) -> Command {
    Command::new("cargo").with_arg(subcommand)
}

trait CommandExt {
    fn with_arg(self, arg: &str) -> Self;
    fn with_args<I, S>(self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;
    fn run(self, msg: &str) -> std::io::Result<std::process::Child>;
}

impl CommandExt for Command {
    fn with_arg(mut self, arg: &str) -> Self {
        self.arg(arg);

        self
    }

    fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args(args);

        self
    }

    fn run(mut self, msg: &str) -> std::io::Result<std::process::Child> {
        println!("{msg}");
        self.spawn()
    }
}

fn zip_dir(src_dir: &str, dst_file: &str) -> anyhow::Result<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(anyhow!("source dir not found"));
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter().filter_map(|e| e.ok());

    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Zstd)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(src_dir)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;

    Ok(())
}

fn unzip(fname: &str, dst_prefix: &str) -> anyhow::Result<()> {
    let file = File::open(fname)?;
    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => Path::new(dst_prefix).to_path_buf().join(path),
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {i} comment: {comment}");
            }
        }

        if (*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = std::fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    Ok(())
}
