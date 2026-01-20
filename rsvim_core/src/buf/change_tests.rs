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
    }
    _ => unreachable!(),
  }
  change_manager.commit();
}
