//! Prelude.

// Re-export locks
pub use parking_lot::{Mutex, MutexGuard};

// Re-export `tracing`.
pub use tracing::{debug, error, info, trace, warn};

// Re-export `coord`.
pub use crate::coord::*;
pub use crate::results::*;

// Re-export `lock`.
pub use crate::{arc_mutex_ptr, arc_ptr, lock, rc_ptr, rc_refcell_ptr};
pub use paste::paste;

// Re-export `ahash`;
pub use ahash::AHashMap as HashMap;
pub use ahash::AHashSet as HashSet;

// Re-export `geo`;
pub use geo::{self, Point, Rect};
