//! Prelude.

pub use log::{debug, error, info, trace, warn};

pub use crate::constant::*;
pub use crate::coord::*;
pub use crate::results::*;

pub use crate::{arc_mutex_ptr, arc_ptr, lock, rc_ptr, rc_refcell_ptr};
pub use paste::paste;

pub use foldhash::fast::RandomState as FandomState;
pub use foldhash::{
  HashMap as FashMap, HashMapExt as FashMapExt, HashSet as FashSet,
  HashSetExt as FashSetExt,
};
pub use std::collections::{BTreeMap, BTreeSet /*, HashMap, HashSet */};
// pub use std::hash::RandomState;

pub use geo::{self, Point, Rect};
