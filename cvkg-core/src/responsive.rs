//! Responsive breakpoint tokens and layout integration.
//!
//! Defines standard CSS-style breakpoints (sm/md/lg/xl/2xl) as constants
//! and provides a `Breakpoint` enum for runtime layout decisions.

use std::fmt;

/// Standard responsive breakpoints in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Breakpoint {
    /// Small: 640px — large phones, small tablets.
    Sm,
    /// Medium: 768px — tablets.
    Md,
    /// Large: 1024px — small laptops, landscape tablets.
    Lg,
    /// Extra large: 1280px — desktops.
    Xl,
    /// 2X large: 1536px — large desktops.
    Xxl,
}

impl Breakpoint {
    /// The minimum width in logical pixels for this breakpoint.
    pub fn min_width(self) -> f32 {
        match self {
            Breakpoint::Sm => 640.0,
            Breakpoint::Md => 768.0,
            Breakpoint::Lg => 1024.0,
            Breakpoint::Xl => 1280.0,
            Breakpoint::Xxl => 1536.0,
        }
    }

    /// Determine the breakpoint for a given width.
    pub fn from_width(width: f32) -> Self {
        if width >= 1536.0 {
            Breakpoint::Xxl
        } else if width >= 1280.0 {
            Breakpoint::Xl
        } else if width >= 1024.0 {
            Breakpoint::Lg
        } else if width >= 768.0 {
            Breakpoint::Md
        } else {
            Breakpoint::Sm
        }
    }

    /// Check if this breakpoint is at least as large as another.
    pub fn is_at_least(self, other: Breakpoint) -> bool {
        self.min_width() >= other.min_width()
    }
}

impl fmt::Display for Breakpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Breakpoint::Sm => write!(f, "sm"),
            Breakpoint::Md => write!(f, "md"),
            Breakpoint::Lg => write!(f, "lg"),
            Breakpoint::Xl => write!(f, "xl"),
            Breakpoint::Xxl => write!(f, "2xl"),
        }
    }
}

/// Named z-index layers for responsive layout control.
pub mod z_index_layers {
    /// Base content layer.
    pub const BASE: i32 = 0;
    /// Sticky header layer.
    pub const STICKY: i32 = 100;
    /// Floating element layer.
    pub const FLOATING: i32 = 200;
    /// Overlay/dropdown layer.
    pub const OVERLAY: i32 = 1000;
    /// Modal layer.
    pub const MODAL: i32 = 2000;
    /// Toast notification layer.
    pub const TOAST: i32 = 3000;
    /// Tooltip layer (always on top).
    pub const TOOLTIP: i32 = 5000;
}

/// Responsive value that changes based on the current breakpoint.
#[derive(Debug, Clone)]
pub struct Responsive<T: Clone> {
    pub sm: T,
    pub md: Option<T>,
    pub lg: Option<T>,
    pub xl: Option<T>,
    pub xxl: Option<T>,
}

impl<T: Clone> Responsive<T> {
    /// Create a responsive value with the same value at all breakpoints.
    pub fn all(value: T) -> Self {
        Self {
            sm: value.clone(),
            md: Some(value.clone()),
            lg: Some(value.clone()),
            xl: Some(value.clone()),
            xxl: Some(value),
        }
    }

    /// Create a responsive value with custom values per breakpoint.
    pub fn new(sm: T, md: T, lg: T, xl: T, xxl: T) -> Self {
        Self {
            sm,
            md: Some(md),
            lg: Some(lg),
            xl: Some(xl),
            xxl: Some(xxl),
        }
    }

    /// Get the value for a given breakpoint (falls back to smaller breakpoints).
    pub fn value_at(&self, breakpoint: Breakpoint) -> T {
        match breakpoint {
            Breakpoint::Sm => self.sm.clone(),
            Breakpoint::Md => self.md.clone().unwrap_or_else(|| self.sm.clone()),
            Breakpoint::Lg => self
                .lg
                .clone()
                .or_else(|| self.md.clone())
                .unwrap_or_else(|| self.sm.clone()),
            Breakpoint::Xl => self
                .xl
                .clone()
                .or_else(|| self.lg.clone())
                .or_else(|| self.md.clone())
                .unwrap_or_else(|| self.sm.clone()),
            Breakpoint::Xxl => self
                .xxl
                .clone()
                .or_else(|| self.xl.clone())
                .or_else(|| self.lg.clone())
                .or_else(|| self.md.clone())
                .unwrap_or_else(|| self.sm.clone()),
        }
    }
}

impl<T: Clone> Default for Responsive<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            sm: T::default(),
            md: None,
            lg: None,
            xl: None,
            xxl: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoint_min_widths() {
        assert_eq!(Breakpoint::Sm.min_width(), 640.0);
        assert_eq!(Breakpoint::Md.min_width(), 768.0);
        assert_eq!(Breakpoint::Lg.min_width(), 1024.0);
        assert_eq!(Breakpoint::Xl.min_width(), 1280.0);
        assert_eq!(Breakpoint::Xxl.min_width(), 1536.0);
    }

    #[test]
    fn breakpoint_from_width() {
        assert_eq!(Breakpoint::from_width(320.0), Breakpoint::Sm);
        assert_eq!(Breakpoint::from_width(640.0), Breakpoint::Sm);
        assert_eq!(Breakpoint::from_width(768.0), Breakpoint::Md);
        assert_eq!(Breakpoint::from_width(1024.0), Breakpoint::Lg);
        assert_eq!(Breakpoint::from_width(1280.0), Breakpoint::Xl);
        assert_eq!(Breakpoint::from_width(1536.0), Breakpoint::Xxl);
        assert_eq!(Breakpoint::from_width(2000.0), Breakpoint::Xxl);
    }

    #[test]
    fn breakpoint_is_at_least() {
        assert!(Breakpoint::Md.is_at_least(Breakpoint::Sm));
        assert!(Breakpoint::Lg.is_at_least(Breakpoint::Md));
        assert!(!Breakpoint::Sm.is_at_least(Breakpoint::Md));
    }

    #[test]
    fn responsive_all_same() {
        let r = Responsive::all(10);
        assert_eq!(r.value_at(Breakpoint::Sm), 10);
        assert_eq!(r.value_at(Breakpoint::Xxl), 10);
    }

    #[test]
    fn responsive_fallback() {
        let r: Responsive<i32> = Responsive::new(10, 20, 30, 40, 50);
        assert_eq!(r.value_at(Breakpoint::Sm), 10);
        assert_eq!(r.value_at(Breakpoint::Md), 20);
        assert_eq!(r.value_at(Breakpoint::Lg), 30);
        assert_eq!(r.value_at(Breakpoint::Xl), 40);
        assert_eq!(r.value_at(Breakpoint::Xxl), 50);
    }

    #[test]
    fn responsive_partial_fallback() {
        // Only set sm and xl; others fall back
        let r = Responsive {
            sm: 100,
            md: None,
            lg: None,
            xl: Some(200),
            xxl: None,
        };
        assert_eq!(r.value_at(Breakpoint::Sm), 100);
        assert_eq!(r.value_at(Breakpoint::Md), 100); // falls back to sm
        assert_eq!(r.value_at(Breakpoint::Lg), 100); // falls back to sm
        assert_eq!(r.value_at(Breakpoint::Xl), 200);
        assert_eq!(r.value_at(Breakpoint::Xxl), 200); // falls back to xl
    }

    #[test]
    fn breakpoint_display() {
        assert_eq!(format!("{}", Breakpoint::Sm), "sm");
        assert_eq!(format!("{}", Breakpoint::Md), "md");
        assert_eq!(format!("{}", Breakpoint::Lg), "lg");
        assert_eq!(format!("{}", Breakpoint::Xl), "xl");
        assert_eq!(format!("{}", Breakpoint::Xxl), "2xl");
    }
}
