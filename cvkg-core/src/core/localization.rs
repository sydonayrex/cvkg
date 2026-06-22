// =============================================================================
// LOCALIZATION -- Item 12: Localization / Internationalization
// =============================================================================
// OS-agnostic: works on all platforms. No platform-specific string loading.

use std::sync::RwLock;

/// Layout direction for UI elements (LTR or RTL).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    LTR,
    RTL,
    Auto,
}

impl Direction {
    pub fn is_rtl(self) -> bool {
        matches!(self, Direction::RTL)
    }
}
#[derive(Clone, Debug)]
pub struct L10nBundle {
    pub locale: String,
    pub strings: HashMap<String, String>,
    pub is_rtl: bool,
}

impl L10nBundle {
    pub fn new(locale: impl Into<String>) -> Self {
        Self {
            locale: locale.into(),
            strings: HashMap::new(),
            is_rtl: false,
        }
    }

    pub fn add(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.strings.insert(key.into(), value.into());
        self
    }

    pub fn from_strings_format(locale: impl Into<String>, input: &str) -> Self {
        let mut bundle = Self::new(locale);
        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            if let Some(eq_pos) = line.find(" = ") {
                let key = line[..eq_pos].trim_matches('"').to_string();
                let val = line[eq_pos + 3..]
                    .trim_end_matches(';')
                    .trim_matches('"')
                    .to_string();
                bundle.strings.insert(key, val);
            }
        }
        bundle
    }
    /// Get a translated string by key. Returns the key itself if not found.
    pub fn t(&self, key: &str) -> String {
        self.strings
            .get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| key.to_string())
    }

    /// Translate with interpolation. Replaces {0}, {1}, etc. with args.
    pub fn tf(&self, key: &str, args: &[&str]) -> String {
        let mut result = self.t(key);
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), arg);
        }
        result
    }
}

/// Global localization manager.
pub struct L10n {
    bundles: HashMap<String, L10nBundle>,
    current: String,
}

impl L10n {
    pub fn new(default_locale: &str) -> Self {
        Self {
            bundles: HashMap::new(),
            current: default_locale.to_string(),
        }
    }

    pub fn add_bundle(&mut self, bundle: L10nBundle) {
        self.bundles.insert(bundle.locale.clone(), bundle);
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.current = locale.to_string();
    }
    pub fn current_locale(&self) -> &str {
        &self.current
    }

    pub fn is_rtl(&self) -> bool {
        self.bundles
            .get(self.current.as_str())
            .map(|b| b.is_rtl)
            .unwrap_or(false)
    }

    pub fn t(&self, key: &str) -> String {
        self.bundles
            .get(self.current.as_str())
            .map(|b| b.t(key))
            .unwrap_or_else(|| key.to_string())
    }

    pub fn tf(&self, key: &str, args: &[&str]) -> String {
        let mut result = self.t(key);
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), arg);
        }
        result
    }

    pub fn direction(&self) -> Direction {
        if self.is_rtl() {
            Direction::RTL
        } else {
            Direction::LTR
        }
    }
}

static L10N: once_cell::sync::Lazy<Arc<RwLock<L10n>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(L10n::new("en"))));

pub fn init_l10n(l10n: L10n) {
    if let Ok(mut guard) = L10N.write() {
        *guard = l10n;
    }
}

pub fn l10n() -> Arc<RwLock<L10n>> {
    L10N.clone()
}

pub fn t(key: &str) -> String {
    L10N.read()
        .map(|g| g.t(key).to_string())
        .unwrap_or_else(|_| key.to_string())
}

pub fn tf(key: &str, args: &[&str]) -> String {
    L10N.read()
        .map(|g| g.tf(key, args))
        .unwrap_or_else(|_| key.to_string())
}

pub fn set_locale(locale: &str) {
    if let Ok(mut guard) = L10N.write() {
        guard.set_locale(locale);
    }
}

pub fn current_locale() -> String {
    L10N.read()
        .map(|g| g.current_locale().to_string())
        .unwrap_or_else(|_| "en".to_string())
}

pub fn is_rtl() -> bool {
    L10N.read().map(|g| g.is_rtl()).unwrap_or(false)
}

