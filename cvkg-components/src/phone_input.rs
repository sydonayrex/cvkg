//! PhoneInput component combining a country-code selector and number input.
//!
//! Provides a standardized layout for international telephone number input.

use crate::interactive::Input;
use crate::native_select::NativeSelect;
use cvkg_core::{Never, Rect, Renderer, View, SizeProposal, Size};
use std::sync::Arc;

/// A specialized phone input component with a country-code selector and number input.
#[derive(Clone)]
pub struct PhoneInput {
    /// The current country code (e.g. "+1", "+44").
    pub(crate) country_code: String,
    /// The current phone number.
    pub(crate) phone_number: String,
    /// Callback invoked when the country code or number changes.
    pub(crate) on_change: Arc<dyn Fn(String, String) + Send + Sync>,
}

impl PhoneInput {
    /// Create a new PhoneInput with default values.
    ///
    /// # Contract
    /// - Default country code is "+1".
    /// - Default phone number is empty.
    pub fn new() -> Self {
        Self {
            country_code: "+1".to_string(),
            phone_number: String::new(),
            on_change: Arc::new(|_, _| {}),
        }
    }

    /// Set the current country code.
    pub fn country_code(mut self, code: impl Into<String>) -> Self {
        self.country_code = code.into();
        self
    }

    /// Set the current phone number value.
    pub fn phone_number(mut self, val: impl Into<String>) -> Self {
        self.phone_number = val.into();
        self
    }

    /// Set the callback for value updates.
    ///
    /// # Arguments
    /// * `callback` - Invoked with `(country_code, phone_number)` on any change.
    pub fn on_change(mut self, callback: impl Fn(String, String) + Send + Sync + 'static) -> Self {
        self.on_change = Arc::new(callback);
        self
    }
}

impl View for PhoneInput {
    type Body = Never;

    fn body(self) -> Self::Body {
        unreachable!()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        renderer.push_vnode(rect, "PhoneInput");

        let country_w = 80.0;
        let select_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: country_w,
            height: rect.height,
        };
        let input_rect = Rect {
            x: rect.x + country_w + 8.0,
            y: rect.y,
            width: (rect.width - country_w - 8.0).max(0.0),
            height: rect.height,
        };

        // Construct standard country choices
        let options = vec![
            ("+1".to_string(), "+1 (US)".to_string()),
            ("+44".to_string(), "+44 (UK)".to_string()),
            ("+49".to_string(), "+49 (DE)".to_string()),
            ("+81".to_string(), "+81 (JP)".to_string()),
            ("+86".to_string(), "+86 (CN)".to_string()),
        ];

        let on_change_select = self.on_change.clone();
        let phone_clone = self.phone_number.clone();
        let select = NativeSelect::new(options, self.country_code.clone(), move |new_code| {
            (on_change_select)(new_code, phone_clone.clone());
        });

        let on_change_input = self.on_change.clone();
        let code_clone = self.country_code.clone();
        let input = Input::new("Phone Number")
            .value(self.phone_number.clone())
            .on_change(move |new_num| {
                (on_change_input)(code_clone.clone(), new_num);
            });

        select.render(renderer, select_rect);
        input.render(renderer, input_rect);

        renderer.pop_vnode();
    }

    fn intrinsic_size(&self, _renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        Size {
            width: proposal.width.unwrap_or(280.0).max(180.0),
            height: 44.0,
        }
    }
}
