//! Internal tree structure.

pub mod inode;
pub mod itree;
pub mod shapes;

// Re-export
pub use crate::ui::tree::internal::inode::{InodeBase, InodeId, Inodeable};
pub use crate::ui::tree::internal::itree::{Itree, ItreeIter, ItreeIterMut};
