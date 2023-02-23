# ytdlp-gui
a GUI for yt-dlp written in Rust

# Preview
![image](https://user-images.githubusercontent.com/85521119/197349230-dac48be5-d855-4d06-aa7e-d3a762b9efeb.png)
![image](https://user-images.githubusercontent.com/85521119/197349217-9be1988a-e869-4f85-bc1e-0b4b79253c14.png)

# Installation
## Linux

### Fedora
download the rpm package from the releases page then install it with ``sudo dnf localinstall <rpm_package_name>``

### Ubuntu
download the deb package from the releases page then install it with ``sudo apt install ./<deb_package_name>``

### Arch

Available in the AUR [ytdlp-gui](https://aur.archlinux.org/packages/ytdlp-gui)

### other distributions

#### 1- download ``yt-dlp``

a- from your distribution repo

b- or download the [binary](https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp) then move it to your bin directory and make it an executable

#### 2- download ``ffmpeg`` or ``ffmpeg-free`` from your distribution repos


#### 3- download the ``ytdlp-gui`` binary from the [realeases page](https://github.com/BKSalman/ytdlp-gui/releases)

## Windows
##### just download the zip file from the releases page, extract it in a subfolder and start the ``ytdlp-gui.exe``

# Configuration

For v0.2.2+ the application saves configuration in the default config directory for the respective platform/OS in ``<config_dir>/ytdlp-gui/config.toml``

the default file looks like this:

```toml
# Optional
# This is the directory of the bin, not the bin itself
bin_path = "<some_cool_path>" # if not set the command will be `yt-dlp <app_args>`

# Optional
download_folder = "<some_cool_path>" # default = "~/Videos"

[options]
video_resolution = "FullHD" # options: "Sd" "Hd" "FullHD" "TwoK" "FourK"
video_format = "Mp4" # options: "Mp4" "Mkv" "Webm"
audio_quality = "Good" # options: "Best" "Good" "Medium" "Low"
audio_format = "Mp3" #  options: "Mp3" "Wav" "Vorbis" "M4a" "Opus"
```
