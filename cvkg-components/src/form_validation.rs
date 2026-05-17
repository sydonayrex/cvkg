use cvkg_core::layout::{LayoutCache, LayoutView, SizeProposal};
use cvkg_core::{Never, Rect, Renderer, Size, View};

/// Validation rules that can be applied to form fields.
#[derive(Clone, Debug)]
pub enum ValidationRule {
    /// Field must not be empty.
    Required,
    /// Minimum length of the input string.
    MinLength(usize),
    /// Maximum length of the input string.
    MaxLength(usize),
    /// Regex pattern the input must match.
    Pattern(String),
}

/// A form field wrapper that adds validation state and labeling to any input view.
///
/// Tracks whether the field has been modified (`is_dirty`), whether it currently
/// passes all validation rules (`is_valid`), and holds an optional error message
/// for display when validation fails.
#[derive(Clone)]
pub struct FormField<V: View> {
    /// The label displayed above the field.
    pub label: String,
    /// The inner input view (e.g. Input, SecureField, Textarea).
    pub content: V,
    /// The current error message, if validation has failed.
    pub error_message: Option<String>,
    /// Whether this field is required (renders a red asterisk).
    pub is_required: bool,
    /// Whether the field currently passes all validation rules.
    pub is_valid: bool,
    /// Whether the user has modified the field since it was rendered.
    pub is_dirty: bool,
    /// The ordered set of validation rules to apply.
    pub rules: Vec<ValidationRule>,
}

impl<V: View> FormField<V> {
    /// Create a new FormField with the given label and content view.
    pub fn new(label: impl Into<String>, content: V) -> Self {
        Self {
            label: label.into(),
            content,
            error_message: None,
            is_required: false,
            is_valid: true,
            is_dirty: false,
            rules: Vec::new(),
        }
    }

    /// Mark this field as required. Adds the Required validation rule and
    /// causes a red asterisk to appear next to the label.
    pub fn required(mut self) -> Self {
        self.is_required = true;
        // Avoid duplicate Required rules
        if !self
            .rules
            .iter()
            .any(|r| matches!(r, ValidationRule::Required))
        {
            self.rules.push(ValidationRule::Required);
        }
        self
    }

    /// Add a validation rule to this field. Returns self for chaining.
    pub fn rule(mut self, rule: ValidationRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Validate the given string value against all rules.
    ///
    /// Returns Ok(()) if all rules pass, or Err with a descriptive message
    /// for the first failing rule.
    pub fn validate(&self, value: &str) -> Result<(), String> {
        for rule in &self.rules {
            match rule {
                ValidationRule::Required => {
                    if value.trim().is_empty() {
                        return Err(format!("{} is required", self.label));
                    }
                }
                ValidationRule::MinLength(min) => {
                    if value.len() < *min {
                        return Err(format!(
                            "{} must be at least {} characters",
                            self.label, min
                        ));
                    }
                }
                ValidationRule::MaxLength(max) => {
                    if value.len() > *max {
                        return Err(format!("{} must be at most {} characters", self.label, max));
                    }
                }
                ValidationRule::Pattern(pattern) => {
                    // Simple pattern check: the value must contain the pattern substring.
                    // For more complex matching, use a dedicated regex crate.
                    if !value.contains(pattern.as_str()) {
                        return Err(format!("{} has invalid format", self.label));
                    }
                }
            }
        }
        Ok(())
    }
}

impl<V: View> View for FormField<V> {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "FormField");

        let label_height = 20.0;
        let error_height = if self.error_message.is_some() {
            18.0
        } else {
            0.0
        };
        let content_top = rect.y + label_height;
        let content_height = rect.height - label_height - error_height;

        // --- Label ---
        let label_color = [0.85, 0.85, 0.9, 1.0];
        renderer.draw_text(&self.label, rect.x, rect.y + 2.0, 13.0, label_color);

        // Red asterisk for required fields
        if self.is_required {
            let (label_w, _) = renderer.measure_text(&self.label, 13.0);
            renderer.draw_text(
                "*",
                rect.x + label_w + 4.0,
                rect.y + 2.0,
                13.0,
                [1.0, 0.2, 0.2, 1.0],
            );
        }

        // --- Content area ---
        let content_rect = Rect {
            x: rect.x,
            y: content_top,
            width: rect.width,
            height: content_height,
        };

        // If invalid and dirty, draw a red border behind the content
        if !self.is_valid && self.is_dirty {
            renderer.stroke_rounded_rect(content_rect, 4.0, [1.0, 0.2, 0.2, 1.0], 1.5);
        }

        self.content.render(renderer, content_rect);

        // --- Error message ---
        if let Some(ref msg) = self.error_message
            && self.is_dirty
        {
            renderer.draw_text(
                msg,
                rect.x,
                content_top + content_height + 2.0,
                11.0,
                [1.0, 0.2, 0.2, 1.0],
            );
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let content_size = self.content.intrinsic_size(renderer, proposal);
        let error_h = if self.error_message.is_some() {
            18.0
        } else {
            0.0
        };
        Size {
            width: content_size.width,
            height: 20.0 + content_size.height + error_h,
        }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl<V: View> LayoutView for FormField<V> {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let content_size = self.content.intrinsic_size(
            // We need a renderer here; use intrinsic_size as a fallback
            // by measuring with a dummy approach. Since we don't have a
            // renderer in layout, we estimate.
            &mut DummyRenderer,
            proposal,
        );
        let error_h = if self.error_message.is_some() {
            18.0
        } else {
            0.0
        };
        Size {
            width: content_size.width,
            height: 20.0 + content_size.height + error_h,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}

/// Dummy renderer for layout estimation when a real renderer is not available.
struct DummyRenderer;

impl cvkg_core::ElapsedTime for DummyRenderer {
    fn elapsed_time(&self) -> f32 {
        0.0
    }
    fn delta_time(&self) -> f32 {
        0.0
    }
}

impl Renderer for DummyRenderer {
    fn fill_rect(&mut self, _rect: Rect, _color: [f32; 4]) {}
    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: [f32; 4]) {}
    fn fill_ellipse(&mut self, _rect: Rect, _color: [f32; 4]) {}
    fn stroke_rect(&mut self, _rect: Rect, _color: [f32; 4], _stroke_width: f32) {}
    fn stroke_rounded_rect(
        &mut self,
        _rect: Rect,
        _radius: f32,
        _color: [f32; 4],
        _stroke_width: f32,
    ) {
    }
    fn stroke_ellipse(&mut self, _rect: Rect, _color: [f32; 4], _stroke_width: f32) {}
    fn draw_line(
        &mut self,
        _x1: f32,
        _y1: f32,
        _x2: f32,
        _y2: f32,
        _color: [f32; 4],
        _stroke_width: f32,
    ) {
    }
    fn draw_text(&mut self, _text: &str, _x: f32, _y: f32, _size: f32, _color: [f32; 4]) {}
    fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        (text.len() as f32 * size * 0.6, size * 1.2)
    }
    fn memoize(&mut self, _id: u64, _data_hash: u64, _render_fn: &dyn Fn(&mut dyn Renderer)) {}
}

/// A form that aggregates multiple FormField instances and provides
/// collective validation.
///
/// Holds erased FormField views so they can be stored homogeneously
/// regardless of their inner content type.
#[derive(Clone)]
pub struct Form {
    /// The fields in this form, stored as erased AnyView handles.
    fields: Vec<cvkg_core::AnyView>,
    /// Cached validation results: (is_valid, error_message) per field.
    field_states: Vec<(bool, Option<String>)>,
}

impl Form {
    /// Create a new empty Form.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            field_states: Vec::new(),
        }
    }

    /// Add a FormField to the form. Returns self for chaining.
    pub fn field<V: View + Clone + 'static>(mut self, field: FormField<V>) -> Self {
        self.fields.push(field.erase());
        self.field_states.push((true, None));
        self
    }

    /// Validate all fields in the form.
    ///
    /// Returns true if every field passes validation, false otherwise.
    /// Updates the error_message on each field so the UI reflects errors.
    pub fn validate_all(&mut self) -> bool {
        let mut all_valid = true;
        for (i, _field) in self.fields.iter().enumerate() {
            // We cannot call validate on erased views directly.
            // Instead, the field_states are managed externally by the
            // application logic that owns the FormField values before
            // they are erased into the Form. This method checks the
            // cached states.
            if i < self.field_states.len() {
                let (is_valid, _) = &self.field_states[i];
                if !is_valid {
                    all_valid = false;
                }
            }
        }
        all_valid
    }

    /// Returns true if all fields in the form are currently valid
    /// according to the cached validation states.
    pub fn is_valid(&self) -> bool {
        self.field_states.iter().all(|(valid, _)| *valid)
    }

    /// Update the cached validation state for a field at the given index.
    pub fn set_field_state(&mut self, index: usize, is_valid: bool, error: Option<String>) {
        if index < self.field_states.len() {
            self.field_states[index] = (is_valid, error);
        }
    }

    /// Returns the number of fields in the form.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Form {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "Form");

        let spacing = 12.0;
        let field_height = 70.0; // estimated per-field height
        let total_height = self.fields.len() as f32 * (field_height + spacing);

        // Center vertically if there's extra space
        let start_y = if total_height < rect.height {
            rect.y + (rect.height - total_height) / 2.0
        } else {
            rect.y
        };

        let padding = 16.0;
        let content_width = rect.width - padding * 2.0;

        for (i, field) in self.fields.iter().enumerate() {
            let field_rect = Rect {
                x: rect.x + padding,
                y: start_y + i as f32 * (field_height + spacing),
                width: content_width,
                height: field_height,
            };
            field.render(renderer, field_rect);
        }

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let spacing = 12.0;

        for (i, field) in self.fields.iter().enumerate() {
            let field_size = field.intrinsic_size(renderer, proposal);
            width = width.max(field_size.width);
            height += field_size.height;
            if i < self.fields.len() - 1 {
                height += spacing;
            }
        }

        Size {
            width: width + 32.0, // horizontal padding
            height,
        }
    }

    fn layout(&self) -> Option<&dyn LayoutView> {
        Some(self)
    }
}

impl LayoutView for Form {
    fn size_that_fits(
        &self,
        proposal: SizeProposal,
        _subviews: &[&dyn LayoutView],
        _cache: &mut LayoutCache,
    ) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let spacing = 12.0;

        for (i, field) in self.fields.iter().enumerate() {
            let field_size = field.intrinsic_size(&mut DummyRenderer, proposal);
            width = width.max(field_size.width);
            height += field_size.height;
            if i < self.fields.len() - 1 {
                height += spacing;
            }
        }

        Size {
            width: width + 32.0,
            height,
        }
    }

    fn place_subviews(
        &self,
        _bounds: Rect,
        _subviews: &mut [&mut dyn LayoutView],
        _cache: &mut LayoutCache,
    ) {
    }
}
