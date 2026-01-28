use crate::next_incremental_id_impl;
use crate::prelude::*;
use crate::struct_id_impl;
use crate::tests::log::init as test_log_init;
use std::sync::atomic::AtomicU8;

struct_id_impl!(Test1Id, u8);
next_incremental_id_impl!(next_test1_id, Test1Id, AtomicU8, u8, 1);

#[test]
fn test_next_test1_id() {
  test_log_init();

  let mut miss_count = 0;
  let mut last_id: Option<Test1Id> = None;
  for i in 0..1000 {
    let id = next_test1_id();
    info!(
      "i:{:?},id:{:?},last_id:{:?},miss_count:{:?}",
      i, id, last_id, miss_count
    );
    if let Some(last_id) = last_id {
      assert!(std::convert::Into::<u8>::into(last_id) >= 1);
      if std::convert::Into::<u8>::into(last_id) == u8::MAX {
        assert_eq!(std::convert::Into::<u8>::into(id), 1);
      } else {
        assert_eq!(
          std::convert::Into::<u8>::into(last_id) + 1,
          std::convert::Into::<u8>::into(id)
        );
      }
    } else {
      miss_count += 1;
      assert!(miss_count <= 1);
    }
    last_id = Some(id);
  }
}

struct_id_impl!(Test2Id, u8);
next_incremental_id_impl!(next_test2_id, Test2Id, AtomicU8, u8, 100);

#[test]
fn test_next_test2_id() {
  test_log_init();

  let mut miss_count = 0;
  let mut last_id: Option<Test2Id> = None;
  for i in 0..1000 {
    let id = next_test2_id();
    info!(
      "i:{:?},id:{:?},last_id:{:?},miss_count:{:?}",
      i, id, last_id, miss_count
    );
    if let Some(last_id) = last_id {
      assert!(std::convert::Into::<u8>::into(last_id) >= 100);
      if std::convert::Into::<u8>::into(last_id) == u8::MAX {
        assert_eq!(std::convert::Into::<u8>::into(id), 100);
      } else {
        assert_eq!(
          std::convert::Into::<u8>::into(last_id) + 1,
          std::convert::Into::<u8>::into(id)
        );
      }
    } else {
      miss_count += 1;
      assert!(miss_count <= 1);
    }
    last_id = Some(id);
  }
}
