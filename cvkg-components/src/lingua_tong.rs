//! LinguaTong -- i18n Localization Infrastructure.
//!
//! Provides translation lookup, locale management, and RTL detection.
//! All user-visible strings in components should go through this system.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

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
        if let Some(table) = guard.get(&locale)
            && let Some(value) = table.get(key)
        {
            return value.clone();
        }
        // Fallback to "en"
        if locale != "en"
            && let Some(table) = guard.get("en")
            && let Some(value) = table.get(key)
        {
            return value.clone();
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
    en.insert(
        "tokenstream.streaming".to_string(),
        "Streaming...".to_string(),
    );
    en.insert("tokenstream.complete".to_string(), "Complete".to_string());

    // TrustMark
    en.insert("trustmark.high".to_string(), "High confidence".to_string());
    en.insert(
        "trustmark.medium".to_string(),
        "Moderate confidence".to_string(),
    );
    en.insert(
        "trustmark.low".to_string(),
        "Low confidence - verify".to_string(),
    );
    en.insert(
        "trustmark.verylow".to_string(),
        "Speculative - unreliable".to_string(),
    );
    en.insert(
        "trustmark.unknown".to_string(),
        "Confidence unknown".to_string(),
    );

    // DropVault
    en.insert(
        "dropvault.drop_here".to_string(),
        "Drop files here".to_string(),
    );
    en.insert(
        "dropvault.browse".to_string(),
        "Drag files here or click to browse".to_string(),
    );
    en.insert("dropvault.waiting".to_string(), "Waiting...".to_string());
    en.insert("dropvault.done".to_string(), "Done".to_string());

    // ConsentGate
    en.insert(
        "consentgate.title".to_string(),
        "Data Usage Consent".to_string(),
    );
    en.insert("consentgate.accept".to_string(), "Accept".to_string());
    en.insert("consentgate.reject".to_string(), "Reject".to_string());
    en.insert(
        "consentgate.data_label".to_string(),
        "Data: {data}".to_string(),
    );
    en.insert(
        "consentgate.purpose_label".to_string(),
        "Purpose: {purpose}".to_string(),
    );
    en.insert(
        "consentgate.data_used".to_string(),
        "Data used:".to_string(),
    );
    en.insert(
        "consentgate.items_count".to_string(),
        "({count} items)".to_string(),
    );

    // AwaitVeil
    en.insert("awaitveil.loading".to_string(), "Loading...".to_string());

    // Dialog
    en.insert("dialog.cancel".to_string(), "Cancel".to_string());
    en.insert("dialog.delete".to_string(), "Delete".to_string());
    en.insert("dialog.ok".to_string(), "OK".to_string());
    en.insert("dialog.confirm".to_string(), "Confirm".to_string());

    // DatePicker
    en.insert(
        "datepicker.placeholder".to_string(),
        "Select date...".to_string(),
    );
    en.insert(
        "datepicker.range_placeholder".to_string(),
        "Select range...".to_string(),
    );
    en.insert("datepicker.label".to_string(), "Date picker".to_string());
    en.insert("datepicker.format".to_string(), "DD/MM/YYYY".to_string());
    en.insert(
        "datepicker.month.january".to_string(),
        "January".to_string(),
    );
    en.insert(
        "datepicker.month.february".to_string(),
        "February".to_string(),
    );
    en.insert("datepicker.month.march".to_string(), "March".to_string());
    en.insert("datepicker.month.april".to_string(), "April".to_string());
    en.insert("datepicker.month.may".to_string(), "May".to_string());
    en.insert("datepicker.month.june".to_string(), "June".to_string());
    en.insert("datepicker.month.july".to_string(), "July".to_string());
    en.insert("datepicker.month.august".to_string(), "August".to_string());
    en.insert(
        "datepicker.month.september".to_string(),
        "September".to_string(),
    );
    en.insert(
        "datepicker.month.october".to_string(),
        "October".to_string(),
    );
    en.insert(
        "datepicker.month.november".to_string(),
        "November".to_string(),
    );
    en.insert(
        "datepicker.month.december".to_string(),
        "December".to_string(),
    );
    en.insert("datepicker.day.su".to_string(), "Su".to_string());
    en.insert("datepicker.day.mo".to_string(), "Mo".to_string());
    en.insert("datepicker.day.tu".to_string(), "Tu".to_string());
    en.insert("datepicker.day.we".to_string(), "We".to_string());
    en.insert("datepicker.day.th".to_string(), "Th".to_string());
    en.insert("datepicker.day.fr".to_string(), "Fr".to_string());
    en.insert("datepicker.day.sa".to_string(), "Sa".to_string());

    // Abbreviated month names (used by DateRangePicker and calendar headers)
    en.insert("datepicker.month.jan".to_string(), "Jan".to_string());
    en.insert("datepicker.month.feb".to_string(), "Feb".to_string());
    en.insert("datepicker.month.mar".to_string(), "Mar".to_string());
    en.insert("datepicker.month.apr".to_string(), "Apr".to_string());
    en.insert("datepicker.month.may_short".to_string(), "May".to_string());
    en.insert("datepicker.month.jun".to_string(), "Jun".to_string());
    en.insert("datepicker.month.jul".to_string(), "Jul".to_string());
    en.insert("datepicker.month.aug".to_string(), "Aug".to_string());
    en.insert("datepicker.month.sep".to_string(), "Sep".to_string());
    en.insert("datepicker.month.oct".to_string(), "Oct".to_string());
    en.insert("datepicker.month.nov".to_string(), "Nov".to_string());
    en.insert("datepicker.month.dec".to_string(), "Dec".to_string());

    load_translations("en", en);
}

/// Initialize with Japanese translations.
pub fn init_japanese_translations() {
    let mut ja = HashMap::new();

    // PhaseGate
    ja.insert("phasegate.tooltip".to_string(), "ツールチップ".to_string());
    ja.insert(
        "phasegate.dropdown".to_string(),
        "ドロップダウン".to_string(),
    );
    ja.insert("phasegate.modal".to_string(), "モーダル".to_string());
    ja.insert("phasegate.toast".to_string(), "トースト".to_string());

    // TokenStream
    ja.insert(
        "tokenstream.streaming".to_string(),
        "ストリーミング中...".to_string(),
    );
    ja.insert("tokenstream.complete".to_string(), "完了".to_string());

    // TrustMark
    ja.insert("trustmark.high".to_string(), "高い信頼性".to_string());
    ja.insert("trustmark.medium".to_string(), "中程度の信頼性".to_string());
    ja.insert(
        "trustmark.low".to_string(),
        "低い信頼性 - 要確認".to_string(),
    );
    ja.insert(
        "trustmark.verylow".to_string(),
        "推測 - 信頼できない".to_string(),
    );
    ja.insert("trustmark.unknown".to_string(), "信頼性不明".to_string());

    // DropVault
    ja.insert(
        "dropvault.drop_here".to_string(),
        "ここにファイルをドロップ".to_string(),
    );
    ja.insert(
        "dropvault.browse".to_string(),
        "ここにファイルをドラッグするか、クリックして参照".to_string(),
    );
    ja.insert("dropvault.waiting".to_string(), "待機中...".to_string());
    ja.insert("dropvault.done".to_string(), "完了".to_string());

    // ConsentGate
    ja.insert(
        "consentgate.title".to_string(),
        "データ利用の同意".to_string(),
    );
    ja.insert("consentgate.accept".to_string(), "同意する".to_string());
    ja.insert("consentgate.reject".to_string(), "拒否する".to_string());
    ja.insert(
        "consentgate.data_label".to_string(),
        "データ: {data}".to_string(),
    );
    ja.insert(
        "consentgate.purpose_label".to_string(),
        "目的: {purpose}".to_string(),
    );
    ja.insert(
        "consentgate.data_used".to_string(),
        "使用データ:".to_string(),
    );
    ja.insert(
        "consentgate.items_count".to_string(),
        "({count} 件)".to_string(),
    );

    // AwaitVeil
    ja.insert("awaitveil.loading".to_string(), "読み込み中...".to_string());

    // Dialog
    ja.insert("dialog.cancel".to_string(), "キャンセル".to_string());
    ja.insert("dialog.delete".to_string(), "削除".to_string());
    ja.insert("dialog.ok".to_string(), "OK".to_string());
    ja.insert("dialog.confirm".to_string(), "確認".to_string());

    // DatePicker
    ja.insert(
        "datepicker.placeholder".to_string(),
        "日付を選択...".to_string(),
    );
    ja.insert(
        "datepicker.range_placeholder".to_string(),
        "範囲を選択...".to_string(),
    );
    ja.insert("datepicker.label".to_string(), "日付ピッカー".to_string());
    ja.insert("datepicker.format".to_string(), "YYYY/MM/DD".to_string());
    ja.insert("datepicker.month.january".to_string(), "1月".to_string());
    ja.insert("datepicker.month.february".to_string(), "2月".to_string());
    ja.insert("datepicker.month.march".to_string(), "3月".to_string());
    ja.insert("datepicker.month.april".to_string(), "4月".to_string());
    ja.insert("datepicker.month.may".to_string(), "5月".to_string());
    ja.insert("datepicker.month.june".to_string(), "6月".to_string());
    ja.insert("datepicker.month.july".to_string(), "7月".to_string());
    ja.insert("datepicker.month.august".to_string(), "8月".to_string());
    ja.insert("datepicker.month.september".to_string(), "9月".to_string());
    ja.insert("datepicker.month.october".to_string(), "10月".to_string());
    ja.insert("datepicker.month.november".to_string(), "11月".to_string());
    ja.insert("datepicker.month.december".to_string(), "12月".to_string());
    ja.insert("datepicker.day.su".to_string(), "日".to_string());
    ja.insert("datepicker.day.mo".to_string(), "月".to_string());
    ja.insert("datepicker.day.tu".to_string(), "火".to_string());
    ja.insert("datepicker.day.we".to_string(), "水".to_string());
    ja.insert("datepicker.day.th".to_string(), "木".to_string());
    ja.insert("datepicker.day.fr".to_string(), "金".to_string());
    ja.insert("datepicker.day.sa".to_string(), "土".to_string());

    // Abbreviated month names (Japanese months are the same abbreviated/full)
    ja.insert("datepicker.month.jan".to_string(), "1月".to_string());
    ja.insert("datepicker.month.feb".to_string(), "2月".to_string());
    ja.insert("datepicker.month.mar".to_string(), "3月".to_string());
    ja.insert("datepicker.month.apr".to_string(), "4月".to_string());
    ja.insert("datepicker.month.may_short".to_string(), "5月".to_string());
    ja.insert("datepicker.month.jun".to_string(), "6月".to_string());
    ja.insert("datepicker.month.jul".to_string(), "7月".to_string());
    ja.insert("datepicker.month.aug".to_string(), "8月".to_string());
    ja.insert("datepicker.month.sep".to_string(), "9月".to_string());
    ja.insert("datepicker.month.oct".to_string(), "10月".to_string());
    ja.insert("datepicker.month.nov".to_string(), "11月".to_string());
    ja.insert("datepicker.month.dec".to_string(), "12月".to_string());

    load_translations("ja", ja);
}

/// Shorthand macro for translation lookup.
/// Usage: `t!("key")` or `t!("key", "default")`
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::lingua_tong::t($key)
    };
    ($key:expr, $default:expr) => {{
        let result = $crate::lingua_tong::t($key);
        if result == $key {
            $default.to_string()
        } else {
            result
        }
    }};
}

/// Helper to initialize English translations with common UI strings.
/// Call this at app startup before using any components with i18n labels.
pub fn init_english() {
    use std::collections::HashMap;
    let mut en = HashMap::new();
    // Common actions
    en.insert("action.ok".to_string(), "OK".to_string());
    en.insert("action.cancel".to_string(), "Cancel".to_string());
    en.insert("action.save".to_string(), "Save".to_string());
    en.insert("action.delete".to_string(), "Delete".to_string());
    en.insert("action.edit".to_string(), "Edit".to_string());
    en.insert("action.create".to_string(), "Create".to_string());
    en.insert("action.submit".to_string(), "Submit".to_string());
    en.insert("action.close".to_string(), "Close".to_string());
    en.insert("action.back".to_string(), "Back".to_string());
    en.insert("action.next".to_string(), "Next".to_string());
    en.insert("action.search".to_string(), "Search".to_string());
    en.insert("action.filter".to_string(), "Filter".to_string());
    en.insert("action.refresh".to_string(), "Refresh".to_string());
    // Navigation
    en.insert("nav.home".to_string(), "Home".to_string());
    en.insert("nav.settings".to_string(), "Settings".to_string());
    en.insert("nav.profile".to_string(), "Profile".to_string());
    en.insert("nav.logout".to_string(), "Sign out".to_string());
    // Status
    en.insert("status.loading".to_string(), "Loading...".to_string());
    en.insert("status.error".to_string(), "Error".to_string());
    en.insert("status.success".to_string(), "Success".to_string());
    en.insert("status.empty".to_string(), "No data".to_string());
    load_translations("en", en);
}
