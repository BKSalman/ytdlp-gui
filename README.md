# ytdlp-gui
a GUI for yt-dlp written in Rust

- [Installation](https://github.com/BKSalman/ytdlp-gui#installation)
  - [NixOS (Flake)](https://github.com/BKSalman/ytdlp-gui#nixos-flake)
  - [Fedora](https://github.com/BKSalman/ytdlp-gui#fedora)
  - [Ubuntu](https://github.com/BKSalman/ytdlp-gui#ubuntu)
  - [other distributions](https://github.com/BKSalman/ytdlp-gui#other-distributions)
  - [Windows](https://github.com/BKSalman/ytdlp-gui#windows)
- [Build from source](https://github.com/BKSalman/ytdlp-gui#build-from-source)
- [Configuration](https://github.com/BKSalman/ytdlp-gui#configuration)
- [Contribution](https://github.com/BKSalman/ytdlp-gui#contribution)

# Preview
![2023-07-29_16-52_1](https://github.com/BKSalman/ytdlp-gui/assets/85521119/0703580a-0662-4aad-864e-d3f402d5d3c3)
![2023-07-29_16-49](https://github.com/BKSalman/ytdlp-gui/assets/85521119/d6e87147-f65c-4b74-ae43-14a6d4b6c1be)
![2023-07-29_16-52](https://github.com/BKSalman/ytdlp-gui/assets/85521119/832d8ade-5a8a-4876-9a3d-34a43f8574b9)


# Installation
## Linux

### NixOS (Flake)
you can use the flake.nix in the repo

in your `flake.nix`:
```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    ytdlp-gui = {
      url = "github:bksalman/ytdlp-gui";
    };
  };

    outputs = { nixpkgs, ytdlp-gui, ...}:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          ytdlp-gui.overlay
        ];
      };
    in
    {
      ...snip
```

then you can add it as a normal package, either to your home-manager or nixosConfiguration

### Fedora
download the rpm package from the releases page then install it with ``sudo dnf localinstall <rpm_package_name>``

### Ubuntu
download the deb package from the releases page then install it with ``sudo apt install ./<deb_package_name>``

### Arch

Available in the AUR [ytdlp-gui](https://aur.archlinux.org/packages/ytdlp-gui)

### other distributions

#### 1- download ``yt-dlp``
either

&nbsp; &nbsp; &nbsp; a- from your distribution repo

&nbsp; &nbsp; &nbsp; b- or download the [binary](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp), then move it to your bin directory, and make it an executable by running `chmod +x <bin_file>`

#### 2- download ``ffmpeg`` or ``ffmpeg-free`` from your distribution repos


#### 3- download the ``ytdlp-gui`` binary from the [realeases page](https://github.com/BKSalman/ytdlp-gui/releases)

## Windows
##### just download the zip file from the releases page, extract it in a subfolder and start the ``ytdlp-gui.exe``

# Build from source
to build from source you need to have `cargo` and `rustc`, you can install them through `rustup` (rust toolchain manager), or from your distribution repos, whatever you like

after that run the following commands:
```bash
# clone the repository to "ytdlp-gui" folder
git clone https://github.com/BKSalman/ytdlp-gui
# enter the folder
cd ytdlp-gui
# you can either build the project using this
cargo build
# or build it in release mode for better performance
cargo build -r
```
then the binary will be either in `<project-root>/target/debug/ytdlp-gui` or `<project-root>/target/release/ytdlp-gui`

and you can either run it directly:
```bash
# from project root
./target/release/ytdlp-gui
```

or using cargo:
```bash
cargo r
# or for release mode
cargo r -r
```

# Configuration

For v0.2.2+ the application saves configuration in the default config directory for the respective platform/OS in ``<config_dir>/ytdlp-gui/config.toml``

the default file looks like this:

```toml
# Optional
# This is the directory of the bin, not the bin itself
# bin_path = "<some_cool_path>" # (0.2.4)

bin_dir = "<some_cool_path>" # (0.2.5+) if not set the command will be `yt-dlp <app_args>`

# Optional
download_folder = "<some_cool_path>" # default = "~/Videos"

[options]
video_resolution = "FullHD" # options: "Sd" "Hd" "FullHD" "TwoK" "FourK"
video_format = "Mp4" # options: "Mp4" "Mkv" "Webm"
audio_quality = "Good" # options: "Best" "Good" "Medium" "Low"
audio_format = "Mp3" #  options: "Mp3" "Wav" "Vorbis" "M4a" "Opus"
```

### Note: the quality/format options get automatically saved when pressing the download button

# Contribution
All contribution forms are welcomed, whether it's Pull requests, Issues (bug reports/enhancement requests)

However, I might not be quick to reply to them, or implement the requested stuff, since I'm focusing on other things

But I will do my best 👍
