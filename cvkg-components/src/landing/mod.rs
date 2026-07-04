//! Landing page compound components.
//!
//! Hero, PricingTable, FeatureGrid, and TestimonialCard for marketing/landing pages.

pub mod hero;
pub mod pricing_table;
pub mod feature_grid;
pub mod testimonial_card;

pub use hero::Hero;
pub use pricing_table::{PricingTable, PricingCard};
pub use feature_grid::{FeatureGrid, FeatureItem};
pub use testimonial_card::{TestimonialCard, TestimonialItem};