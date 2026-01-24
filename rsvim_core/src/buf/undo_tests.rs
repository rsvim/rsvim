use super::undo::*;
use compact_str::ToCompactString;

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
  let mut undo_manager = UndoManager::new();
  let payload = "Hello, World!";
  for (i, c) in payload.chars().enumerate() {
    undo_manager.insert(Insert {
      payload: c.to_compact_string(),
      char_idx_before: i,
      char_idx_after: i + c.to_compact_string().chars().count(),
    });
  }
  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(
    &undo_manager,
    0,
    Insert {
      payload: payload.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload.to_compact_string().chars().count(),
    },
  );
  undo_manager.commit();

  let actual = undo_manager.current();
  assert!(actual.records().is_empty());
}

#[test]
fn insert2() {
  let mut undo_manager = UndoManager::new();
  let payload1 = "Hello, ";
  for (i, c) in payload1.chars().enumerate() {
    undo_manager.insert(Insert {
      payload: c.to_compact_string(),
      char_idx_before: i,
      char_idx_after: i + 1,
    });
  }
  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(
    &undo_manager,
    0,
    Insert {
      payload: payload1.to_compact_string(),
      char_idx_before: 0,
      char_idx_after: payload1.to_compact_string().chars().count(),
    },
  );

  let payload2 = "World!";
  for (i, c) in payload2.chars().enumerate() {
    undo_manager.insert(Insert {
      char_idx_before: i + 3,
      char_idx_after: i + 4,
      payload: c.to_compact_string(),
    });
  }
  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(
    &undo_manager,
    0,
    Insert {
      payload: "HelWorld!lo, ".to_compact_string(),
      char_idx_before: 0,
      char_idx_after: "HelWorld!lo, ".chars().count(),
    },
  );

  let payload3 = "汤姆(Tom)?";
  for (i, c) in payload3.chars().enumerate() {
    undo_manager.insert(Insert {
      char_idx_before: i + payload1.chars().count() + payload2.chars().count(),
      char_idx_after: i
        + payload1.chars().count()
        + payload2.chars().count()
        + c.to_compact_string().chars().count(),
      payload: c.to_compact_string(),
    });
  }
  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(
    &undo_manager,
    0,
    Insert {
      payload: "HelWorld!lo, 汤姆(Tom)?".to_compact_string(),
      char_idx_before: 0,
      char_idx_after: "HelWorld!lo, 汤姆(Tom)?".chars().count(),
    },
  );

  let payload4 = "no, it's jerry";
  for (i, c) in payload4.chars().enumerate() {
    undo_manager.insert(Insert {
      payload: c.to_compact_string(),
      char_idx_before: i + 100,
      char_idx_after: i + 100 + 1,
    });
  }
  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 2);
  assert_insert(
    &undo_manager,
    0,
    Insert {
      payload: "HelWorld!lo, 汤姆(Tom)?".to_compact_string(),
      char_idx_before: 0,
      char_idx_after: "HelWorld!lo, 汤姆(Tom)?"
        .to_compact_string()
        .chars()
        .count(),
    },
  );
  assert_insert(
    &undo_manager,
    1,
    Insert {
      payload: "no, it's jerry".to_compact_string(),
      char_idx_before: 100,
      char_idx_after: 100 + "no, it's jerry".chars().count(),
    },
  );

  undo_manager.commit();

  let actual = undo_manager.current();
  assert!(actual.records().is_empty());
}

#[test]
fn delete1() {
  let mut undo_manager = UndoManager::new();
  let payload1 = "Hello, World!";
  for (i, c) in payload1.chars().enumerate() {
    undo_manager.insert(i, c.to_compact_string());
  }

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(&undo_manager, 0, 0, payload1);

  undo_manager.delete(12, "!".to_compact_string());

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 2);
  assert_insert(&undo_manager, 0, 0, payload1);
  assert_delete(&undo_manager, 1, 12, "!");

  let payload2 = "Tom（汤姆） and Jerry（杰瑞）。";
  undo_manager.insert(12, payload2.to_compact_string());

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 3);
  assert_insert(&undo_manager, 0, 0, payload1);
  assert_delete(&undo_manager, 1, 12, "!");
  assert_insert(&undo_manager, 2, 12, payload2);

  undo_manager.delete(12, payload2.to_compact_string());

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 2);

  assert_insert(&undo_manager, 0, 0, payload1);
  assert_delete(&undo_manager, 1, 12, "!");

  undo_manager.commit();

  let actual = undo_manager.current();
  assert!(actual.records().is_empty());
}

#[test]
fn delete2() {
  let mut undo_manager = UndoManager::new();
  let payload1 = "Hello, World!";
  for (i, c) in payload1.chars().enumerate() {
    undo_manager.insert(i, c.to_compact_string());
  }

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(&undo_manager, 0, 0, payload1);

  undo_manager.delete(12, "!".to_compact_string());
  undo_manager.delete(11, "d".to_compact_string());
  undo_manager.delete(10, "l".to_compact_string());

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 2);

  assert_insert(&undo_manager, 0, 0, payload1);
  assert_delete(&undo_manager, 1, 10, "ld!");

  undo_manager.delete(8, "or".to_compact_string());

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 2);

  assert_insert(&undo_manager, 0, 0, payload1);
  assert_delete(&undo_manager, 1, 8, "orld!");

  undo_manager.commit();

  let actual = undo_manager.current();
  assert!(actual.records().is_empty());
}

#[test]
fn delete3() {
  let mut undo_manager = UndoManager::new();
  let payload1 = "Hello, World!";
  undo_manager.insert(0, payload1.to_compact_string());

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 1);
  assert_insert(&undo_manager, 0, 0, payload1);

  undo_manager.delete(5, ", ".to_compact_string());

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 2);

  assert_insert(&undo_manager, 0, 0, payload1);
  assert_delete(&undo_manager, 1, 5, ", ");

  undo_manager.delete(3, "loWo".to_compact_string());

  let actual = undo_manager.current();
  assert_eq!(actual.records().len(), 2);

  assert_insert(&undo_manager, 0, 0, payload1);
  assert_delete(&undo_manager, 1, 3, "lo, Wo");
}
