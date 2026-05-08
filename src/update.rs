use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;

const CURRENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Failed to fetch update: {0}")]
    UpdateFetchFailed(String),
    #[error("Failed to deserialize version: {0}")]
    DeserializationFailed(String),
    #[error("Invalid tag name: {0}")]
    InvalidTagName(String),
    #[error("Invalid version number: {0}")]
    InvalidVersionNumber(String),
}

#[derive(Debug, Clone)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('v');
        let mut parts = s.splitn(3, '.').map(|p| p.parse::<u32>());
        Ok(Version {
            major: parts
                .next()
                .ok_or_else(|| Error::InvalidVersionNumber(s.to_string()))?
                .map_err(|_| Error::InvalidVersionNumber(s.to_string()))?,
            minor: parts
                .next()
                .ok_or_else(|| Error::InvalidVersionNumber(s.to_string()))?
                .map_err(|_| Error::InvalidVersionNumber(s.to_string()))?,
            patch: parts
                .next()
                .ok_or_else(|| Error::InvalidVersionNumber(s.to_string()))?
                .map_err(|_| Error::InvalidVersionNumber(s.to_string()))?,
        })
    }
}

impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

pub async fn check_for_update() -> Result<Option<Version>, Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        HeaderValue::from_str(&format!("ytdlp-gui/{CURRENT_VERSION}")).unwrap(),
    );

    let release = reqwest::Client::new()
        .get("https://api.github.com/repos/BKSalman/ytdlp-gui/releases/latest")
        .headers(headers)
        .send()
        .await
        .map_err(|e| Error::UpdateFetchFailed(e.to_string()))?
        .json::<Release>()
        .await
        .map_err(|e| Error::DeserializationFailed(e.to_string()))?;

    let tag_name = release.tag_name.to_lowercase();
    let Some(version) = tag_name.strip_prefix('v') else {
        return Err(Error::InvalidTagName(release.tag_name));
    };

    let Ok(fetched_version) = Version::from_str(version) else {
        return Err(Error::InvalidVersionNumber(version.to_string()));
    };
    let Ok(current_version) = Version::from_str(CURRENT_VERSION) else {
        return Err(Error::InvalidVersionNumber(version.to_string()));
    };

    let fetched = (
        fetched_version.major,
        fetched_version.minor,
        fetched_version.patch,
    );
    let current = (
        current_version.major,
        current_version.minor,
        current_version.patch,
    );

    if fetched > current {
        return Ok(Some(fetched_version));
    }

    Ok(None)
}
