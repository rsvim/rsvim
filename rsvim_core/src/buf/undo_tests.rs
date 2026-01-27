use super::undo::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use compact_str::ToCompactString;
use ropey::RopeBuilder;

const MAX_SIZE: usize = 100;

fn assert_insert(undo_manager: &UndoManager, op_idx: usize, op: Insert) {
  assert!(undo_manager.current().records().len() > op_idx);
  let actual = undo_manager.current().records()[op_idx].clone();
  assert!(matches!(actual.op, Operation::Insert(_)));
  match actual.op {
    Operation::Insert(insert) => assert_eq!(insert, op),
    _ => unreachable!(),
  }
}

fn assert_delete(undo_manager: &UndoManager, op_idx: usize, op: Delete) {
  assert!(undo_manager.current().records().len() > op_idx);
  let actual = undo_manager.current().records()[op_idx].clone();
  assert!(matches!(actual.op, Operation::Delete(_)));
  match actual.op {
    Operation::Delete(delete) => assert_eq!(delete, op),
    _ => unreachable!(),
  }
}

#[test]
fn insert1() {
  let mut undo_mgr = UndoManager::new(MAX_SIZE);
  let payload = "Hello, World!";
  for (i, c) in payload.chars().enumerate() {
    undo_mgr.current_mut().insert(Insert {
      payload: c.to_compact_string(),
      char_idx_before: i,
      char_idx_after: i + c.to_compact_string().chars().count(),
    });
  }
  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload.to_compact_string().chars().count(),
    },
  );
  undo_mgr.commit();

  let actual = undo_mgr.current();
  assert!(actual.records().is_empty());
}

#[test]
fn insert2() {
  let mut undo_mgr = UndoManager::new(MAX_SIZE);
  let payload1 = "Hello, ";
  for (i, c) in payload1.chars().enumerate() {
    undo_mgr.current_mut().insert(Insert {
      payload: c.to_compact_string(),
      char_idx_before: i,
      char_idx_after: i + 1,
    });
  }
  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.to_compact_string().chars().count(),
    },
  );

  let payload2 = "World!";
  for (i, c) in payload2.chars().enumerate() {
    undo_mgr.current_mut().insert(Insert {
      char_idx_before: i + 3,
      char_idx_after: i + 4,
      payload: c.to_compact_string(),
    });
  }
  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 2);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: "Hello, ".to_compact_string(),
      char_idx_before: 0,
      char_idx_after: "Hello, ".chars().count(),
    },
  );
  assert_insert(
    &undo_mgr,
    1,
    Insert {
      payload: payload2.to_compact_string(),
      char_idx_before: 3,
      char_idx_after: 3 + "World!".chars().count(),
    },
  );

  let payload3 = "汤姆(Tom)?";
  for (i, c) in payload3.chars().enumerate() {
    undo_mgr.current_mut().insert(Insert {
      char_idx_before: i + payload1.chars().count() + payload2.chars().count(),
      char_idx_after: i
        + payload1.chars().count()
        + payload2.chars().count()
        + c.to_compact_string().chars().count(),
      payload: c.to_compact_string(),
    });
  }
  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 3);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );
  assert_insert(
    &undo_mgr,
    1,
    Insert {
      payload: payload2.to_compact_string(),
      char_idx_before: 3,
      char_idx_after: 3 + payload2.chars().count(),
    },
  );
  assert_insert(
    &undo_mgr,
    2,
    Insert {
      payload: payload3.to_compact_string(),
      char_idx_before: payload1.chars().count() + payload2.chars().count(),
      char_idx_after: payload1.chars().count()
        + payload2.chars().count()
        + payload3.chars().count(),
    },
  );

  let payload4 = "no, it's jerry";
  for (i, c) in payload4.chars().enumerate() {
    undo_mgr.current_mut().insert(Insert {
      payload: c.to_compact_string(),
      char_idx_before: i + 100,
      char_idx_after: i + 100 + 1,
    });
  }
  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 4);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );
  assert_insert(
    &undo_mgr,
    1,
    Insert {
      payload: payload2.to_compact_string(),
      char_idx_before: 3,
      char_idx_after: 3 + payload2.chars().count(),
    },
  );
  assert_insert(
    &undo_mgr,
    2,
    Insert {
      payload: payload3.to_compact_string(),
      char_idx_before: payload1.chars().count() + payload2.chars().count(),
      char_idx_after: payload1.chars().count()
        + payload2.chars().count()
        + payload3.chars().count(),
    },
  );
  assert_insert(
    &undo_mgr,
    3,
    Insert {
      payload: payload4.to_compact_string(),
      char_idx_before: 100,
      char_idx_after: 100 + payload4.chars().count(),
    },
  );

  undo_mgr.commit();

  let actual = undo_mgr.current();
  assert!(actual.records().is_empty());
}

#[test]
fn delete1() {
  let mut undo_mgr = UndoManager::new(MAX_SIZE);
  let payload1 = "Hello, World!";
  for (i, c) in payload1.chars().enumerate() {
    undo_mgr.current_mut().insert(Insert {
      payload: c.to_compact_string(),
      char_idx_before: i,
      char_idx_after: i + 1,
    });
  }

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );

  undo_mgr.current_mut().delete(Delete {
    payload: "!".to_compact_string(),
    char_idx_before: 12,
    char_idx_after: 11,
  });

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 2);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );
  assert_delete(
    &undo_mgr,
    1,
    Delete {
      payload: "!".to_compact_string(),
      char_idx_before: 12,
      char_idx_after: 11,
    },
  );

  let payload2 = "Tom（汤姆） and Jerry（杰瑞）。";
  undo_mgr.current_mut().insert(Insert {
    payload: payload2.to_compact_string(),
    char_idx_before: 12,
    char_idx_after: 12 + payload2.chars().count(),
  });

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 3);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );
  assert_delete(
    &undo_mgr,
    1,
    Delete {
      payload: "!".to_compact_string(),
      char_idx_before: 12,
      char_idx_after: 11,
    },
  );
  assert_insert(
    &undo_mgr,
    2,
    Insert {
      payload: payload2.to_compact_string(),
      char_idx_before: 12,
      char_idx_after: 12 + payload2.chars().count(),
    },
  );

  undo_mgr.current_mut().delete(Delete {
    payload: payload2.to_compact_string(),
    char_idx_before: 12,
    char_idx_after: 12,
  });

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 2);

  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );
  assert_delete(
    &undo_mgr,
    1,
    Delete {
      payload: "!".to_compact_string(),
      char_idx_before: 12,
      char_idx_after: 11,
    },
  );

  undo_mgr.commit();

  let actual = undo_mgr.current();
  assert!(actual.records().is_empty());
}

#[test]
fn delete2() {
  let mut undo_mgr = UndoManager::new(MAX_SIZE);
  let payload1 = "Hello, World!";
  for (i, c) in payload1.chars().enumerate() {
    undo_mgr.current_mut().insert(Insert {
      payload: c.to_compact_string(),
      char_idx_before: i,
      char_idx_after: i + 1,
    });
  }

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );

  undo_mgr.current_mut().delete(Delete {
    payload: "!".to_compact_string(),
    char_idx_before: 12,
    char_idx_after: 11,
  });
  undo_mgr.current_mut().delete(Delete {
    payload: "d".to_compact_string(),
    char_idx_before: 11,
    char_idx_after: 10,
  });
  undo_mgr.current_mut().delete(Delete {
    payload: "l".to_compact_string(),
    char_idx_before: 10,
    char_idx_after: 9,
  });

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 2);

  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );
  assert_delete(
    &undo_mgr,
    1,
    Delete {
      payload: "ld!".to_compact_string(),
      char_idx_before: 12,
      char_idx_after: 9,
    },
  );

  undo_mgr.current_mut().delete(Delete {
    payload: "or".to_compact_string(),
    char_idx_before: 9,
    char_idx_after: 7,
  });

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 2);

  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );
  assert_delete(
    &undo_mgr,
    1,
    Delete {
      payload: "orld!".to_compact_string(),
      char_idx_before: 12,
      char_idx_after: 7,
    },
  );

  undo_mgr.commit();

  let actual = undo_mgr.current();
  assert!(actual.records().is_empty());
}

#[test]
fn delete3() {
  let mut undo_mgr = UndoManager::new(MAX_SIZE);
  let payload1 = "Hello, World!";
  undo_mgr.current_mut().insert(Insert {
    payload: payload1.to_compact_string(),
    char_idx_before: 0,
    char_idx_after: payload1.chars().count(),
  });

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );

  undo_mgr.current_mut().delete(Delete {
    payload: ", ".to_compact_string(),
    char_idx_before: 5,
    char_idx_after: 5,
  });

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 2);

  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );
  assert_delete(
    &undo_mgr,
    1,
    Delete {
      payload: ", ".to_compact_string(),
      char_idx_before: 5,
      char_idx_after: 5,
    },
  );

  undo_mgr.current_mut().delete(Delete {
    payload: "loWo".to_compact_string(),
    char_idx_before: 3,
    char_idx_after: 3,
  });

  let actual = undo_mgr.current();
  assert_eq!(actual.records().len(), 3);

  assert_insert(
    &undo_mgr,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.chars().count(),
    },
  );
  assert_delete(
    &undo_mgr,
    1,
    Delete {
      payload: ", ".to_compact_string(),
      char_idx_before: 5,
      char_idx_after: 5,
    },
  );
  assert_delete(
    &undo_mgr,
    2,
    Delete {
      payload: "loWo".to_compact_string(),
      char_idx_before: 3,
      char_idx_after: 3,
    },
  );
}

#[test]
fn revert1() {
  test_log_init();

  let mut undo_mgr = UndoManager::new(MAX_SIZE);
  let mut text1 = RopeBuilder::new().finish();

  let payload1 = "Hello";
  for (i, c) in payload1.chars().enumerate() {
    text1.insert_char(i, c);
    undo_mgr.current_mut().insert(Insert {
      char_idx_before: i,
      char_idx_after: i + 1,
      payload: c.to_compact_string(),
    });
  }

  let payload2 = ", ";
  text1.insert(payload1.len(), payload2);
  undo_mgr.current_mut().insert(Insert {
    char_idx_before: payload1.len(),
    char_idx_after: payload1.len() + payload2.len(),
    payload: payload2.to_compact_string(),
  });

  let payload3 = "World!";
  text1.insert(payload1.len() + payload2.len(), payload3);
  undo_mgr.current_mut().insert(Insert {
    char_idx_before: payload1.len() + payload2.len(),
    char_idx_after: payload1.len() + payload2.len() + payload3.len(),
    payload: payload3.to_compact_string(),
  });

  undo_mgr.commit();
  info!("undo_mgr:{:?}", undo_mgr);

  // let mut text2 = text1.clone();
  // let result = undo_mgr.revert(commit_idx, buf_id, rope)
}
