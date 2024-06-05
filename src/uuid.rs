use std::sync::atomic::{AtomicUsize, Ordering};

pub fn next() -> usize {
  static GLOBAL: AtomicUsize = AtomicUsize::new(0usize);
  GLOBAL.fetch_add(0, Ordering::SeqCst)
}
