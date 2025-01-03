//! Indexing for vim buffer.

// Re-export
pub use crate::buf::idx::lidx::BufLindex;
pub use crate::buf::idx::widx::BufWindex;

pub mod lidx;
pub mod widx;
