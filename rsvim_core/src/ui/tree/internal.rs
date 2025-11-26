//! Internal tree structure.

pub mod itree;
pub mod shapes;

#[cfg(test)]
mod itree_tests;
#[cfg(test)]
mod shapes_tests;

pub use itree::*;
