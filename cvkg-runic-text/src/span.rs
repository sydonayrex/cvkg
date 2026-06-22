use crate::style::TextStyle;

// ── TextSpan ─────────────────────────────────────────────────────────────────

/// Vertical alignment strategies for inline UI portals within a text line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PortalAlignment {
    /// Align the bottom of the portal box to the text baseline.
    #[default]
    Baseline,
    /// Align the top of the portal box to the top of the line height.
    Top,
    /// Center the portal box vertically within the line height.
    Center,
    /// Align the bottom of the portal box to the bottom of the line height.
    Bottom,
}

/// Identifies the layout behavior of a TextSpan (standard text vs inline portal).
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TextSpanKind {
    /// Standard text flow.
    #[default]
    Text,
    /// An inline interactive widget box.
    Portal {
        /// Width of the portal box in pixels.
        width: f32,
        /// Height of the portal box in pixels.
        height: f32,
        /// Vertical alignment mode.
        alignment: PortalAlignment,
        /// Unique identifier for downstream portal instantiation.
        id: String,
    },
}

/// A span of text or an inline UI portal with associated styling.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan {
    /// The text content (stores "\u{FFFC}" object placeholder for portals).
    pub text: String,
    /// The style to apply.
    pub style: TextStyle,
    /// Byte offset in the full text where this span starts.
    pub byte_offset: usize,
    /// Layout category of the span.
    pub kind: TextSpanKind,
}

impl TextSpan {
    /// Create a new text span.
    pub fn new(text: &str, style: TextStyle) -> Self {
        TextSpan {
            text: text.to_string(),
            style,
            byte_offset: 0,
            kind: TextSpanKind::Text,
        }
    }

    /// Create a new text span at a specific byte offset.
    pub fn at(text: &str, style: TextStyle, byte_offset: usize) -> Self {
        TextSpan {
            text: text.to_string(),
            style,
            byte_offset,
            kind: TextSpanKind::Text,
        }
    }

    /// Create a new inline UI portal span.
    pub fn portal(
        width: f32,
        height: f32,
        alignment: PortalAlignment,
        id: &str,
        style: TextStyle,
    ) -> Self {
        TextSpan {
            text: "\u{FFFC}".to_string(),
            style,
            byte_offset: 0,
            kind: TextSpanKind::Portal {
                width,
                height,
                alignment,
                id: id.to_string(),
            },
        }
    }

    /// Create a new inline UI portal span at a specific byte offset.
    pub fn portal_at(
        width: f32,
        height: f32,
        alignment: PortalAlignment,
        id: &str,
        style: TextStyle,
        byte_offset: usize,
    ) -> Self {
        TextSpan {
            text: "\u{FFFC}".to_string(),
            style,
            byte_offset,
            kind: TextSpanKind::Portal {
                width,
                height,
                alignment,
                id: id.to_string(),
            },
        }
    }
}

// ── Text Semantic Layer (P0-42) ──────────────────────────────────────────────

/// A styled range of text representing a contiguous semantic block within a document.
///
/// Under UAX #29 and accessibility guidelines, this represents a span of text that shares
/// the same style and semantic properties, mapping directly to screen reader text offsets.
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    /// The start index of this run in the parent string.
    pub start: usize,
    /// The end index of this run in the parent string.
    pub end: usize,
    /// The text content of this run.
    pub text: String,
    /// The style applied to this run.
    pub style: TextStyle,
}

impl TextRun {
    /// Create a new TextRun.
    pub fn new(start: usize, end: usize, text: &str, style: TextStyle) -> Self {
        Self {
            start,
            end,
            text: text.to_string(),
            style,
        }
    }
}

/// Enumerates the standard semantic categories of text ranges for platform accessibility mappings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticKind {
    /// Standard plain text body.
    Normal,
    /// Header/title element (level 1-6).
    Header(u8),
    /// A hyperlink URL node.
    Link,
    /// Strong emphasis/bold text block.
    Emphasis,
    /// Inline code or syntax block.
    Code,
    /// List item element.
    ListItem,
}

/// Defines a semantic range over text to expose structural meaning to platform accessibility APIs.
///
/// Matches AXTextMarkerRange and UIAutomation text range concepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticRange {
    /// Start character/byte index in the text document.
    pub start: usize,
    /// End character/byte index in the text document.
    pub end: usize,
    /// The accessibility category of this range.
    pub kind: SemanticKind,
    /// Optional payload data (e.g. the target URL for a Link).
    pub data: Option<String>,
}

impl SemanticRange {
    /// Create a new SemanticRange.
    pub fn new(start: usize, end: usize, kind: SemanticKind, data: Option<String>) -> Self {
        Self {
            start,
            end,
            kind,
            data,
        }
    }
}

/// A block-level text paragraph exposing semantic structure and style spans to screen readers.
///
/// Paragraphs are the foundational unit for platform accessibility navigators (e.g. AXParagraph, AXStaticText).
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// Raw concatenated string text of the paragraph.
    pub text: String,
    /// Ordered styled text runs.
    pub runs: Vec<TextRun>,
    /// High-level semantic markers for accessibility indexing.
    pub semantic_ranges: Vec<SemanticRange>,
}

impl Paragraph {
    /// Create a new paragraph with empty spans.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            runs: Vec::new(),
            semantic_ranges: Vec::new(),
        }
    }

    /// Add a styled text run to the paragraph.
    pub fn add_run(&mut self, run: TextRun) {
        self.runs.push(run);
    }

    /// Add an accessibility semantic range marker.
    pub fn add_semantic_range(&mut self, range: SemanticRange) {
        self.semantic_ranges.push(range);
    }
}
