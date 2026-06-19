// ── Accessibility Conformance Tests (P1-44/P1-45) ────────────────────────────
//
// Validates CVKG's accessibility model against platform protocols:
// UIAutomation (Windows), VoiceOver (macOS/iOS), AT-SPI (Linux), ARIA (web).

use std::collections::{HashMap, HashSet};

/// Platform accessibility protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum A11yProtocol {
    /// Microsoft UI Automation (Windows).
    UIAutomation,
    /// Apple Accessibility API (macOS/iOS).
    VoiceOver,
    /// AT-SPI (Linux).
    AT_SPI,
    /// WAI-ARIA (web).
    ARIA,
}

impl A11yProtocol {
    /// All supported protocols.
    pub fn all() -> &'static [A11yProtocol] {
        &[
            A11yProtocol::UIAutomation,
            A11yProtocol::VoiceOver,
            A11yProtocol::AT_SPI,
            A11yProtocol::ARIA,
        ]
    }

    /// Returns true if this protocol supports live regions.
    pub fn supports_live_regions(&self) -> bool {
        match self {
            A11yProtocol::UIAutomation => true,
            A11yProtocol::VoiceOver => true,
            A11yProtocol::AT_SPI => true,
            A11yProtocol::ARIA => true,
        }
    }

    /// Returns true if this protocol supports custom actions.
    pub fn supports_custom_actions(&self) -> bool {
        match self {
            A11yProtocol::UIAutomation => true,
            A11yProtocol::VoiceOver => true,
            A11yProtocol::AT_SPI => false,
            A11yProtocol::ARIA => false,
        }
    }
}

/// CVKG accessibility role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum A11yRole {
    Button,
    Checkbox,
    Radio,
    Slider,
    TextInput,
    Label,
    Heading,
    Link,
    List,
    ListItem,
    Table,
    Row,
    Cell,
    Tree,
    TreeItem,
    Dialog,
    Menu,
    MenuItem,
    Tab,
    TabPanel,
    ProgressBar,
    Unknown,
}

/// Platform-specific role mapping.
pub struct RoleMapping {
    /// CVKG role.
    pub role: A11yRole,
    /// Protocol-specific role name.
    pub protocol_role: HashMap<A11yProtocol, &'static str>,
}

impl RoleMapping {
    /// Create a new role mapping with the given protocol roles.
    pub fn new(role: A11yRole) -> Self {
        Self {
            role,
            protocol_role: HashMap::new(),
        }
    }

    /// Set the role name for a specific protocol.
    pub fn with_protocol(mut self, protocol: A11yProtocol, name: &'static str) -> Self {
        self.protocol_role.insert(protocol, name);
        self
    }

    /// Get the role name for a specific protocol.
    pub fn for_protocol(&self, protocol: A11yProtocol) -> Option<&str> {
        self.protocol_role.get(&protocol).copied()
    }
}

/// Accessibility conformance test case.
#[derive(Debug)]
pub struct A11yConformanceTest {
    /// Name of the test.
    pub name: &'static str,
    /// The CVKG role being tested.
    pub role: A11yRole,
    /// Expected role name per protocol.
    pub expected: HashMap<A11yProtocol, &'static str>,
}

/// Accessibility conformance suite.
pub struct A11yConformanceSuite {
    tests: Vec<A11yConformanceTest>,
}

impl A11yConformanceSuite {
    pub fn new() -> Self {
        Self { tests: Vec::new() }
    }

    /// Register a test case.
    pub fn register(&mut self, test: A11yConformanceTest) {
        self.tests.push(test);
    }

    /// Run all tests. Returns (name, passed) pairs.
    pub fn run_all(&self) -> Vec<(&str, bool)> {
        let mappings = Self::role_mappings();
        self.tests
            .iter()
            .map(|test| {
                let passed = mappings.iter().any(|m| {
                    m.role == test.role
                        && test.expected.iter().all(|(proto, expected_name)| {
                            m.for_protocol(*proto) == Some(*expected_name)
                        })
                });
                (test.name, passed)
            })
            .collect()
    }

    /// Built-in role mappings for common roles.
    fn role_mappings() -> Vec<RoleMapping> {
        vec![
            RoleMapping::new(A11yRole::Button)
                .with_protocol(A11yProtocol::UIAutomation, "Button")
                .with_protocol(A11yProtocol::VoiceOver, "AXButton")
                .with_protocol(A11yProtocol::AT_SPI, "push_button")
                .with_protocol(A11yProtocol::ARIA, "button"),
            RoleMapping::new(A11yRole::Checkbox)
                .with_protocol(A11yProtocol::UIAutomation, "CheckBox")
                .with_protocol(A11yProtocol::VoiceOver, "AXCheckbox")
                .with_protocol(A11yProtocol::AT_SPI, "check_box")
                .with_protocol(A11yProtocol::ARIA, "checkbox"),
            RoleMapping::new(A11yRole::TextInput)
                .with_protocol(A11yProtocol::UIAutomation, "Edit")
                .with_protocol(A11yProtocol::VoiceOver, "AXTextField")
                .with_protocol(A11yProtocol::AT_SPI, "text")
                .with_protocol(A11yProtocol::ARIA, "textbox"),
        ]
    }
}

impl Default for A11yConformanceSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate that an accessibility tree has required properties.
pub struct A11yValidator;

impl A11yValidator {
    /// Check that all interactive elements have accessible names.
    pub fn validate_accessible_names(
        node_names: &HashMap<u64, String>,
        interactive_nodes: &[u64],
    ) -> Vec<u64> {
        interactive_nodes
            .iter()
            .filter(|id| {
                node_names
                    .get(id)
                    .map(|n| n.is_empty())
                    .unwrap_or(true)
            })
            .copied()
            .collect()
    }

    /// Check that heading levels are sequential.
    pub fn validate_heading_levels(levels: &[u64]) -> bool {
        if levels.is_empty() {
            return true;
        }
        // First heading must be h1 or h2
        if levels[0] > 2 {
            return false;
        }
        // No level should skip (e.g., h1 -> h4)
        levels.windows(2).all(|w| w[1] <= w[0] + 1)
    }

    /// Check that all form fields have associated labels.
    pub fn validate_form_labels(
        form_fields: &[u64],
        labels: &HashMap<u64, Vec<u64>>, // field_id -> label_ids
    ) -> Vec<u64> {
        form_fields
            .iter()
            .filter(|id| {
                labels
                    .get(id)
                    .map(|l| l.is_empty())
                    .unwrap_or(true)
            })
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod p1_44_45_a11y_tests {
    use super::*;

    #[test]
    fn protocols_all_supported() {
        assert_eq!(A11yProtocol::all().len(), 4);
    }

    #[test]
    fn all_protocols_support_live_regions() {
        for protocol in A11yProtocol::all() {
            assert!(protocol.supports_live_regions());
        }
    }

    #[test]
    fn uia_supports_custom_actions() {
        assert!(A11yProtocol::UIAutomation.supports_custom_actions());
        assert!(A11yProtocol::VoiceOver.supports_custom_actions());
        assert!(!A11yProtocol::AT_SPI.supports_custom_actions());
    }

    #[test]
    fn role_mapping_button() {
        let mapping = RoleMapping::new(A11yRole::Button)
            .with_protocol(A11yProtocol::UIAutomation, "Button");

        assert_eq!(
            mapping.for_protocol(A11yProtocol::UIAutomation),
            Some("Button")
        );
        assert_eq!(mapping.for_protocol(A11yProtocol::VoiceOver), None);
    }

    #[test]
    fn conformance_suite_runs() {
        let mut suite = A11yConformanceSuite::new();
        suite.register(A11yConformanceTest {
            name: "button_uia",
            role: A11yRole::Button,
            expected: {
                let mut m = HashMap::new();
                m.insert(A11yProtocol::UIAutomation, "Button");
                m
            },
        });
        let results = suite.run_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].1); // passed
    }

    #[test]
    fn conformance_suite_detects_failure() {
        let mut suite = A11yConformanceSuite::new();
        suite.register(A11yConformanceTest {
            name: "button_wrong",
            role: A11yRole::Button,
            expected: {
                let mut m = HashMap::new();
                m.insert(A11yProtocol::UIAutomation, "WrongName");
                m
            },
        });
        let results = suite.run_all();
        assert_eq!(results.len(), 1);
        assert!(!results[0].1); // failed
    }

    #[test]
    fn validate_accessible_names_catches_empty() {
        let mut names: HashMap<u64, String> = HashMap::new();
        names.insert(1, "OK".to_string());
        names.insert(2, "".to_string());
        let interactive = vec![1, 2];
        let violations = A11yValidator::validate_accessible_names(&names, &interactive);
        assert_eq!(violations, vec![2]);
    }

    #[test]
    fn validate_heading_levels_ok() {
        assert!(A11yValidator::validate_heading_levels(&[1, 2, 2, 3]));
        assert!(A11yValidator::validate_heading_levels(&[2, 3]));
        assert!(A11yValidator::validate_heading_levels(&[]));
    }

    #[test]
    fn validate_heading_levels_rejects_bad() {
        assert!(!A11yValidator::validate_heading_levels(&[1, 4])); // skip
        assert!(!A11yValidator::validate_heading_levels(&[3])); // doesn't start at 1 or 2
    }
}
