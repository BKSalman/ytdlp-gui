use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::Path,
    process::Command,
};

use anyhow::{anyhow, Context};
use walkdir::WalkDir;
use zip::write::FileOptions;

pub trait CommandExt {
    fn with_arg(self, arg: &str) -> Self;
    fn with_args<I, S>(self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;
    fn run(self, msg: &str) -> anyhow::Result<()>;
    fn run_with_output(self, msg: &str) -> anyhow::Result<String>;
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

    fn run(mut self, msg: &str) -> anyhow::Result<()> {
        println!("{msg}");
        self.spawn()?.wait()?.check()
    }

    fn run_with_output(mut self, msg: &str) -> anyhow::Result<String> {
        println!("{msg}");
        let output = self.output()?;
        if let Err(e) = output.check().context("failed to run command") {
            println!("stderr:\n\t{}", String::from_utf8_lossy(&output.stderr));
            return Err(e);
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

pub trait CheckStatus {
    fn check(&self) -> anyhow::Result<()>;
}

impl CheckStatus for std::process::ExitStatus {
    fn check(&self) -> anyhow::Result<()> {
        match self.success() {
            true => Ok(()),
            false => Err(anyhow!(
                "Process exited with error code {}",
                self.code().unwrap_or(-1)
            )),
        }
    }
}

impl CheckStatus for std::process::Output {
    fn check(&self) -> anyhow::Result<()> {
        self.status.check()
    }
}

pub fn cargo(subcommand: &str) -> Command {
    Command::new("cargo").with_arg(subcommand)
}

pub fn git(subcommand: &str) -> Command {
    Command::new("git").with_arg(subcommand)
}

pub fn zip_dir(src_dir: &str, dst_file: &str) -> anyhow::Result<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(anyhow!("source dir not found"));
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter().filter_map(|e| e.ok());

    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

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

pub fn unzip(fname: &str, dst_prefix: &str) -> anyhow::Result<()> {
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

#[macro_export]
macro_rules! iter_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(#[$variant_meta:meta])*
            $($variant:ident),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $(#[$variant_meta])*
            $($variant),*
        }

        impl $name {
            const fn variants() -> &'static [Self] {
                &[$($name::$variant),*]
            }
        }
    };
}
