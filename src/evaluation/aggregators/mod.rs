//! Aggregators and scoring: modules that combine features into final scores.
//!
//! This namespace groups evaluation components responsible for weighting,
//! interpolation, integration, caching and performance aggregation.
//!
//! Feature extraction lives under `crate::evaluation::extractors`.

pub use crate::evaluation::tapered_eval::*;
pub use crate::evaluation::phase_transition::*;
pub use crate::evaluation::integration::*;
pub use crate::evaluation::advanced_interpolation::*;
pub use crate::evaluation::advanced_integration::*;
pub use crate::evaluation::eval_cache::*;
pub use crate::evaluation::statistics::*;
pub use crate::evaluation::performance::*;
pub use crate::evaluation::weight_tuning::*;
pub use crate::evaluation::component_coordinator::*;
pub use crate::evaluation::config::*;
pub use crate::evaluation::telemetry::*;


