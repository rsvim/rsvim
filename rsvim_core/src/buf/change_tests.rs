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
}
