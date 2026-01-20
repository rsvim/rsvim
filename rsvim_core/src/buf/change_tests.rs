use super::change::*;
use compact_str::ToCompactString;

#[test]
fn insert1() {
  let mut change_manager = ChangeManager::new();
  let payload = "Hello, World!";
  for (i, c) in payload.chars().enumerate() {
    change_manager.save(Operation::Insert(Insert {
      char_idx: i,
      payload: c.to_string().to_compact_string(),
    }));
  }
  let actual = change_manager.current_change();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);
  let actual = &actual.operations()[0];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert_actual) => {
      assert_eq!(insert_actual.payload, payload);
      assert_eq!(insert_actual.char_idx, 0);
    }
    _ => unreachable!(),
  }
  change_manager.commit();
}

#[test]
fn insert2() {
  let mut change_manager = ChangeManager::new();
  let payload1 = "Hello, ";
  for (i, c) in payload1.chars().enumerate() {
    change_manager.save(Operation::Insert(Insert {
      char_idx: i,
      payload: c.to_string().to_compact_string(),
    }));
  }
  let actual = change_manager.current_change();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);
  let actual = &actual.operations()[0];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert_actual) => {
      assert_eq!(insert_actual.payload, payload1);
      assert_eq!(insert_actual.char_idx, 0);
    }
    _ => unreachable!(),
  }

  let payload2 = "World!";
  for (i, c) in payload2.chars().enumerate() {
    change_manager.save(Operation::Insert(Insert {
      char_idx: i + 3,
      payload: c.to_string().to_compact_string(),
    }));
  }
  let actual = change_manager.current_change();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);
  let actual = &actual.operations()[0];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert_actual) => {
      assert_eq!(insert_actual.payload, "HelWorld!lo, ");
      assert_eq!(insert_actual.char_idx, 0);
    }
    _ => unreachable!(),
  }

  change_manager.commit();
}
