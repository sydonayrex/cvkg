//! Ornamental components -- Decorative border systems inspired by Norse art.
//!
//! This module provides runic ornament borders that decorate rectangular
//! regions with patterns drawn from Elder Futhark runes, Celtic knotwork,
//! hammered metal, dragon scales, and ice crystals.

pub mod aetti_frame;
pub use aetti_frame::{RunicStyle, ÆttiFrame};

pub mod mjolnir_frame_ext;
pub use mjolnir_frame_ext::{MjolnirFrameStyle, RuneGlyph, render_mjolnir_frame};
