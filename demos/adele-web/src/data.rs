#![allow(clippy::collapsible_if)]
use serde::{Deserialize, Serialize};

/// Base attribute structure used throughout design system data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub data: String,
    pub label: Option<String>,
    pub url: Option<String>,
}

/// Status enum for design systems (handles "yes", "no", and null)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Yes,
    No,
}

/// Main design system data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystem {
    pub company: CompanyInfo,
    pub system: SystemInfo,
    pub repository: RepositoryInfo,

    // Technology
    pub code_depth: Attribute,
    pub js: JsInfo,
    pub ts: Attribute,
    pub web_components: Attribute,
    pub css: CssInfo,

    // Components
    pub components: Attribute,
    pub ui_kit: Attribute,
    pub design_tokens: Attribute,

    // Design
    pub color_palette: DesignAttribute,
    pub typography: DesignAttribute,
    pub icons: DesignAttribute,
    pub animation: DesignAttribute,
    pub accessibility_guidelines: Attribute,
    pub design_principles: Attribute,

    // Documentation
    pub website_documentation: Option<Attribute>,
    pub storybook: Attribute,
}

/// Company information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub data: String,
    pub label: String,
}

/// System information with deprecation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub data: String,
    pub deprecated: Option<String>,
    pub url: String,
    pub label: String,
    #[serde(rename = "$addedAt")]
    pub added_at: String,
}

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub data: String,
    pub url: String,
    pub label: String,
}

/// JavaScript framework information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsInfo {
    pub data: String,
    pub label: String,
}

/// CSS technology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssInfo {
    pub data: String,
    pub label: String,
}

/// Design attribute with optional URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignAttribute {
    pub data: String,
    pub url: Option<String>,
}

/// Filter criteria for design system queries
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    pub search_query: String,
    pub code_depth: Option<String>,
    pub js_framework: Option<String>,
    pub has_typescript: Option<bool>,
    pub has_components: Option<bool>,
    pub has_accessibility_guidelines: Option<bool>,
}

impl FilterCriteria {
    pub fn matches(&self, system: &DesignSystem) -> bool {
        // Search filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            if !system.company.data.to_lowercase().contains(&query)
                && !system.system.data.to_lowercase().contains(&query)
            {
                return false;
            }
        }

        // Tech filters
        if let Some(ref depth) = self.code_depth {
            if !system.code_depth.data.contains(depth) {
                return false;
            }
        }

        if let Some(has_ts) = self.has_typescript {
            let system_has_ts = system.ts.data.to_lowercase() == "yes";
            if system_has_ts != has_ts {
                return false;
            }
        }

        if let Some(has_components) = self.has_components {
            let system_has_comp = system.components.data.to_lowercase() == "yes";
            if system_has_comp != has_components {
                return false;
            }
        }

        if let Some(has_access) = self.has_accessibility_guidelines {
            let system_has_access = system.accessibility_guidelines.data.to_lowercase() == "yes";
            if system_has_access != has_access {
                return false;
            }
        }

        if let Some(ref js) = self.js_framework {
            if !system.js.data.contains(js) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_system(company: &str, system: &str, has_ts: bool) -> DesignSystem {
        let attr = |s: &str| Attribute {
            data: s.to_string(),
            label: None,
            url: None,
        };
        DesignSystem {
            company: CompanyInfo {
                data: company.to_string(),
                label: company.to_string(),
            },
            system: SystemInfo {
                data: system.to_string(),
                deprecated: None,
                url: "".to_string(),
                label: "".to_string(),
                added_at: "".to_string(),
            },
            repository: RepositoryInfo {
                data: "".to_string(),
                url: "".to_string(),
                label: "".to_string(),
            },
            code_depth: attr(""),
            js: JsInfo {
                data: "".to_string(),
                label: "".to_string(),
            },
            ts: attr(if has_ts { "yes" } else { "no" }),
            web_components: attr(""),
            css: CssInfo {
                data: "".to_string(),
                label: "".to_string(),
            },
            components: attr("yes"),
            ui_kit: attr(""),
            design_tokens: attr(""),
            color_palette: DesignAttribute {
                data: "".to_string(),
                url: None,
            },
            typography: DesignAttribute {
                data: "".to_string(),
                url: None,
            },
            icons: DesignAttribute {
                data: "".to_string(),
                url: None,
            },
            animation: DesignAttribute {
                data: "".to_string(),
                url: None,
            },
            accessibility_guidelines: attr("yes"),
            design_principles: attr(""),
            website_documentation: None,
            storybook: attr(""),
        }
    }

    #[test]
    fn test_filter_matches() {
        let system = mock_system("Google", "Material", true);

        let mut criteria = FilterCriteria::default();
        assert!(criteria.matches(&system));

        criteria.search_query = "Material".to_string();
        assert!(criteria.matches(&system));

        criteria.search_query = "Apple".to_string();
        assert!(!criteria.matches(&system));

        criteria.search_query = "".to_string();
        criteria.has_typescript = Some(true);
        assert!(criteria.matches(&system));

        criteria.has_typescript = Some(false);
        assert!(!criteria.matches(&system));
    }
}
