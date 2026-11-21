use serde::{Deserialize, Serialize};

// =============================================================================
// KEYBOARD NAVIGATION
// =============================================================================

/// Modifier keys for keyboard events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// A keyboard shortcut binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyShortcut {
    pub key: String,
    pub modifiers: KeyModifiers,
    pub description: String,
}

impl KeyShortcut {
    pub fn new(key: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: KeyModifiers::default(),
            description: desc.into(),
        }
    }
    pub fn with_ctrl(mut self) -> Self {
        self.modifiers.ctrl = true;
        self
    }
    pub fn with_shift(mut self) -> Self {
        self.modifiers.shift = true;
        self
    }
    pub fn with_alt(mut self) -> Self {
        self.modifiers.alt = true;
        self
    }
    pub fn with_meta(mut self) -> Self {
        self.modifiers.meta = true;
        self
    }
}

// =============================================================================
// INPUT PARITY (P1-5): shared shortcut parsing and matching
// =============================================================================
//
// Native front-ends report physical key codes (`KeyS`, `ShiftLeft`, `OSLeft`),
// while web front-ends report logical keys (`s`, `Shift`, `Meta`) plus modifier
// booleans. Without a shared layer each backend has to reinvent shortcut
// parsing, which is how the two platforms drifted apart. The helpers below are
// pure and additive: they consume the existing `KeyModifiers`/`KeyShortcut`
// types and change no existing signature, so both platforms can adopt them
// incrementally.

/// Which OS convention a shortcut uses for its "primary" modifier.
///
/// macOS maps Command (the `meta` flag); Windows and Linux map Control.
/// Web callers should choose a variant from their host-platform detection,
/// since `KeyboardEvent` exposes both `metaKey` and `ctrlKey`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShortcutPlatform {
    /// macOS / iOS: the primary shortcut modifier is Command (`meta`).
    Apple,
    /// Windows, Linux and other platforms: the primary modifier is Control.
    #[default]
    Other,
}

/// Failure modes for [`KeyShortcut::parse`].
///
/// Parsing never panics: malformed shortcut strings are reported as values so
/// the result can be surfaced in settings UI or a validation log.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ShortcutParseError {
    /// The input was empty or contained only whitespace.
    #[error("shortcut string is empty")]
    Empty,
    /// Every token was a modifier; a shortcut needs a non-modifier key.
    #[error("shortcut '{0}' has modifiers but no key")]
    MissingKey(String),
    /// Two tokens resolved to the same modifier flag
    /// (for example `Ctrl+Control`, `Cmd+Meta`, or `Alt+Option`).
    #[error("conflicting modifiers '{0}' and '{1}' resolve to the same flag")]
    ConflictingModifiers(String, String),
    /// More than one non-modifier key token was supplied.
    #[error("shortcut '{0}' has more than one key")]
    MultipleKeys(String),
    /// A `+` separator had no token on one side (for example `Ctrl++S`).
    #[error("malformed shortcut '{0}': empty token around '+'")]
    MalformedSeparator(String),
}

/// A single canonical modifier, recovered from native or web spellings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModifierFlag {
    Shift,
    Ctrl,
    Alt,
    Meta,
}

impl ModifierFlag {
    /// Recognise modifier spellings used by winit physical codes, DOM
    /// `KeyboardEvent.key` values, and the held-key set in `cvkg-inputs`.
    fn from_token(token: &str) -> Option<Self> {
        match token.to_ascii_lowercase().as_str() {
            "shift" | "shiftleft" | "shiftright" | "lshift" | "rshift" => Some(Self::Shift),
            "ctrl" | "control" | "controlleft" | "controlright" | "lctrl" | "rctrl" => {
                Some(Self::Ctrl)
            }
            "alt" | "option" | "opt" | "altleft" | "altright" | "lalt" | "ralt" => Some(Self::Alt),
            "meta" | "cmd" | "command" | "super" | "win" | "windows" | "metaleft" | "metaright"
            | "osleft" | "osright" | "lgui" | "rgui" => Some(Self::Meta),
            _ => None,
        }
    }

    /// Set the corresponding flag on a modifier set.
    fn apply(self, modifiers: &mut KeyModifiers) {
        match self {
            Self::Shift => modifiers.shift = true,
            Self::Ctrl => modifiers.ctrl = true,
            Self::Alt => modifiers.alt = true,
            Self::Meta => modifiers.meta = true,
        }
    }
}

/// Normalise a key token from any platform into one canonical name.
///
/// Accepts winit physical codes (`KeyS` -> `s`, `Digit1` -> `1`), DOM logical
/// keys (`S` -> `s`, `" "` -> `Space`, `Esc` -> `Escape`), and aliases such as
/// `Return`/`Enter` or `Up`/`ArrowUp`. Returns `None` for empty input and for
/// bare modifier keys, which cannot be the key of a shortcut. Unrecognised
/// tokens are returned trimmed and unchanged so custom keys keep working.
pub fn normalize_key_name(name: &str) -> Option<String> {
    // DOM `KeyboardEvent.key` reports a literal single space for the space bar;
    // check it before trimming so it does not collapse into an empty token.
    if name == " " {
        return Some("Space".to_string());
    }
    let trimmed = name.trim();
    if trimmed.is_empty() || ModifierFlag::from_token(trimmed).is_some() {
        return None;
    }

    let canonical = match trimmed.to_ascii_lowercase().as_str() {
        " " | "space" | "spacebar" => Some("Space"),
        "esc" | "escape" => Some("Escape"),
        "del" | "delete" => Some("Delete"),
        "ins" | "insert" => Some("Insert"),
        "return" | "enter" => Some("Enter"),
        "arrowup" | "up" => Some("ArrowUp"),
        "arrowdown" | "down" => Some("ArrowDown"),
        "arrowleft" | "left" => Some("ArrowLeft"),
        "arrowright" | "right" => Some("ArrowRight"),
        _ => None,
    };
    if let Some(canonical) = canonical {
        return Some(canonical.to_string());
    }

    // Single-character keys are case-insensitive: a shortcut authored as
    // "Ctrl+K" must match an incoming "k" and vice versa.
    if trimmed.chars().count() == 1 && trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return Some(trimmed.to_ascii_lowercase());
    }

    // Physical key codes carry a prefix around a single character.
    if let Some(rest) = trimmed
        .strip_prefix("Key")
        .or_else(|| trimmed.strip_prefix("Digit"))
        && rest.chars().count() == 1
        && rest
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
    {
        return Some(rest.to_ascii_lowercase());
    }

    if trimmed.chars().count() == 1
        && trimmed
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
    {
        return Some(trimmed.to_ascii_lowercase());
    }

    Some(trimmed.to_string())
}

/// Reinterpret a shortcut's Ctrl/Command bit using the platform's primary
/// modifier. Bare keys (neither flag set) are left untouched.
fn rebase_primary(modifiers: KeyModifiers, platform: ShortcutPlatform) -> KeyModifiers {
    let has_primary = modifiers.ctrl || modifiers.meta;
    let mut rebased = modifiers;
    match platform {
        ShortcutPlatform::Apple => {
            rebased.ctrl = false;
            rebased.meta = has_primary;
        }
        ShortcutPlatform::Other => {
            rebased.meta = false;
            rebased.ctrl = has_primary;
        }
    }
    rebased
}

impl KeyModifiers {
    /// Returns `true` when no modifier flag is active.
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.meta
    }

    /// Returns how many modifier flags are active (zero to four).
    pub fn count(&self) -> usize {
        [self.shift, self.ctrl, self.alt, self.meta]
            .iter()
            .filter(|active| **active)
            .count()
    }

    /// Returns the platform's primary modifier as a standalone flag set:
    /// `meta` on macOS, `ctrl` everywhere else.
    pub fn primary(platform: ShortcutPlatform) -> Self {
        match platform {
            ShortcutPlatform::Apple => Self {
                meta: true,
                ..Self::default()
            },
            ShortcutPlatform::Other => Self {
                ctrl: true,
                ..Self::default()
            },
        }
    }

    /// Derive modifier flags from held-key names emitted by either backend.
    ///
    /// Native physical codes (`ShiftLeft`, `ControlRight`, `OSLeft`) and web
    /// logical names (`Shift`, `Control`, `Meta`) collapse to the same flags.
    /// Unrecognised names are ignored.
    pub fn from_pressed_keys<'a, I>(keys: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut modifiers = Self::default();
        for key in keys {
            if let Some(flag) = ModifierFlag::from_token(key) {
                flag.apply(&mut modifiers);
            }
        }
        modifiers
    }
}

impl KeyShortcut {
    /// Parse a `+`-separated shortcut such as `Ctrl+Shift+S`, `cmd+k`, or
    /// `Space`. Modifier order is irrelevant and aliases (`Cmd`, `Control`,
    /// `Option`, `Super`) are accepted. The original string becomes the
    /// shortcut's description.
    pub fn parse(input: &str) -> Result<Self, ShortcutParseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ShortcutParseError::Empty);
        }

        let mut modifiers = KeyModifiers::default();
        let mut seen: Vec<(String, ModifierFlag)> = Vec::new();
        let mut key: Option<String> = None;

        for segment in trimmed.split('+') {
            let token = segment.trim();
            if let Some(flag) = ModifierFlag::from_token(token) {
                if let Some((previous, _)) = seen.iter().find(|(_, seen_flag)| *seen_flag == flag) {
                    return Err(ShortcutParseError::ConflictingModifiers(
                        previous.clone(),
                        token.to_string(),
                    ));
                }
                seen.push((token.to_string(), flag));
                flag.apply(&mut modifiers);
            } else {
                let normalized = normalize_key_name(token)
                    .ok_or_else(|| ShortcutParseError::MalformedSeparator(input.to_string()))?;
                if let Some(first) = &key {
                    return Err(ShortcutParseError::MultipleKeys(first.clone()));
                }
                key = Some(normalized);
            }
        }

        let key = key.ok_or_else(|| ShortcutParseError::MissingKey(input.to_string()))?;
        Ok(Self {
            key,
            modifiers,
            description: trimmed.to_string(),
        })
    }

    /// Match an incoming key and modifier set exactly, normalising key names on
    /// both sides. Use this when Ctrl and Command are meaningfully different.
    pub fn matches(&self, key: &str, modifiers: KeyModifiers) -> bool {
        match (normalize_key_name(&self.key), normalize_key_name(key)) {
            (Some(expected), Some(actual)) => expected == actual && self.modifiers == modifiers,
            _ => false,
        }
    }

    /// Match a shortcut while treating its Ctrl/Command bit as the platform's
    /// primary modifier. A shortcut authored once as `Ctrl+S` therefore matches
    /// `Meta+S` on macOS and `Ctrl+S` on Windows/Linux. Shift and Alt are
    /// compared exactly. Use [`Self::matches`] for literal Ctrl/Meta shortcuts.
    pub fn matches_on(
        &self,
        key: &str,
        modifiers: KeyModifiers,
        platform: ShortcutPlatform,
    ) -> bool {
        let expected = rebase_primary(self.modifiers, platform);
        match (normalize_key_name(&self.key), normalize_key_name(key)) {
            (Some(expected_key), Some(actual_key)) => {
                expected_key == actual_key && expected == modifiers
            }
            _ => false,
        }
    }
}

impl std::str::FromStr for KeyShortcut {
    type Err = ShortcutParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_native_and_web_spellings() {
        let native = KeyShortcut::parse("ShiftLeft+KeyS").unwrap();
        assert!(native.modifiers.shift);
        assert_eq!(native.key, "s");

        let web = KeyShortcut::parse("shift+s").unwrap();
        assert_eq!(web.modifiers, native.modifiers);
        assert_eq!(web.key, native.key);
    }

    #[test]
    fn parse_orders_modifiers_independently() {
        let a = KeyShortcut::parse("Ctrl+Alt+Shift+S").unwrap();
        let b = KeyShortcut::parse("s+shift+option+control").unwrap();
        assert_eq!(a.modifiers, b.modifiers);
        assert_eq!(a.key, b.key);
    }

    #[test]
    fn parse_rejects_empty_and_modifier_only() {
        assert_eq!(KeyShortcut::parse("  "), Err(ShortcutParseError::Empty));
        assert_eq!(
            KeyShortcut::parse("Ctrl+Alt"),
            Err(ShortcutParseError::MissingKey("Ctrl+Alt".to_string()))
        );
    }

    #[test]
    fn parse_rejects_conflicting_and_multiple_keys() {
        assert_eq!(
            KeyShortcut::parse("Ctrl+Control+S"),
            Err(ShortcutParseError::ConflictingModifiers(
                "Ctrl".to_string(),
                "Control".to_string()
            ))
        );
        assert_eq!(
            KeyShortcut::parse("Cmd+Meta+S"),
            Err(ShortcutParseError::ConflictingModifiers(
                "Cmd".to_string(),
                "Meta".to_string()
            ))
        );
        assert_eq!(
            KeyShortcut::parse("Alt+Option+S"),
            Err(ShortcutParseError::ConflictingModifiers(
                "Alt".to_string(),
                "Option".to_string()
            ))
        );
        assert_eq!(
            KeyShortcut::parse("S+T"),
            Err(ShortcutParseError::MultipleKeys("s".to_string()))
        );
    }

    #[test]
    fn parse_rejects_malformed_separator() {
        assert_eq!(
            KeyShortcut::parse("Ctrl++S"),
            Err(ShortcutParseError::MalformedSeparator(
                "Ctrl++S".to_string()
            ))
        );
    }

    #[test]
    fn key_normalisation_covers_both_backends() {
        assert_eq!(normalize_key_name("KeyS").as_deref(), Some("s"));
        assert_eq!(normalize_key_name("S").as_deref(), Some("s"));
        assert_eq!(normalize_key_name("Digit1").as_deref(), Some("1"));
        assert_eq!(normalize_key_name(" ").as_deref(), Some("Space"));
        assert_eq!(normalize_key_name("Spacebar").as_deref(), Some("Space"));
        assert_eq!(normalize_key_name("Esc").as_deref(), Some("Escape"));
        assert_eq!(normalize_key_name("Return").as_deref(), Some("Enter"));
        assert_eq!(normalize_key_name("ArrowUp").as_deref(), Some("ArrowUp"));
        assert_eq!(normalize_key_name("F13").as_deref(), Some("F13"));
        assert_eq!(normalize_key_name("ShiftLeft"), None);
        assert_eq!(normalize_key_name("   "), None);
    }

    #[test]
    fn modifiers_from_pressed_keys_are_platform_agnostic() {
        let native = KeyModifiers::from_pressed_keys(["ShiftLeft", "ControlRight", "OSLeft"]);
        let web = KeyModifiers::from_pressed_keys(["Shift", "Control", "Meta"]);
        assert_eq!(native, web);
        assert_eq!(native.count(), 3);
        assert!(!native.is_empty());
        assert!(KeyModifiers::default().is_empty());
    }

    #[test]
    fn matches_exactly_and_across_platforms() {
        let shortcut = KeyShortcut::parse("Ctrl+Shift+S").unwrap();
        let ctrl_shift_s = KeyModifiers {
            ctrl: true,
            shift: true,
            ..KeyModifiers::default()
        };
        let meta_shift_s = KeyModifiers {
            meta: true,
            shift: true,
            ..KeyModifiers::default()
        };

        assert!(shortcut.matches("KeyS", ctrl_shift_s));
        assert!(shortcut.matches("s", ctrl_shift_s));
        assert!(!shortcut.matches("s", meta_shift_s));
        assert!(shortcut.matches_on("s", meta_shift_s, ShortcutPlatform::Apple));
        assert!(shortcut.matches_on("s", ctrl_shift_s, ShortcutPlatform::Other));
        assert!(!shortcut.matches_on("s", ctrl_shift_s, ShortcutPlatform::Apple));
    }

    #[test]
    fn primary_modifier_matches_platform_convention() {
        assert_eq!(KeyModifiers::primary(ShortcutPlatform::Apple).meta, true);
        assert_eq!(KeyModifiers::primary(ShortcutPlatform::Apple).ctrl, false);
        assert_eq!(KeyModifiers::primary(ShortcutPlatform::Other).ctrl, true);
        assert_eq!(KeyModifiers::primary(ShortcutPlatform::Other).meta, false);
        assert_eq!(ShortcutPlatform::default(), ShortcutPlatform::Other);
    }

    #[test]
    fn from_str_delegates_to_parse() {
        let shortcut: KeyShortcut = "cmd+k".parse().unwrap();
        assert!(shortcut.modifiers.meta);
        assert_eq!(shortcut.key, "k");
        assert_eq!(shortcut.description, "cmd+k");
    }
}
