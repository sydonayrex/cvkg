//! Responsive breakpoint component.
//!
//! Provides `Responsive<T>` — a view that picks one of four pre-built layout
//! variants based on the **available container width**, using Material Design
//! breakpoints: xs (<600px), sm (600–899px), md (900–1199px), lg (≥1200px).
//!
//! Unlike `FlexiScope` (which uses generic breakpoints and a builder pattern),
//! `Responsive<T>` provides a fixed 4-slot API optimised for the most common
//! responsive pattern: one layout per breakpoint tier.
//!
//! # Example
//!
//! ```ignore
//! use cvkg_components::Responsive;
//!
//! let view = Responsive::new(
//!     || CompactNav::new(),       // xs: mobile
//!     || SideNav::new(),          // sm: tablet
//!     || FullNav::new(),          // md: desktop
//!     || WideNav::new(),          // lg: wide
//! );
//! ```

use crate::flexiscope::{ContainerLayout, FlexiScope, ScopeThreshold};
use cvkg_core::{AriaProperties, Rect, Renderer, Size, SizeProposal, View};

/// Material Design breakpoint tiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Breakpoint {
    /// xs: 0–599px — handset portrait
    Xs,
    /// sm: 600–899px — tablet portrait / handset landscape
    Sm,
    /// md: 900–1199px — tablet landscape / desktop
    Md,
    /// lg: ≥1200px — desktop wide
    Lg,
}

impl ContainerLayout for Breakpoint {
    fn select_mode(width: f32, _breakpoints: &[ScopeThreshold<Self>]) -> Self {
        if width >= 1200.0 {
            Breakpoint::Lg
        } else if width >= 900.0 {
            Breakpoint::Md
        } else if width >= 600.0 {
            Breakpoint::Sm
        } else {
            Breakpoint::Xs
        }
    }
}

/// A responsive wrapper that picks one of four pre-built layout variants
/// based on available container width.
///
/// Breakpoint thresholds (Material Design):
/// - **xs**: 0–599px  — handset portrait
/// - **sm**: 600–899px — tablet portrait / handset landscape
/// - **md**: 900–1199px — tablet landscape / desktop
/// - **lg**: ≥1200px  — desktop wide
///
/// Each tier receives a **builder closure** that produces a `View`. Only the
/// matching tier's closure is called per frame, so unused layouts cost nothing.
///
/// # Example
///
/// ```ignore
/// use cvkg_components::Responsive;
///
/// let view = Responsive::new(
///     || CompactCard::new("Mobile"),
///     || Card::new("Tablet"),
///     || WideCard::new("Desktop"),
///     || WideCard::new("Wide").with_sidebar(),
/// );
/// ```
pub struct Responsive<V: View> {
    inner: FlexiScope<V, Breakpoint>,
}

impl<V: View + 'static> Responsive<V> {
    /// Create a `Responsive` with one layout variant per breakpoint tier.
    ///
    /// Each argument is a **builder closure** `Fn() -> V`. The closure for the
    /// active breakpoint is called on each render frame; unused closures are
    /// not invoked.
    pub fn new(
        xs: impl Fn() -> V + Send + Sync + 'static,
        sm: impl Fn() -> V + Send + Sync + 'static,
        md: impl Fn() -> V + Send + Sync + 'static,
        lg: impl Fn() -> V + Send + Sync + 'static,
    ) -> Self {
        let breakpoints = vec![
            ScopeThreshold {
                min_width: 0.0,
                mode: Breakpoint::Xs,
            },
            ScopeThreshold {
                min_width: 600.0,
                mode: Breakpoint::Sm,
            },
            ScopeThreshold {
                min_width: 900.0,
                mode: Breakpoint::Md,
            },
            ScopeThreshold {
                min_width: 1200.0,
                mode: Breakpoint::Lg,
            },
        ];
        Self {
            inner: FlexiScope::new(
                move |bp: Breakpoint| match bp {
                    Breakpoint::Xs => xs(),
                    Breakpoint::Sm => sm(),
                    Breakpoint::Md => md(),
                    Breakpoint::Lg => lg(),
                },
                breakpoints,
            ),
        }
    }
}

impl<V: View + 'static> View for Responsive<V> {
    type Body = <FlexiScope<V, Breakpoint> as View>::Body;

    fn body(self) -> Self::Body {
        self.inner.body()
    }

    fn render(&self, renderer: &mut dyn Renderer, rect: Rect) {
        self.inner.render(renderer, rect);
    }

    fn intrinsic_size(&self, renderer: &mut dyn Renderer, proposal: SizeProposal) -> Size {
        self.inner.intrinsic_size(renderer, proposal)
    }

    fn layout(&self) -> Option<&dyn cvkg_core::layout::LayoutView> {
        self.inner.layout()
    }

    fn aria_properties(&self) -> Option<AriaProperties> {
        self.inner.aria_properties()
    }
}
