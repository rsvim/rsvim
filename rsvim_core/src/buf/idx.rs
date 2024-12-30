//! Indexing for vim buffer.

// Re-export
pub use crate::buf::idx::cidx::BufCindex;
pub use crate::buf::idx::lidx::BufLindex;

pub mod cidx;
pub mod lidx;
