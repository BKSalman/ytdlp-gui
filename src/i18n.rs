//! Provides localization support for this crate.

use std::sync::LazyLock;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    unic_langid::LanguageIdentifier,
    DefaultLocalizer, LanguageLoader, Localizer,
};
use rust_embed::RustEmbed;

/// Applies the requested language(s) to requested translations from the `fl!()` macro.
pub fn init(requested_languages: &[LanguageIdentifier]) {
    if let Err(why) = localizer().select(requested_languages) {
        eprintln!("error while loading fluent localizations: {why}");
    }
}

// Get the `Localizer` to be used for localizing this library.
#[must_use]
pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

/// Request a localized string by ID from the i18n/ directory.
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}
