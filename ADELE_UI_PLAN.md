# Adele Design Systems UI Plan

## Executive Summary

This document outlines the UI architecture for an interactive interface to explore the Adele design systems repository. The UI will provide powerful filtering, searching, and comparison capabilities across 100+ design systems from companies like Shopify, Atlassian, Salesforce, and more.

---

## 1. UI Architecture Overview

### 1.1 Target Platform
- **Primary**: Web-based interface (CVKG/WebAssembly target)
- **Secondary**: Desktop app (Tauri/eGUI)
- **Data Source**: `/tmp/adele/src/data/data.json` + individual system JSON files

### 1.2 Core Features
| Feature | Description |
|---------|-------------|
| Design System Catalog | Browse all 100+ design systems |
| Advanced Filtering | Filter by 20+ attributes (tech stack, components, accessibility) |
| Comparison Mode | Side-by-side design system comparison |
| Search & Discovery | Full-text search across system names and attributes |
| Export Capabilities | JSON/CSV export of filtered results |

---

## 2. UI Component Structure

### 2.1 Main Layout Components

```
App
├── Header (Search + Stats)
├── Sidebar (Filters)
├── Main Content
│   ├── System Grid/List View
│   └── System Detail View
└── Footer (Attribution)
```

### 2.2 Filter Panel Structure

**Technology Filters**
- Code Depth: HTML/CSS, HTML/CSS/JS, React, Vue, Angular
- JS Framework: Vanilla, React, Vue, Angular, Preact, Stencil
- TypeScript Support: Yes/No
- Web Components: Yes/No
- CSS Approach: Sass, Less, CSS-in-JS, Styled Components

**Component Filters**
- Components Available: Yes/No
- UI Kit: Yes/No
- Design Tokens: Yes/No
- Tests: Yes/No
- Storybook: Yes/No

**Design Filters**
- Color Palette: Yes/No
- Typography: Yes/No
- Icons: Yes/No
- Space/Grid: Yes/No
- Animation: Yes/No
- Voice/Tone: Yes/No

**Quality & Compliance**
- Accessibility Guidelines: Yes/No
- Design Principles: Yes/No
- Brand Guidelines: Yes/No
- Code Documentation: HTML, MD, Storybook, Other

**Distribution**
- npm, Bower, CDN, GitHub Packages, Other

---

## 3. Data Schema Mapping

### 3.1 Core System Object (Rust)

```rust
use serde::{Deserialize, Serialize};

/// Base attribute structure used throughout design system data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub data: String,
    pub label: Option<String>,
    pub url: Option<String>,
}

/// Deprecated status enum for design systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeprecatedStatus {
    Yes,
    No,
}

/// Main design system data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub css_in_js: Attribute,
    
    // Components
    pub components: Attribute,
    pub ui_kit: Attribute,
    pub design_tokens: Attribute,
    pub bundle_manager: Attribute,
    
    // Design
    pub color_palette: DesignAttribute,
    pub color_naming: Attribute,
    pub contrast_analysis: DesignAttribute,
    pub typography: DesignAttribute,
    pub icons: DesignAttribute,
    pub space_grid: DesignAttribute,
    pub illustrations: Attribute,
    pub data_visualization: Attribute,
    pub animation: DesignAttribute,
    pub voice_tone: Attribute,
    pub accessibility_guidelines: Attribute,
    pub design_principles: Attribute,
    
    // Documentation
    pub website_documentation: Attribute,
    pub code_documentation: CodeDocInfo,
    pub storybook: Attribute,
    
    // Distribution
    pub distribution: DistributionInfo,
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
    pub deprecated: Option<DeprecatedStatus>,
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

/// Code documentation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDocInfo {
    pub data: String,
    pub label: String,
}

/// Distribution methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionInfo {
    pub data: Vec<String>,
}
```

### 3.2 Filter Criteria (Rust)

```rust
/// Filter criteria for design system queries
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    // Technology filters
    pub code_depth: Option<String>,
    pub js_framework: Option<String>,
    pub has_typescript: Option<bool>,
    pub has_web_components: Option<bool>,
    pub css_technology: Option<String>,
    
    // Component filters
    pub has_components: Option<bool>,
    pub has_ui_kit: Option<bool>,
    pub has_design_tokens: Option<bool>,
    pub has_tests: Option<bool>,
    pub has_storybook: Option<bool>,
    
    // Design filters
    pub has_color_palette: Option<bool>,
    pub has_typography: Option<bool>,
    pub has_icons: Option<bool>,
    pub has_space_grid: Option<bool>,
    pub has_animation: Option<bool>,
    
    // Quality & compliance
    pub has_accessibility_guidelines: Option<bool>,
    pub has_design_principles: Option<bool>,
    pub has_brand_guidelines: Option<bool>,
}

impl FilterCriteria {
    pub fn matches(&self, system: &DesignSystem) -> bool {
        let mut matches = true;
        
        if let Some(ref depth) = self.code_depth {
            matches &= system.code_depth.data.contains(depth);
        }
        
        if let Some(has_ts) = self.has_typescript {
            matches &= (system.ts.data == "yes") == has_ts;
        }
        
        if let Some(has_components) = self.has_components {
            matches &= (system.components.data == "yes") == has_components;
        }
        
        if let Some(has_access) = self.has_accessibility_guidelines {
            matches &= (system.accessibility_guidelines.data == "yes") == has_access;
        }
        
        matches
    }
}

---

## 4. View Specifications

### 4.1 Catalog View
- **Display Mode**: Card or Table view toggle
- **Card Content**:
  - Company logo/name
  - System name (with link)
  - Tech stack badges (JS, CSS, TS)
  - Component availability indicator
  - Accessibility status badge

### 4.2 Detail View
- **Tabs**:
  1. Overview (all attributes grouped by category)
  2. Technology Stack
  3. Design System Elements
  4. Documentation Links
  5. Similar Systems (based on filters)

### 4.3 Comparison View
- **Features**:
  - Multi-select systems for comparison
  - Attribute matrix view
  - Export comparison results

---

## 5. Search & Filter Implementation

### 5.1 Search Index
```
Fields to index:
- company.data
- system.data
- repository.url
- All .data fields
- URL fields
```

### 5.2 Filter Logic
```javascript
// Example filter combination
{
  js: "React",
  components: "yes",
  accessibilityGuidelines: "yes",
  storybook: "yes"
}
```

---

## 6. CVKG Integration Points

### 6.1 Component Mapping to CVKG

| Adele Feature | CVKG Component | Module |
|---------------|----------------|--------|
| Card View | `cvkg_components::visual::Card` | cvkg-components |
| Filter Panel | `cvkg_components::container::Tabs` | cvkg-components |
| Search Input | `cvkg_components::interactive::Input` | cvkg-components |
| Badge Display | `cvkg_components::visual::Badge` | cvkg-components |
| Grid Layout | `cvkg_layout` | cvkg-layout |
| Theme Support | `cvkg_themes` | cvkg-themes |
| Virtual Scrolling | `cvkg_components::virtual_list` | cvkg-components |
| Table View | `cvkg_components::virtual_table` | cvkg-components |

### 6.2 Data Loading Integration

```rust
// Pseudo-code for CVKG integration
pub struct AdeleDataSource {
    systems: Vec<DesignSystem>,
}

impl AdeleDataSource {
    pub fn load_from_json(&mut self, path: &Path) -> Result<(), Error> {
        // Load and parse data.json
    }
    
    pub fn filter_by(&self, criteria: &FilterCriteria) -> Vec<DesignSystem> {
        // Apply filters using CVKG's functional patterns
    }
}
```

---

## 7. Technical Implementation Plan

### Phase 1: Core Infrastructure (Week 1)
- [ ] JSON data loader and parser
- [ ] Basic UI shell with routing
- [ ] Static data display

### Phase 2: Filtering & Search (Week 2)
- [ ] Filter panel implementation
- [ ] Search functionality
- [ ] URL state persistence

### Phase 3: Advanced Features (Week 3)
- [ ] Comparison mode
- [ ] Export functionality
- [ ] Responsive design

### Phase 4: CVKG Integration (Week 4)
- [ ] Port to CVKG components
- [ ] Theme integration
- [ ] Performance optimization

---

## 8. UI Wireframes

### 8.1 Main Catalog View
```
+----------------------------------------------------------+
| [Search bar.................] [Stats: 105 systems]       |
+---------------------------+--------------------------------
| FILTERS                   | SYSTEMS                        |
|                           |                                |
| [x] JS Framework          | [Shopify Polaris] [Card]       |
|   [React]                 | Components: Yes | Tokens: Yes   |
|   [Vue]                   | Accessibility: Yes             |
|                           |                                |
| [x] Accessibility         | [Atlassian Design] [Card]    |
|   [Guidelines: Yes]       | Components: Yes | Tokens: Yes   |
|                           | Storybook: Yes                 |
| [x] Components            |                                |
|   [Available: Yes]        | [More systems...]              |
|                           |                                |
+---------------------------+--------------------------------
```

### 8.2 Detail View
```
+----------------------------------------------------------+
| Company: Shopify           System: Polaris              |
+----------------------------------------------------------+
| [Overview] [Technology] [Design] [Docs] [Similar]       |
+----------------------------------------------------------+
| Code Depth: React/Vanilla                               |
| JS Framework: React                                     |
| TypeScript: No                                          |
| CSS: Sass                                               |
| Web Components: No                                      |
|                                                           |
| Components: Yes    | Design Tokens: Yes                  |
| UI Kit: Yes        | Tests: Yes                          |
| Storybook: Yes     | Bundle Manager: No                   |
+----------------------------------------------------------+
| Links:                                                  |
| - Website: https://polaris.shopify.com                 |
| - Repository: https://github.com/Shopify/polaris     |
| - Storybook: https://storybook.shopify.design           |
+----------------------------------------------------------+
```

---

## 9. Performance Considerations

- Implement virtual scrolling for large dataset display
- Use indexedDB/localStorage for cached data
- Lazy-load system detail views
- Debounce search input (300ms)
- Memoize filter results

---

## 10. Analytics & Insights

### 10.1 Potential Visualizations
- Most common tech stacks
- Accessibility adoption rate
- Storybook adoption trends
- Component availability correlation

### 10.2 Export Formats
- JSON (full system data)
- CSV (filtered results)
- Markdown (comparison report)

---

*Generated: 2026-04-30*
*Adele Repository: https://github.com/UXPin/adele*
