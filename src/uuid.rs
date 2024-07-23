//! ID generator and UUID.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Get the next global unique ID.
pub fn next() -> usize {
  static GLOBAL: AtomicUsize = AtomicUsize::new(0usize);
  GLOBAL.fetch_add(0, Ordering::SeqCst)
}
