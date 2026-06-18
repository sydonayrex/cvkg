//! PromptForge -- Prompt Template Editor with variable binding UI.
//!
//! Provides a prompt template editor with {{variable}} placeholders and
//! a side panel for filling in variable values.

use crate::theme;
use cvkg_core::{Never, Rect, Renderer, View};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A prompt template with {{variable}} placeholders.
///
/// # Syntax:
/// - `{{name}}` -- required variable (highlighted in accent color)
/// - `{{name:default}}` -- variable with default value (highlighted in muted color)
#[derive(Clone)]
pub struct PromptForge {
    template: String,
    variables: Arc<Mutex<HashMap<String, String>>>,
    segments: Vec<ForgeSegment>,
}

#[derive(Debug, Clone)]
pub enum ForgeSegment {
    Text(String),
    Variable {
        name: String,
        default: Option<String>,
    },
}

impl PromptForge {
    pub fn new(template: impl Into<String>) -> Self {
        let template = template.into();
        let segments = Self::parse(&template);
        Self {
            template,
            variables: Arc::new(Mutex::new(HashMap::new())),
            segments,
        }
    }

    /// Set a variable value.
    pub fn set_variable(&self, name: &str, value: impl Into<String>) {
        if let Ok(mut vars) = self.variables.lock() {
            vars.insert(name.to_string(), value.into());
        }
    }

    /// Get a variable value.
    pub fn get_variable(&self, name: &str) -> Option<String> {
        self.variables
            .lock()
            .ok()
            .and_then(|v| v.get(name).cloned())
    }

    /// Get the rendered prompt with all variables substituted.
    pub fn rendered(&self) -> String {
        let vars = match self.variables.lock() {
            Ok(v) => v.clone(),
            Err(_) => return self.template.clone(),
        };

        let mut result = String::new();
        for segment in &self.segments {
            match segment {
                ForgeSegment::Text(t) => result.push_str(t),
                ForgeSegment::Variable { name, default } => {
                    if let Some(value) = vars.get(name) {
                        result.push_str(value);
                    } else if let Some(default) = default {
                        result.push_str(default);
                    } else {
                        result.push_str(&format!("{{{{{}}}}}", name));
                    }
                }
            }
        }
        result
    }

    /// Get the template string.
    pub fn template(&self) -> &str {
        &self.template
    }

    /// Get the parsed segments.
    pub fn segments(&self) -> &[ForgeSegment] {
        &self.segments
    }

    fn parse(template: &str) -> Vec<ForgeSegment> {
        let mut segments = Vec::new();
        let mut current_pos = 0;

        while current_pos < template.len() {
            if let Some(start) = template[current_pos..].find("{{") {
                let abs_start = current_pos + start;

                if abs_start > current_pos {
                    segments.push(ForgeSegment::Text(
                        template[current_pos..abs_start].to_string(),
                    ));
                }

                if let Some(end_offset) = template[abs_start + 2..].find("}}") {
                    let abs_end = abs_start + 2 + end_offset;
                    let var_content = &template[abs_start + 2..abs_end];

                    let (name, default) = if let Some(colon_pos) = var_content.find(':') {
                        (
                            var_content[..colon_pos].trim().to_string(),
                            Some(var_content[colon_pos + 1..].trim().to_string()),
                        )
                    } else {
                        (var_content.trim().to_string(), None)
                    };

                    if !name.is_empty() {
                        segments.push(ForgeSegment::Variable { name, default });
                    }

                    current_pos = abs_end + 2;
                } else {
                    segments.push(ForgeSegment::Text(template[current_pos..].to_string()));
                    break;
                }
            } else {
                segments.push(ForgeSegment::Text(template[current_pos..].to_string()));
                break;
            }
        }

        segments
    }
}

impl View for PromptForge {
    type Body = Never;
    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        let mut x = rect.x + 4.0;
        let mut y = rect.y + 4.0;
        let line_height = 20.0;

        for segment in &self.segments {
            match segment {
                ForgeSegment::Text(text) => {
                    renderer.draw_text(text, x, y, 14.0, theme::text());
                    let (w, _) = renderer.measure_text(text, 14.0);
                    x += w;
                }
                ForgeSegment::Variable { name, default } => {
                    let display = if let Some(default) = default {
                        format!("{}:{}", name, default)
                    } else {
                        name.clone()
                    };
                    let var_text = format!("{{{{{}}}}}", display);
                    let color = if default.is_some() {
                        theme::text_muted()
                    } else {
                        theme::accent()
                    };
                    renderer.draw_text(&var_text, x, y, 14.0, color);
                    let (w, _) = renderer.measure_text(&var_text, 14.0);
                    x += w;
                }
            }

            if x > rect.x + rect.width - 20.0 {
                x = rect.x + 4.0;
                y += line_height;
            }
        }
    }
}
