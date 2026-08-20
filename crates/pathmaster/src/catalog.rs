//! The Catalogue at runtime: the embedded `.mo` files, the loader that serves
//! them from memory, and the one lookup every string goes through (spec §11).
//!
//! There is exactly one Catalogue, and that is structural rather than
//! disciplinary: visible labels and Announcements alike come through
//! [`translate`], so what is shown and what is spoken cannot drift apart. The
//! log is deliberately outside it — a diagnostic artifact stays greppable and
//! independent of catalogue completeness — and that too is structural: every
//! log line is built in `pathmaster-core`, which cannot reach this module
//! without reversing the dependency direction (ADR-0007).
//!
//! Nothing is read from disk. The catalogues are `include_bytes!`-ed from
//! `OUT_DIR`, where `build.rs` compiled them from `i18n/*.po`, so a single file
//! stays a single file (NFR-portable).
//!
//! What is *composed* out of those strings is not here: it is
//! `pathmaster_core::catalogue`, which reaches this module through the
//! [`Installed`] adapter below (ADR-0009). That is still one lookup — the
//! adapter wraps [`translate`] rather than adding a second answer — and it is
//! what lets a composition rule be tested without linking wxWidgets.
//!
//! The module keeps gettext's spelling (spec §17 names it `catalog`, as do
//! `add_catalog` and `.mo` catalogs); the domain term for what it serves is the
//! Catalogue.

use std::borrow::Cow;

use pathmaster_core::catalogue::Lookup;
use pathmaster_core::language::Language;
use wxdragon::translations::{
    translate as wx_translate, translate_plural as wx_translate_plural, Translations,
    TranslationsLoader,
};

// `static CATALOGUES: &[(&str, &[u8])]` — one row per `i18n/<code>.po`.
include!(concat!(env!("OUT_DIR"), "/catalogues.rs"));

/// The Catalogue's gettext domain. One domain, one Catalogue.
const DOMAIN: &str = "pathmaster";

/// Serves the compiled catalogues out of the executable's own bytes.
struct Embedded;

impl TranslationsLoader for Embedded {
    fn load_catalog(&self, domain: &str, lang: &str) -> Option<Cow<'_, [u8]>> {
        if domain != DOMAIN {
            return None;
        }
        CATALOGUES
            .iter()
            .find(|(code, _)| *code == lang)
            .map(|(_, mo)| Cow::Borrowed(*mo))
    }

    fn available_translations(&self, domain: &str) -> Vec<String> {
        if domain != DOMAIN {
            return Vec::new();
        }
        CATALOGUES
            .iter()
            .map(|(code, _)| (*code).to_owned())
            .collect()
    }
}

/// Installs the Catalogue for `language` as the global one.
///
/// Both languages load a catalogue, English included: its own is the identity
/// of the msgids, gated as such, so neither language rides the fallback and
/// neither is a special case here. The fallback is still the safety net it was
/// — a lookup that finds nothing returns the msgid, which is English.
///
/// `add_std_catalog()` is never called: wx's own "OK"/"Cancel" are not ours,
/// and every dialog whose button text carries meaning builds its own.
pub fn install(language: Language) {
    let translations = Translations::new();
    translations.set_loader(Embedded);
    // By code, never by `set_language` — wxdragon's `Language` enum mirrors
    // wxWidgets 3.2's ordinals and the vendored 3.3.3 renumbered them, so the
    // enum names a different language than it says (see `platform::locale`).
    translations.set_language_str(language.code());
    // `false` would mean neither a catalogue nor the msgid language matched.
    // The completeness gate and the smoke test below make that unreachable for
    // both shipped languages, so there is nothing here for a run to report.
    translations.add_catalog(DOMAIN);
    translations.set_global();
}

/// The one lookup. A miss returns the msgid, which is English source text —
/// degrading to readable English rather than to a symbolic key is why msgids
/// are not keys (ADR-0004).
pub fn translate(msgid: &str) -> String {
    wx_translate(msgid)
}

/// The one lookup for a string whose wording depends on a count. `singular` is
/// the msgid both forms are found by; the catalogue's own `Plural-Forms` rule
/// picks between them, so Ukrainian's three forms need nothing here.
pub fn translate_plural(singular: &str, plural: &str, n: u32) -> String {
    wx_translate_plural(singular, plural, n)
}

/// The production side of `pathmaster_core`'s lookup seam: the Catalogue
/// [`install`] installed, asked through the two functions above (ADR-0009).
///
/// It holds nothing, and that is the point — [`Translations::set_global`]
/// hands ownership to wx, and the free `translate` is how wx is asked
/// afterwards. An adapter holding a `Translations` would be a second Catalogue
/// with a second lifetime.
pub struct Installed;

impl Lookup for Installed {
    fn translate(&self, msgid: &str) -> String {
        translate(msgid)
    }

    fn translate_plural(&self, singular: &str, plural: &str, n: u32) -> String {
        translate_plural(singular, plural, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pathmaster_core::catalogue::{Announcement, Catalogue, UndoStep};
    use pathmaster_core::msgids::{REGISTRY, TAB_USER};
    use pathmaster_core::session::{Operation, Scope, UndoOutcome};

    /// The wx half of the completeness gate, and the only test that links
    /// wxWidgets (ADR-0007). The `.po`-side gate lives in `pathmaster-core`;
    /// this one proves the same strings survive compilation into a `.mo`,
    /// embedding, and lookup through wx — the part a pure test cannot see.
    ///
    /// It runs the composed assertions **through [`Installed`]** rather than
    /// past it, so the one line of production glue between core's composition
    /// and wx is covered, and so that one Announcement is asserted whole, in
    /// real Ukrainian, with wx choosing the plural form (ADR-0009).
    ///
    /// It is one test rather than several because it installs a global: wx is
    /// not thread-safe and cargo runs tests in threads.
    #[test]
    fn every_registered_msgid_is_answered_by_the_embedded_catalogues() {
        // English first, and asked the same way as any other language. A value
        // comparison could not tell a loaded English catalogue from a missing
        // one — both answer with the msgid — so presence is the only question
        // that means anything here.
        install(Language::English);
        let english = Translations::get().expect("the Catalogue is installed");
        for entry in REGISTRY {
            assert!(
                english.get_string(entry.msgid, "").is_some(),
                "the embedded English catalogue does not answer for {:?}",
                entry.msgid
            );
        }

        install(Language::Ukrainian);
        let translations = Translations::get().expect("the Catalogue is installed");

        for code in ["en", "uk"] {
            assert!(
                translations
                    .get_available_translations(DOMAIN)
                    .contains(&code.to_string()),
                "the loader must offer {code} before wx will ask for it"
            );
        }

        for entry in REGISTRY {
            // Presence is asked of wx directly. Comparing a translation to its
            // msgid would flag every string whose Ukrainian equals its English.
            assert!(
                translations.get_string(entry.msgid, "").is_some(),
                "the embedded catalogue does not answer for {:?}",
                entry.msgid
            );
            if let Some(plural) = entry.plural {
                assert!(
                    translations
                        .get_plural_string(entry.msgid, plural, 2, "")
                        .is_some(),
                    "the embedded catalogue does not answer the plural {:?}",
                    entry.msgid
                );
            }
        }

        // Self-sensitivity: a msgid that was never registered must come back
        // empty-handed, or "present" would mean nothing above.
        assert!(translations
            .get_string("There is no such string in PathMaster", "")
            .is_none());

        // The bytes really are Ukrainian: a bare label, looked up as every
        // bare label is.
        assert_eq!(translate(TAB_USER), "PATH користувача");

        // And composition really runs on them. nplurals=3 means 1, 2 and 5 are
        // three different words, and the form is chosen by wx — which is the
        // reason core's tests do not choose it (ADR-0009).
        let catalogue = Catalogue::new(Installed);
        let count = |n| {
            catalogue.announcement(Announcement::EntryCount {
                scope: Scope::User,
                count: n,
            })
        };
        assert_eq!(count(1), "PATH користувача: 1 запис");
        assert_eq!(count(2), "PATH користувача: 2 записи");
        assert_eq!(count(5), "PATH користувача: 5 записів");

        // One Announcement end-to-end, in the language the user hears: a
        // translated template, a translated operation name filled into it, and
        // the suffix an undo across the Apply barrier earns (spec §10.1 items
        // 4 and 5). Three Catalogue lookups and two composition rules in one
        // sentence — the assertion core's identity adapter cannot make.
        assert_eq!(
            catalogue.announcement(Announcement::UndoRedo {
                step: UndoStep::Undone,
                outcome: UndoOutcome {
                    focus: None,
                    operation: Operation::Delete,
                    crossed_apply: true,
                },
            }),
            "Скасовано: Видалення запису, незбережені зміни"
        );
    }
}
