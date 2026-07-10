use crate::fl;

/// A browser yt-dlp can load cookies from via `--cookies-from-browser`.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    #[default]
    Disabled,
    Brave,
    Chrome,
    Chromium,
    Edge,
    Firefox,
    Opera,
    Safari,
    Vivaldi,
    Whale,
}

impl Browser {
    pub const ALL: [Browser; 10] = [
        Browser::Disabled,
        Browser::Brave,
        Browser::Chrome,
        Browser::Chromium,
        Browser::Edge,
        Browser::Firefox,
        Browser::Opera,
        Browser::Safari,
        Browser::Vivaldi,
        Browser::Whale,
    ];

    pub fn arg(self) -> Option<&'static str> {
        Some(match self {
            Browser::Disabled => return None,
            Browser::Brave => "brave",
            Browser::Chrome => "chrome",
            Browser::Chromium => "chromium",
            Browser::Edge => "edge",
            Browser::Firefox => "firefox",
            Browser::Opera => "opera",
            Browser::Safari => "safari",
            Browser::Vivaldi => "vivaldi",
            Browser::Whale => "whale",
        })
    }

    pub fn from_arg(name: &str) -> Browser {
        let name = name.trim().to_ascii_lowercase();
        Browser::ALL
            .into_iter()
            .find(|browser| browser.arg() == Some(name.as_str()))
            .unwrap_or(Browser::Disabled)
    }

    pub fn supports_keyring(self) -> bool {
        matches!(
            self,
            Browser::Brave
                | Browser::Chrome
                | Browser::Chromium
                | Browser::Edge
                | Browser::Opera
                | Browser::Vivaldi
                | Browser::Whale
        )
    }

    pub fn supports_container(self) -> bool {
        matches!(self, Browser::Firefox)
    }
}

impl core::fmt::Display for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Browser::Disabled => f.write_str(&fl!("cookies_browser_none")),
            Browser::Brave => f.write_str("Brave"),
            Browser::Chrome => f.write_str("Chrome"),
            Browser::Chromium => f.write_str("Chromium"),
            Browser::Edge => f.write_str("Edge"),
            Browser::Firefox => f.write_str("Firefox"),
            Browser::Opera => f.write_str("Opera"),
            Browser::Safari => f.write_str("Safari"),
            Browser::Vivaldi => f.write_str("Vivaldi"),
            Browser::Whale => f.write_str("Whale"),
        }
    }
}

/// The keyring yt-dlp uses to decrypt Chromium cookies on Linux. `Auto` leaves
/// the choice to yt-dlp (no `+KEYRING` is emitted).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyring {
    #[default]
    Auto,
    BasicText,
    GnomeKeyring,
    KWallet,
    KWallet5,
    KWallet6,
}

impl Keyring {
    pub const ALL: [Keyring; 6] = [
        Keyring::Auto,
        Keyring::BasicText,
        Keyring::GnomeKeyring,
        Keyring::KWallet,
        Keyring::KWallet5,
        Keyring::KWallet6,
    ];

    pub fn arg(self) -> Option<&'static str> {
        Some(match self {
            Keyring::Auto => return None,
            Keyring::BasicText => "basictext",
            Keyring::GnomeKeyring => "gnomekeyring",
            Keyring::KWallet => "kwallet",
            Keyring::KWallet5 => "kwallet5",
            Keyring::KWallet6 => "kwallet6",
        })
    }

    pub fn from_arg(name: &str) -> Keyring {
        let name = name.trim().to_ascii_lowercase();
        Keyring::ALL
            .into_iter()
            .find(|keyring| keyring.arg() == Some(name.as_str()))
            .unwrap_or(Keyring::Auto)
    }
}

impl core::fmt::Display for Keyring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyring::Auto => f.write_str(&fl!("cookies_keyring_auto")),
            Keyring::BasicText => f.write_str("Basic text"),
            Keyring::GnomeKeyring => f.write_str("GNOME Keyring"),
            Keyring::KWallet => f.write_str("KWallet"),
            Keyring::KWallet5 => f.write_str("KWallet 5"),
            Keyring::KWallet6 => f.write_str("KWallet 6"),
        }
    }
}

/// The parsed components of yt-dlp's
/// `BROWSER[+KEYRING][:PROFILE][::CONTAINER]` specification.
///
/// The config stores the assembled string in a single field, so this is the
/// bridge between that string and the individual widgets in the settings tab.
#[derive(Default, Debug, Clone)]
pub struct CookiesFromBrowser {
    pub browser: Browser,
    pub keyring: Keyring,
    pub profile: String,
    pub container: String,
}

impl CookiesFromBrowser {
    pub fn from_config(value: &str) -> Self {
        // `::CONTAINER` is the trailing segment.
        let (head, container) = match value.split_once("::") {
            Some((head, container)) => (head, container.to_string()),
            None => (value, String::new()),
        };

        // The browser name runs until the first `+` (keyring) or `:` (profile).
        let name_end = head.find(['+', ':']).unwrap_or(head.len());
        let browser = Browser::from_arg(&head[..name_end]);
        let mut rest = &head[name_end..];

        // Optional `+KEYRING`, up to the next `:`.
        let mut keyring = Keyring::Auto;
        if let Some(after_plus) = rest.strip_prefix('+') {
            let keyring_end = after_plus.find(':').unwrap_or(after_plus.len());
            keyring = Keyring::from_arg(&after_plus[..keyring_end]);
            rest = &after_plus[keyring_end..];
        }

        // Whatever remains after `:` is the profile.
        let profile = rest.strip_prefix(':').unwrap_or("").to_string();

        Self {
            browser,
            keyring,
            profile,
            container,
        }
    }

    /// Assemble the `--cookies-from-browser` argument, or `None` when no browser
    /// is selected. Keyring and container are only emitted for the browsers that
    /// support them.
    pub fn to_arg(&self) -> Option<String> {
        let mut arg = String::from(self.browser.arg()?);

        if self.browser.supports_keyring() {
            if let Some(keyring) = self.keyring.arg() {
                arg.push('+');
                arg.push_str(keyring);
            }
        }

        if !self.profile.is_empty() {
            arg.push(':');
            arg.push_str(&self.profile);
        }

        if self.browser.supports_container() && !self.container.is_empty() {
            arg.push_str("::");
            arg.push_str(&self.container);
        }

        Some(arg)
    }
}
