# Changelog

## 3.1.2
- Minor: change missleading text in download path text input
- Minor: use proper config defaults when config file is not present

## 3.1.1
- Dev: fix CI

## 3.1.0
- Minor: Add the base for localization using [Fluent](https://projectfluent.org/) (contributions are very much welcome!!)
- Minor: Add a notification for downloads (Thanks to @iKetsui! @62)
- Minor: Add X button to errors
- Minor: expand tilde `~` when used in the download path text input

## 3.0.1
- Minor: Prevent ytdlp-gui from creating a new directory if it doesn't exist
- Minor: Add label to file path textbox (Thanks to @YourUnravelling! #59)
- Bugfix: fix the issue where the config file won't load properly (and fallback to the default config) when `bin_path` and `cookies_file` keys are absent from the config file

## 3.0.0
- Major: Updated to `iced` v0.13
- Major: Changed `bin_dir` to `bin_path`; the configuration file now takes the path of the `yt-dlp` binary instead of the directory where it's located
- Major : Add settings tab to the application with settings for `save_window_position` + `bin_path` in there
- Minor: Support cookie file selection for bypassing age restrictions when downloading. (thanks to @daem96 #56)
- Minor: Some changes to the application theme
- Minor: url passed with `--url` is no longer saved in the config file
- Bugfix: Bring back text input focus on application startup
- Bugfix: fix `yt-dlp` progress parsing
- Dev: add basic test for `yt-dlp` progress parsing

## 2.0.0

- Major: Remove download progress modal
- Major: Add `-u` and `--url` to start the application with a URL in the text input (thanks @coding-moding! #38)
- Major: Add SponsorBlock pick list for removing or marking sponsor blocks
- Minor: Focus download URL input on application start
- Minor: Support downloading multiple videos (add all the links into the text input with space between them)
- Bugfix: Properly kill child process
- Bugfix: fix crash on videos with quotes

## 1.1.1

- Minor: Added and option to the config file that saves the window positions and restores it on next launch

## 1.1.0

- Major: Updated `iced` to 0.12.1
- Minor: The download modal now resizes with the window (taking half the width and the height)
- Minor: made folder dialog not block the GUI thread
- Dev: Replaced native_dialog with rfd
- Dev: Update dependencies to be able to compile on rust v1.80
- Dev: Fix Linux CI

## 1.0.2

- Minor: Padding for ETA seconds and minutes. instead of `2:1` now it's `02:01`
- Minor: The application ships with glow support by default now

## 1.0.1

- Bugfix: parse ETA as float instead of integer

## 1.0.0
(got bored of the 0.whatever so I'm bumping it to 1.0.0)

- Major: Better download progress messages (including progress for playlist videos) since there is better yt-dlp parsing
- Major: Move downloads logs from `config_dir` to `cache_dir` linux: `$XDG_CACHE_HOME` or `$HOME/.cache` -- windows: `{FOLDERID_LocalAppData}` -- macos: `$HOME/Library/Caches`
- Major: Dev: Better yt-dlp parsing
- Bugfix: Use `.to_string_lossy()` for download dir instead of `.to_str()`, that will solve [this issue](https://github.com/BKSalman/ytdlp-gui/issues/12)
- Minor: Pressing `Enter` now starts the download (equivalent to clicking `Download` button)
- Minor: Use default configs if config file is broken
- Minor: Added `--version or -V` and `--help or -h` options to the binary to check the version
- Minor: Better error messages in general (there were almost none actually)
- Dev: Replace log4rs with tracing

## 0.3.0

- Major: Added general logs to stderr and a temporary file in temp directory
- Major: Added Download logs after finishing every download
- Major: Replaced radio buttons with a drop-down menu for selecting resolutions and formats
- Minor: Moved the download button to the bottom
- Minor: Moved the "Browse" button to the right of the path text box
- Minor: Options and settings now save on download instead of saving on app close
- Dev: Replaced env_logger with log4rs to use it for std logging and file logging

## 0.2.5

- Bugfix: Update the packaged yt-dlp version for windows, that will solve [this issue](https://github.com/BKSalman/ytdlp-gui/issues/13)
- Minor: Show message in modal when yt-dlp binary is missing

## 0.2.4

- Bugfix: Fixed crash when download folder is not set

## 0.2.3

- Minor: save current configs on application exit, instead of on every change

## 0.2.2

- Major: Added Config file to save previously chosen options, download path, and bin directory
- Dev: Small refactors

## 0.2.1

- Major: Update ``Iced`` to v0.7.0
- Bugfix: Merge format [#9](https://github.com/BKSalman/ytdlp-gui/issues/9)
