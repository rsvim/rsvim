//! Internal tree structure.

// Re-export
pub use inode::*;
pub use itree::*;

pub mod inode;
pub mod itree;
pub mod shapes;

#[cfg(test)]
mod inode_tests;
#[cfg(test)]
mod itree_tests;
#[cfg(test)]
mod shapes_tests;
