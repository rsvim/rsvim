use super::undo::*;
use compact_str::ToCompactString;

fn assert_insert(
  undo_manager: &UndoManager,
  op_idx: usize,
  char_idx: usize,
  payload: &str,
) {
  assert!(undo_manager.current().operations().len() > op_idx);
  let actual = &undo_manager.current().operations()[op_idx];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert) => {
      assert_eq!(insert.payload, payload);
      assert_eq!(insert.char_idx, char_idx);
    }
    _ => unreachable!(),
  }
}

fn assert_delete(
  undo_manager: &UndoManager,
  op_idx: usize,
  char_idx: usize,
  payload: &str,
) {
  assert!(undo_manager.current().operations().len() > op_idx);
  let actual = &undo_manager.current().operations()[op_idx];
  assert!(matches!(actual, Operation::Delete(_)));
  match actual {
    Operation::Delete(delete) => {
      assert_eq!(delete.payload, payload);
      assert_eq!(delete.char_idx, char_idx);
    }
    _ => unreachable!(),
  }
}

#[test]
fn insert1() {
  let mut undo_manager = UndoManager::new();
  let payload = "Hello, World!";
  for (i, c) in payload.chars().enumerate() {
    undo_manager.save(Operation::Insert(Insert {
      char_idx: i,
      payload: c.to_string().to_compact_string(),
    }));
  }
  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);
  assert_insert(&undo_manager, 0, 0, payload);
  undo_manager.commit();

  let actual = undo_manager.current();
  assert!(actual.operations().is_empty());
  assert_eq!(actual.version(), 2);
}

#[test]
fn insert2() {
  let mut undo_manager = UndoManager::new();
  let payload1 = "Hello, ";
  for (i, c) in payload1.chars().enumerate() {
    undo_manager.save(Operation::Insert(Insert {
      char_idx: i,
      payload: c.to_string().to_compact_string(),
    }));
  }
  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);
  assert_insert(&undo_manager, 0, 0, payload1);

  let payload2 = "World!";
  for (i, c) in payload2.chars().enumerate() {
    undo_manager.save(Operation::Insert(Insert {
      char_idx: i + 3,
      payload: c.to_string().to_compact_string(),
    }));
  }
  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);
  assert_insert(&undo_manager, 0, 0, "HelWorld!lo, ");

  let payload3 = "汤姆(Tom)?";
  for (i, c) in payload3.chars().enumerate() {
    undo_manager.save(Operation::Insert(Insert {
      char_idx: i + payload1.chars().count() + payload2.chars().count(),
      payload: c.to_string().to_compact_string(),
    }));
  }
  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);
  assert_insert(&undo_manager, 0, 0, "HelWorld!lo, 汤姆(Tom)?");

  let payload4 = "no, it's jerry";
  for (i, c) in payload4.chars().enumerate() {
    undo_manager.save(Operation::Insert(Insert {
      char_idx: i + 100,
      payload: c.to_string().to_compact_string(),
    }));
  }
  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 2);
  assert_eq!(actual.version(), 1);
  assert_insert(&undo_manager, 0, 0, "HelWorld!lo, 汤姆(Tom)?");
  assert_insert(&undo_manager, 0, 1, "no, it's jerry");

  undo_manager.commit();

  let actual = undo_manager.current();
  assert!(actual.operations().is_empty());
  assert_eq!(actual.version(), 2);
}

#[test]
fn delete1() {
  let mut undo_manager = UndoManager::new();
  let payload1 = "Hello, World!";
  for (i, c) in payload1.chars().enumerate() {
    undo_manager.save(Operation::Insert(Insert {
      char_idx: i,
      payload: c.to_string().to_compact_string(),
    }));
  }

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);
  assert_insert(&undo_manager, 0, 0, payload1);

  undo_manager.save(Operation::Delete(Delete {
    char_idx: 12,
    payload: "!".to_compact_string(),
  }));

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 2);
  assert_eq!(actual.version(), 1);
  assert_insert(&undo_manager, 0, 0, payload1);
  assert_insert(&undo_manager, 1, 12, "!");

  let payload2 = "Tom（汤姆） and Jerry（杰瑞）。";
  undo_manager.save(Operation::Insert(Insert {
    char_idx: 12,
    payload: payload2.to_compact_string(),
  }));

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 3);
  assert_eq!(actual.version(), 1);
  assert_insert(&undo_manager, 0, 0, payload1);
  assert_delete(&undo_manager, 1, 12, "!");
  assert_delete(&undo_manager, 2, 12, payload2);

  undo_manager.save(Operation::Delete(Delete {
    char_idx: 12,
    payload: payload2.to_compact_string(),
  }));

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 2);
  assert_eq!(actual.version(), 1);

  assert_insert(&undo_manager, 0, 0, payload1);
  assert_insert(&undo_manager, 1, 12, "!");

  undo_manager.commit();

  let actual = undo_manager.current();
  assert!(actual.operations().is_empty());
  assert_eq!(actual.version(), 2);
}

#[test]
fn delete2() {
  let mut undo_manager = UndoManager::new();
  let payload1 = "Hello, World!";
  for (i, c) in payload1.chars().enumerate() {
    undo_manager.save(Operation::Insert(Insert {
      char_idx: i,
      payload: c.to_string().to_compact_string(),
    }));
  }

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);

  let actual = &undo_manager.current().operations()[0];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert) => {
      assert_eq!(insert.payload, "Hello, World!");
      assert_eq!(insert.char_idx, 0);
    }
    _ => unreachable!(),
  }

  undo_manager.save(Operation::Delete(Delete {
    char_idx: 12,
    payload: "!".to_compact_string(),
  }));
  undo_manager.save(Operation::Delete(Delete {
    char_idx: 11,
    payload: "d".to_compact_string(),
  }));
  undo_manager.save(Operation::Delete(Delete {
    char_idx: 10,
    payload: "l".to_compact_string(),
  }));

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 2);
  assert_eq!(actual.version(), 1);

  let actual = &undo_manager.current().operations()[0];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert) => {
      assert_eq!(insert.payload, "Hello, World!");
      assert_eq!(insert.char_idx, 0);
    }
    _ => unreachable!(),
  }
  let actual = &undo_manager.current().operations()[1];
  assert!(matches!(actual, Operation::Delete(_)));
  match actual {
    Operation::Delete(delete) => {
      assert_eq!(delete.char_idx, 10);
      assert_eq!(delete.payload, "ld!");
    }
    _ => unreachable!(),
  }

  undo_manager.save(Operation::Delete(Delete {
    char_idx: 8,
    payload: "or".to_compact_string(),
  }));

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 2);
  assert_eq!(actual.version(), 1);

  let actual = &undo_manager.current().operations()[0];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert) => {
      assert_eq!(insert.payload, "Hello, World!");
      assert_eq!(insert.char_idx, 0);
    }
    _ => unreachable!(),
  }
  let actual = &undo_manager.current().operations()[1];
  assert!(matches!(actual, Operation::Delete(_)));
  match actual {
    Operation::Delete(delete) => {
      assert_eq!(delete.char_idx, 8);
      assert_eq!(delete.payload, "orld!");
    }
    _ => unreachable!(),
  }

  undo_manager.commit();

  let actual = undo_manager.current();
  assert!(actual.operations().is_empty());
  assert_eq!(actual.version(), 2);
}

#[test]
fn delete3() {
  let mut undo_manager = UndoManager::new();
  let payload1 = "Hello, World!";
  for (i, c) in payload1.chars().enumerate() {
    undo_manager.save(Operation::Insert(Insert {
      char_idx: i,
      payload: c.to_string().to_compact_string(),
    }));
  }

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 1);
  assert_eq!(actual.version(), 1);

  let actual = &undo_manager.current().operations()[0];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert) => {
      assert_eq!(insert.payload, "Hello, World!");
      assert_eq!(insert.char_idx, 0);
    }
    _ => unreachable!(),
  }

  undo_manager.save(Operation::Delete(Delete {
    char_idx: 12,
    payload: "!".to_compact_string(),
  }));
  undo_manager.save(Operation::Delete(Delete {
    char_idx: 11,
    payload: "d".to_compact_string(),
  }));
  undo_manager.save(Operation::Delete(Delete {
    char_idx: 10,
    payload: "l".to_compact_string(),
  }));

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 2);
  assert_eq!(actual.version(), 1);

  let actual = &undo_manager.current().operations()[0];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert) => {
      assert_eq!(insert.payload, "Hello, World!");
      assert_eq!(insert.char_idx, 0);
    }
    _ => unreachable!(),
  }
  let actual = &undo_manager.current().operations()[1];
  assert!(matches!(actual, Operation::Delete(_)));
  match actual {
    Operation::Delete(delete) => {
      assert_eq!(delete.char_idx, 10);
      assert_eq!(delete.payload, "ld!");
    }
    _ => unreachable!(),
  }

  undo_manager.save(Operation::Delete(Delete {
    char_idx: 8,
    payload: "or".to_compact_string(),
  }));

  let actual = undo_manager.current();
  assert_eq!(actual.operations().len(), 2);
  assert_eq!(actual.version(), 1);

  let actual = &undo_manager.current().operations()[0];
  assert!(matches!(actual, Operation::Insert(_)));
  match actual {
    Operation::Insert(insert) => {
      assert_eq!(insert.payload, "Hello, World!");
      assert_eq!(insert.char_idx, 0);
    }
    _ => unreachable!(),
  }
  let actual = &undo_manager.current().operations()[1];
  assert!(matches!(actual, Operation::Delete(_)));
  match actual {
    Operation::Delete(delete) => {
      assert_eq!(delete.char_idx, 8);
      assert_eq!(delete.payload, "orld!");
    }
    _ => unreachable!(),
  }

  undo_manager.commit();

  let actual = undo_manager.current();
  assert!(actual.operations().is_empty());
  assert_eq!(actual.version(), 2);
}
