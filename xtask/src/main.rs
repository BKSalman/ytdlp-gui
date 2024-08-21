use anyhow::{anyhow, Context};
use cargo_metadata::MetadataCommand;
use sha2::Digest;
use std::io::{BufRead, Write};
use std::str::FromStr;
use std::{fs::File, process::ExitCode};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

use xtask::{cargo, git, unzip, zip_dir, CommandExt};

#[derive(EnumString, EnumIter, Display, Debug, PartialEq)]
#[strum(serialize_all = "snake_case")]
enum Task {
    PackageRPM,
    PackageDEB,
    PackageLinux,
    PackageLinuxAll,
    PackageWindows,
    PackageAUR,
    PublishAUR,
}

fn main() -> ExitCode {
    let flags = xflags::parse_or_exit! {
        /// List all available tasks.
        optional -l,--list
        /// The task to run.
        optional task: String

        /// AUR package commit message.
        optional -m,--message message: String
        /// Specify rel for PKGBUILD (only works package_aur task).
        optional -r,--rel rel: u8
    };

    if !flags.list && flags.task.is_none() {
        eprintln!("No arguments were passed\nuse --help to see options");
        return ExitCode::FAILURE;
    }

    if let Some(task) = flags.task {
        let Ok(task) = Task::from_str(&task) else {
            eprintln!("Invalid task");
            return ExitCode::FAILURE;
        };

        if let Err(e) = std::fs::create_dir_all("packages") {
            eprintln!("Failed to create packages directory: {e}");
            return ExitCode::FAILURE;
        }

        let res = match task {
            Task::PackageWindows => package_windows(),
            Task::PackageRPM => package_rpm(),
            Task::PackageDEB => package_deb(),
            Task::PackageLinux => package_linux(),
            Task::PackageLinuxAll => package_linux_all(),
            Task::PackageAUR => package_aur(flags.rel),
            Task::PublishAUR => publish_aur(flags.message),
        };

        if let Err(e) = res {
            eprintln!("Failed to run task: {e}");
            return ExitCode::FAILURE;
        }
    } else if flags.list {
        println!("\nAvailable tasks:\n");

        Task::iter().for_each(|t| {
            println!("    {t}");
        });
    }

    ExitCode::SUCCESS
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

    let _ = std::fs::remove_dir_all("windows/ffmpeg-master-latest-win64-gpl");
    let _ = std::fs::remove_file("windows/ffmpeg-master-latest-win64-gpl.zip");

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
    zip_dir("windows", "packages/ytdlp-gui-windows-64.zip")?;

    Ok(())
}

fn package_aur(rel: Option<u8>) -> anyhow::Result<()> {
    let mut ytdlp_gui_path = std::env::current_dir()?;
    if ytdlp_gui_path.ends_with("xtask") {
        ytdlp_gui_path.pop();
    }
    let metadata = MetadataCommand::new()
        .manifest_path(ytdlp_gui_path.join("Cargo.toml"))
        .exec()
        .unwrap();

    let root_package = metadata.root_package().unwrap();

    let version = root_package.version.to_string();

    let aur_path = ytdlp_gui_path.join("aur");
    let pkgbuild_path = aur_path.join("PKGBUILD");

    let pkgbuild = std::fs::read_to_string(&pkgbuild_path)?;

    let pkgbuild = pkgbuild
        .lines()
        .try_fold(String::new(), |mut final_str, line| {
            if line.starts_with("pkgver=") {
                final_str.push_str(&format!("pkgver={}\n", version));
            } else if line.starts_with("sha256sums=") {
                let file_name = format!("v{version}.tar.gz");
                let source_code = minreq::get(format!(
                    "https://github.com/BKSalman/ytdlp-gui/archive/refs/tags/{}",
                    file_name
                ))
                .send()
                .context("failed to get source code from github")?
                .into_bytes();

                println!("downloaded source code of tag {file_name}");

                let sha = sha2::Sha256::digest(source_code);
                let hex = hex::encode(sha);
                println!("sha256 of source code: {hex}");

                final_str.push_str(&format!("sha256sums=(\"{}\")\n", hex));
            } else if line.starts_with("pkgrel") {
                if let Some(rel) = rel {
                    final_str.push_str(&format!("pkgrel={}\n", rel));
                } else {
                    final_str.push_str(&format!("{}\n", line));
                }
            } else {
                final_str.push_str(&format!("{}\n", line));
            }

            anyhow::Ok(final_str)
        })?;

    println!("PKGBULID:\n\n{pkgbuild}");

    println!("Do you want to proceed with printing to .SRCINFO? [Y/n]");

    let mut stdin = std::io::stdin().lock();

    let mut buf = String::new();

    stdin.read_line(&mut buf)?;

    if buf.to_lowercase() == "n\n" {
        return Ok(());
    }

    std::fs::write(&pkgbuild_path, pkgbuild)?;

    let old_current_dir = std::env::current_dir()?;

    std::env::set_current_dir(&aur_path)?;

    let srcinfo = std::process::Command::new("makepkg")
        .with_args([&pkgbuild_path.display().to_string(), "--printsrc"])
        .run_with_output("Printing to .SRCINFO\n")?;

    println!("srcinfo:\n\n{srcinfo}");

    std::env::set_current_dir(old_current_dir)?;

    let srcinfo_path = aur_path.join(".SRCINFO");

    std::fs::write(srcinfo_path, srcinfo)?;

    Ok(())
}

fn publish_aur(message: Option<String>) -> anyhow::Result<()> {
    let mut ytdlp_gui_path = std::env::current_dir()?;
    if ytdlp_gui_path.ends_with("xtask") {
        ytdlp_gui_path.pop();
    }

    let pkgbuild_path = ytdlp_gui_path.join("aur/PKGBUILD");
    let srcinfo_path = ytdlp_gui_path.join("aur/.SRCINFO");

    let pkgbuild = std::fs::read_to_string(&pkgbuild_path).context("failed to read PKGBUILD")?;

    println!("PKGBUILD:\n\n{pkgbuild}");

    println!("Do you want to proceed with publishing the package? [Y/n]");

    let mut stdin = std::io::stdin().lock();

    let mut buf = String::new();

    stdin.read_line(&mut buf)?;

    if buf.to_lowercase() == "n\n" {
        return Ok(());
    }

    let pkgname = pkgbuild
        .lines()
        .find(|l| l.starts_with("pkgname"))
        .map(|p| p.split_once('=').unwrap().1)
        .ok_or(anyhow!("no pkgname"))?;

    let pkgver = pkgbuild
        .lines()
        .find(|l| l.starts_with("pkgver"))
        .map(|p| p.split_once('=').unwrap().1)
        .ok_or(anyhow!("no pkgver"))?;

    let pkgrel = pkgbuild
        .lines()
        .find(|l| l.starts_with("pkgrel"))
        .map(|p| p.split_once('=').unwrap().1)
        .ok_or(anyhow!("no pkgrel"))?;

    std::env::set_current_dir(std::env::temp_dir())?;

    let temp_aur = std::env::temp_dir().join("ytdlp-gui-aur");

    let _ = std::fs::remove_dir_all(&temp_aur);

    let clone_output = git("clone")
        .with_args([
            "-v",
            &format!("ssh://aur@aur.archlinux.org/{pkgname}.git"),
            "ytdlp-gui-aur",
        ])
        .run_with_output("Clone AUR package")?;
    println!("git clone stdout:\n{}", clone_output);

    println!("Copying PKGBUILD and .SRCINFO to {}", temp_aur.display());
    std::fs::copy(pkgbuild_path, temp_aur.join("PKGBUILD")).context("failed to copy PKGBUILD")?;
    std::fs::copy(srcinfo_path, temp_aur.join(".SRCINFO")).context("failed to copy .SRCINFO")?;

    std::env::set_current_dir(temp_aur)?;

    let add_output = git("add")
        .with_args(["-v", "."])
        .run_with_output("Add AUR changes")?;
    println!("git add stdout:\n{}", add_output);

    let commit_output = git("commit")
        .with_args([
            "-v",
            "-m",
            &format!(
                "Update to {pkgver}-{pkgrel} {}",
                message.unwrap_or(String::new())
            ),
        ])
        .run_with_output("Commiting AUR changes")
        .context("failed to commit AUR changes")?;
    println!("git commit stdout:\n{}", commit_output);

    let push_output = git("push").run_with_output("Pushing to AUR")?;
    println!("git push stdout:\n{}", push_output);

    Ok(())
}
