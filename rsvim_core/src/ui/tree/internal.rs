//! Internal tree structure.

pub mod inode;
pub mod itree;

#[cfg(test)]
mod itree_tests;

pub use inode::*;
pub use itree::*;
