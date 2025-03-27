//! Prelude.

// Re-export `coord`.
pub use crate::coord::*;
pub use crate::results::*;

// Re-export `ahash`;
pub use ahash::AHashMap as HashMap;
pub use ahash::AHashSet as HashSet;

// Re-export `geo`;
pub use geo::{self, Point, Rect};
