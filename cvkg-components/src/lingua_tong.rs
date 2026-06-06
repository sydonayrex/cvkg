//! LinguaTong — i18n Localization Infrastructure.
//!
//! Provides translation lookup, locale management, and RTL detection.
//! All user-visible strings in components should go through this system.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

static LOCALE: OnceLock<Mutex<String>> = OnceLock::new();
static TRANSLATIONS: OnceLock<Mutex<HashMap<String, HashMap<String, String>>>> = OnceLock::new();

/// Set the active locale. Should be called once at app startup.
pub fn set_locale(locale: &str) {
    let _ = LOCALE.get_or_init(|| Mutex::new(locale.to_string()));
}

/// Get the current locale.
pub fn current_locale() -> String {
    LOCALE
        .get()
        .and_then(|l| l.lock().ok())
        .map(|g| g.clone())
        .unwrap_or_else(|| "en".to_string())
}

/// Load translations for a locale from a key-value map.
pub fn load_translations(locale: &str, table: HashMap<String, String>) {
    let translations = TRANSLATIONS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = translations.lock() {
        guard.insert(locale.to_string(), table);
    }
}

/// Look up a translation key. Falls back to the key itself if not found.
pub fn t(key: &str) -> String {
    let locale = current_locale();
    let translations = TRANSLATIONS.get().and_then(|t| t.lock().ok());

    if let Some(guard) = translations {
        if let Some(table) = guard.get(&locale) {
            if let Some(value) = table.get(key) {
                return value.clone();
            }
        }
        // Fallback to "en"
        if locale != "en" {
            if let Some(table) = guard.get("en") {
                if let Some(value) = table.get(key) {
                    return value.clone();
                }
            }
        }
    }

    key.to_string()
}

/// Look up a translation with simple interpolation.
///
/// # Example:
/// ```no_run
/// use cvkg_components::lingua_tong::{init_english_translations, t_with};
///
/// init_english_translations();
/// let msg = t_with("trustmark.high", &[("name", "Alice")]);
/// // Returns: "High confidence" (from the English translations)
/// ```
pub fn t_with(key: &str, args: &[(&str, &str)]) -> String {
    let mut result = t(key);
    for (name, value) in args {
        result = result.replace(&format!("{{{}}}", name), value);
    }
    result
}

/// Detect if the current locale uses right-to-left text.
pub fn is_rtl() -> bool {
    matches!(current_locale().as_str(), "ar" | "he" | "fa" | "ur")
}

/// Initialize with English translations for all new component strings.
pub fn init_english_translations() {
    let mut en = HashMap::new();

    // PhaseGate
    en.insert("phasegate.tooltip".to_string(), "Tooltip".to_string());
    en.insert("phasegate.dropdown".to_string(), "Dropdown".to_string());
    en.insert("phasegate.modal".to_string(), "Modal".to_string());
    en.insert("phasegate.toast".to_string(), "Toast".to_string());

    // TokenStream
    en.insert("tokenstream.streaming".to_string(), "Streaming...".to_string());
    en.insert("tokenstream.complete".to_string(), "Complete".to_string());

    // TrustMark
    en.insert("trustmark.high".to_string(), "High confidence".to_string());
    en.insert("trustmark.medium".to_string(), "Moderate confidence".to_string());
    en.insert("trustmark.low".to_string(), "Low confidence - verify".to_string());
    en.insert("trustmark.verylow".to_string(), "Speculative - unreliable".to_string());
    en.insert("trustmark.unknown".to_string(), "Confidence unknown".to_string());

    // DropVault
    en.insert("dropvault.drop_here".to_string(), "Drop files here".to_string());
    en.insert("dropvault.browse".to_string(), "Drag files here or click to browse".to_string());
    en.insert("dropvault.waiting".to_string(), "Waiting...".to_string());
    en.insert("dropvault.done".to_string(), "Done".to_string());

    // ConsentGate
    en.insert("consentgate.title".to_string(), "Data Usage Consent".to_string());
    en.insert("consentgate.accept".to_string(), "Accept".to_string());
    en.insert("consentgate.reject".to_string(), "Reject".to_string());

    // AwaitVeil
    en.insert("awaitveil.loading".to_string(), "Loading...".to_string());

    load_translations("en", en);
}
