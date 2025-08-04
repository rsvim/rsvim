//! Prelude.

pub use log::{debug, error, info, trace, warn};

pub use crate::constant::*;
pub use crate::coord::*;
pub use crate::results::*;

pub use crate::{arc_mutex_ptr, arc_ptr, lock, rc_ptr, rc_refcell_ptr};
pub use paste::paste;

pub use ahash::AHashMap as HashMap;
pub use ahash::AHashSet as HashSet;

pub use geo::{self, Point, Rect};
