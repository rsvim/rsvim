//! Internal tree structure.

pub mod inode;
pub mod itree;
pub mod shapes;

// Re-export
pub use crate::ui::tree::internal::inode::{InodeBase, Inodeable, TreeNodeId};
pub use crate::ui::tree::internal::itree::{Itree, ItreeIter /*, ItreeIterMut*/};
