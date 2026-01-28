use crate::next_incremental_id_impl;
use crate::prelude::*;
use crate::struct_id_impl;
use crate::tests::log::init as test_log_init;
use std::sync::atomic::AtomicU8;

struct_id_impl!(TestId, u8);
next_incremental_id_impl!(next_test_id, TestId, AtomicU8, u8);

#[test]
fn next_test_id1() {
  test_log_init();

  let mut miss_count = 0;
  let mut last_id: Option<TestId> = None;
  for i in 0..1000 {
    let id = next_test_id();
    info!(
      "i:{:?},id:{:?},last_id:{:?},miss_count:{:?}",
      i, id, last_id, miss_count
    );
    if let Some(last_id) = last_id {
      assert!(last_id.value() >= 1);
      if last_id.value() == u8::MAX {
        assert_eq!(id.value(), 1);
      } else {
        assert_eq!(last_id.value() + 1, id.value());
      }
    } else {
      miss_count += 1;
      assert!(miss_count <= 1);
    }
    last_id = Some(id);
  }
}
