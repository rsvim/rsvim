//! ID generator and UUID.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Returns the next global unique ID.
pub fn next() -> usize {
  static GLOBAL: AtomicUsize = AtomicUsize::new(0_usize);
  GLOBAL.fetch_add(1_usize, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn incremental_ids() {
    let start = next();
    for i in 0..10 {
      let actual = next();
      // println!(
      //   "uuid::incremental_ids i+start+1:{},next:{}",
      //   i + start + 1,
      //   actual
      // );
      assert_eq!(actual, i + start + 1);
    }
  }
}
