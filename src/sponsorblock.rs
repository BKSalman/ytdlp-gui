#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SponsorBlockOption {
    Remove,
    Mark,
}

impl core::fmt::Display for SponsorBlockOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SponsorBlockOption::Remove => write!(f, "Remove"),
            SponsorBlockOption::Mark => write!(f, "Mark"),
        }
    }
}
