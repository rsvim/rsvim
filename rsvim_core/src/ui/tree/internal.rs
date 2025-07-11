//! Internal tree structure.

pub mod inode;
pub mod itree;
pub mod shapes;

// Re-export
pub use inode::*;
pub use itree::*;

#[cfg(test)]
mod inode_tests;
#[cfg(test)]
mod itree_tests;
#[cfg(test)]
mod shapes_tests;
