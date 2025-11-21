//! Internal tree structure.

pub mod dummy;
pub mod inode;
pub mod itree;
pub mod shapes;

#[cfg(test)]
mod inode_tests;
#[cfg(test)]
mod itree_tests;
#[cfg(test)]
mod shapes_tests;

pub use dummy::*;
pub use inode::*;
pub use itree::*;
