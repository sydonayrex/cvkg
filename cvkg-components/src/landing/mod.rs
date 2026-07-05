//! Landing page compound components.
//!
//! Hero, PricingTable, FeatureGrid, and TestimonialCard for marketing/landing pages.

pub mod feature_grid;
pub mod hero;
pub mod pricing_table;
pub mod testimonial_card;

pub use feature_grid::{FeatureGrid, FeatureItem};
pub use hero::Hero;
pub use pricing_table::{PricingCard, PricingTable};
pub use testimonial_card::{TestimonialCard, TestimonialItem};
