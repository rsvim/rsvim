use super::js::*;

#[test]
fn next_handle_id1() {
  assert!(next_handle_id() > 0);
}
