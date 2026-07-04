//! Headless SSR Mode (Bevy's MinimalPlugins analog).
//!
//! `CvkgHeadless` provides a minimal backend that runs the frame pipeline
//! (State -> Layout -> Animation -> Render) **without** a GPU, window, or input
//! devices. It is the CVKG equivalent of Bevy's `MinimalPlugins` -- useful for
//! CI snapshot testing, SSR dashboards, and accessibility tree export.
//!
//! ## Example
//! ```text
//! let view = Page::new();
//! let mut headless = CvkgHeadless::new(view, Rect::sized(1920, 1080));
//! let frame = headless.render_frame();
//! ```

use cvkg_core::{FramePhase, Rect, View};
use cvkg_layout::AnimationEngine;
use cvkg_vdom::{VDom, VNodeRenderer};
use std::collections::HashMap;

/// Output of one headless frame. SVG-ready when full css/svg pipeline is wired.
#[derive(Debug, Clone)]
pub struct HeadlessFrame {
    /// SVG XML string (empty if no svg-serialize hookup).
    pub svg: String,
    /// Root VDom node id.
    pub root: Option<cvkg_vdom::NodeId>,
    /// Telemetry: phases flushed, JSON-like map.
    pub telemetry: HashMap<String, String>,
}

/// Minimal headless CVKG backend: VDom + layout + AnimationEngine,
/// no GPU context, no window, no input devices.
pub struct CvkgHeadless {
    /// The viewport rectangle.
    viewport: Rect,
    /// Frame scheduler.
    scheduler: cvkg_scheduler::FrameScheduler,
    /// Built VDom from the latest frame.
    vdom: VDom,
    /// Animation engine.
    animation: AnimationEngine,
    /// Stashed handler output.
    prev_root_hash: Option<u64>,
    /// Optional theme name for downstream rendering.
    theme: Option<String>,
}

/// Builder-style options for `CvkgHeadless`.
#[derive(Debug, Clone, Default)]
pub struct HeadlessOptions {
    /// Optional theme name (e.g. "dark", "light", "high-contrast").
    pub theme: Option<String>,
}

impl CvkgHeadless {
    /// Build a headless instance from any `View`.
    pub fn new(view: impl View + 'static, viewport: Rect) -> Self {
        Self::with_options(view, viewport, HeadlessOptions::default())
    }

    /// Build a headless instance with theme override.
    pub fn with_theme(view: impl View + 'static, viewport: Rect, theme: impl Into<String>) -> Self {
        let mut options = HeadlessOptions::default();
        options.theme = Some(theme.into());
        Self::with_options(view, viewport, options)
    }

    /// Build a headless instance with options.
    pub fn with_options(view: impl View + 'static, viewport: Rect, options: HeadlessOptions) -> Self {
        let vdom = {
            let mut r = VNodeRenderer::new();
            view.render(&mut r, viewport);
            r.into_vdom()
        };
        Self {
            viewport,
            scheduler: cvkg_scheduler::FrameScheduler::new(),
            vdom,
            animation: AnimationEngine::new(),
            prev_root_hash: None,
            theme: options.theme,
        }
    }

    /// Return the built VDom.
    pub fn vdom(&self) -> &VDom {
        &self.vdom
    }

    /// Return the viewport rectangle.
    pub fn viewport(&self) -> Rect {
        self.viewport
    }

    /// Return the active theme name, if set.
    pub fn theme(&self) -> Option<&str> {
        self.theme.as_deref()
    }

    /// Reset the AnimationEngine and VDom.
    pub fn reset(&mut self) {
        self.scheduler = cvkg_scheduler::FrameScheduler::new();
        self.animation = AnimationEngine::new();
        self.prev_root_hash = None;
    }

    /// Run one frame through State -> Layout -> Animation -> Render (no GPU).
    pub fn render_frame(&mut self) -> HeadlessFrame {
        let mut telemetry = HashMap::new();

        self.scheduler.begin_frame();
        telemetry.insert(
            "viewport".into(),
            format!(
                "{}x{} at ({},{})",
                self.viewport.width, self.viewport.height, self.viewport.x, self.viewport.y
            ),
        );

        let mut phase_count = 0;
        loop {
            self.scheduler.flush_current_phase();
            let next = self.scheduler.advance_phase();
            phase_count += 1;
            if next == FramePhase::PostFrame || phase_count > 12 {
                break;
            }
        }
        telemetry.insert("phases_flushed".into(), format!("{phase_count}"));
        telemetry.insert(
            "vdom_node_count".into(),
            format!("{}", self.vdom.nodes.len()),
        );

        if let Some(name) = &self.theme {
            telemetry.insert("theme".into(), name.clone());
        }

        if let Some(root_id) = self.vdom.root {
            self.prev_root_hash = Some(root_id.0);
        }

        let svg = self.build_minimal_svg();

        HeadlessFrame {
            svg,
            root: self.vdom.root,
            telemetry,
        }
    }

    /// Build a minimal placeholder SVG describing the VDom for snapshot tests.
    /// Full svg-serialize integration (usvg::Tree) is out of scope for this
    /// minimal scaffold and will be wired when `cvkg-svg-serialize` exposes
    /// a `from_vdom` API.
    fn build_minimal_svg(&self) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        let _ = writeln!(s, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        let _ = writeln!(
            s,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
            self.viewport.width as u32,
            self.viewport.height as u32,
            self.viewport.width as u32,
            self.viewport.height as u32
        );
        if let Some(root) = self.vdom.root {
            if let Some(node) = self.vdom.nodes.get(&root) {
                let attr = escape_attr(&node.component_type);
                let _ = writeln!(
                    s,
                    "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"gray\" data-comp=\"{}\"/>",
                    node.layout.x,
                    node.layout.y,
                    node.layout.width,
                    node.layout.height,
                    attr
                );
            }
        }
        s.push_str("</svg>\n");
        s
    }
}

fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&"),
            '<' => out.push_str("<"),
            '>' => out.push_str(">"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use cvkg_core::Renderer;

    #[test]
    fn test_headless_create() {
        let view = EmptyView;
        let mut h = CvkgHeadless::new(view, Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        });
        let frame = h.render_frame();
        assert!(!frame.telemetry.is_empty());
        assert!(frame.telemetry.contains_key("phases_flushed"));
        assert!(frame.telemetry.contains_key("vdom_node_count"));
        assert!(frame.svg.contains("<svg "));
    }

    #[test]
    fn test_headless_with_theme() {
        let view = EmptyView;
        let h = CvkgHeadless::with_theme(view, Rect::zero(), "dark");
        assert_eq!(h.theme(), Some("dark"));
    }

    #[test]
    fn test_headless_reset() {
        let view = EmptyView;
        let mut h = CvkgHeadless::new(view, Rect::zero());
        h.render_frame();
        h.reset();
    }

    #[test]
    fn test_headless_options_default() {
        let opts = HeadlessOptions::default();
        assert!(opts.theme.is_none());
    }

    #[test]
    fn test_escape_attr_specials() {
        assert_eq!(escape_attr("a&b"), "a&b");
        assert_eq!(escape_attr("a<b>c"), "a<b>c");
        let input = r#"a"b"#;
        let expected = "a\\\"b";
        assert_eq!(escape_attr(input), expected);
    }

    /// Empty view for basic test.
    struct EmptyView;
    impl View for EmptyView {
        type Body = cvkg_core::Never;
        fn body(self) -> Self::Body {
            unreachable!()
        }
        fn render(&self, _renderer: &mut dyn Renderer, _rect: Rect) -> () {
            ()
        }
    }
}