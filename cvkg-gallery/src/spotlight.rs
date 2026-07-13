//! Spotlight search integration — connects component registry to MimirSpotlight.

use crate::state::{ComponentMeta, GalleryState, GalleryTheme, Registry, ViewportScale, CanvasBackground};
use cvkg_components::MimirSpotlight;
use cvkg_core::{Rect, Renderer, View, Size, SizeProposal};
use std::sync::{Arc, Mutex};

/// Spotlight panel that searches components and commands.
#[derive(Clone)]
pub struct GallerySpotlight {
    state: Arc<Mutex<GalleryState>>,
}

impl GallerySpotlight {
    pub fn new(state: Arc<Mutex<GalleryState>>) -> Self {
        Self { state }
    }

    fn build_scored_components(&self) -> Vec<(ComponentMeta, u32)> {
        let state = self.state.lock().unwrap();
        let query = &state.search_text;
        let all_components = Registry::all();
        
        let mut scored: Vec<_> = all_components
            .iter()
            .filter_map(|meta| {
                let score = fuzzy_match(&meta.name, query);
                if score > 0 || query.is_empty() {
                    Some((meta.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| a.0.name.cmp(b.0.name))
        });

        scored
    }

    fn parse_action(&self, input: &str) -> Option<Box<dyn Fn() + Send + Sync + 'static>> {
            let input = input.trim();
            if !input.starts_with('>') {
                return None;
            }
            let cmd = input[1..].trim();
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }
        
            let state = self.state.clone();
        
            match parts[0].to_lowercase().as_str() {
                "theme" if parts.len() >= 2 => {
                    let theme = match parts[1].to_lowercase().as_str() {
                        "dark" => GalleryTheme::Dark,
                        "light" => GalleryTheme::Light,
                        "highcontrast" | "high-contrast" => GalleryTheme::HighContrast,
                        _ => return None,
                    };
                    Some(Box::new(move || {
                        let mut s = state.lock().unwrap();
                        s.theme = theme;
                    }))
                }
                "scale" if parts.len() >= 2 => {
                    let scale_str = parts[1].trim_end_matches('%');
                    if let Ok(v) = scale_str.parse::<u32>() {
                        let scale = match v {
                            50 => ViewportScale::Percent50,
                            75 => ViewportScale::Percent75,
                            100 => ViewportScale::Percent100,
                            150 => ViewportScale::Percent150,
                            200 => ViewportScale::Percent200,
                            _ => return None,
                        };
                        Some(Box::new(move || {
                            let mut s = state.lock().unwrap();
                            s.scale = scale;
                        }))
                    } else {
                        None
                    }
                }
                "bg" | "background" if parts.len() >= 2 => {
                    let bg = match parts[1].to_lowercase().as_str() {
                        "light" => CanvasBackground::Light,
                        "dark" => CanvasBackground::Dark,
                        "checkered" => CanvasBackground::Checkered,
                        "transparent" => CanvasBackground::Transparent,
                        _ => return None,
                    };
                    Some(Box::new(move || {
                        let mut s = state.lock().unwrap();
                        s.canvas_bg = bg;
                    }))
                }
                "viewport" | "preset" if parts.len() >= 2 => {
                    let preset = match parts[1].to_lowercase().as_str() {
                        "desktop" => crate::state::ViewportPreset::Desktop,
                        "tablet" => crate::state::ViewportPreset::Tablet,
                        "mobile" => crate::state::ViewportPreset::Mobile,
                        _ => return None,
                    };
                    Some(Box::new(move || {
                        let mut s = state.lock().unwrap();
                        s.viewport = preset;
                    }))
                }
                "sidebar" if parts.len() >= 2 => {
                    if let Ok(w) = parts[1].parse::<f32>() {
                        Some(Box::new(move || {
                            let mut s = state.lock().unwrap();
                            s.sidebar_width = w.clamp(200.0, 400.0);
                        }))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }

impl View for GallerySpotlight {
    type Body = cvkg_core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        // Build MimirSpotlight with filtered commands
        let search_text = {
            let state = self.state.lock().unwrap();
            state.search_text.clone()
        };
        let mut spotlight = MimirSpotlight::new().search(&search_text);

        // Add action commands when search starts with >
        let action = self.parse_action(&search_text);
        if let Some(action_fn) = action {
            let search_clone = search_text.clone();
            let state = self.state.clone();
            spotlight = spotlight.command(
                &format!("Action: {}", search_clone),
                Some("Execute this action"),
                move || {
                    action_fn();
                    let mut s = state.lock().unwrap();
                    s.spotlight_open = false;
                    s.search_text.clear();
                    s.spotlight_selected = 0;
                },
            );
        }

        // Add filtered components
        for (meta, _) in self.build_scored_components() {
            let name = meta.name.to_string();
            let state = self.state.clone();
            spotlight = spotlight.command(
                &format!("{} — {}", meta.name, meta.category),
                Some(meta.description),
                move || {
                    let mut s = state.lock().unwrap();
                    s.select_component(&name);
                    s.spotlight_open = false;
                    s.search_text.clear();
                    s.spotlight_selected = 0;
                },
            );
        }

        spotlight.render(renderer, rect);
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size { width: proposal.width.unwrap_or(600.0), height: proposal.height.unwrap_or(400.0) }
    }
}

impl cvkg_core::layout::LayoutView for GallerySpotlight {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) -> Size {
        Size { width: proposal.width.unwrap_or(600.0), height: proposal.height.unwrap_or(400.0) }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn cvkg_core::layout::LayoutView],
        _cache: &mut cvkg_core::layout::LayoutCache,
    ) {
        // Leaf
    }
}

/// Fuzzy match a component name against query string.
fn fuzzy_match(label: &str, query: &str) -> u32 {
    if query.is_empty() {
        return 1;
    }
    let label_lower = label.to_lowercase();
    let query_lower = query.to_lowercase();

    // Simple substring match first
    if label_lower.contains(&query_lower) {
        if label_lower.starts_with(&query_lower) {
            return 100;
        }
        for word in label_lower.split(|c: char| !c.is_alphanumeric()) {
            if word.starts_with(&query_lower) {
                return 80;
            }
        }
        return 50;
    }

    // Character-by-character fuzzy match
    let mut qi = 0;
    let query_chars: Vec<char> = query_lower.chars().collect();
    for ch in label_lower.chars() {
        if qi < query_chars.len() && ch == query_chars[qi] {
            qi += 1;
        }
    }
    if qi == query_chars.len() {
        return 25;
    }
    0
}