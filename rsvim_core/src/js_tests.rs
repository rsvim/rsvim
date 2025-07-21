use super::js::*;

#[test]
fn next_future_id1() {
  assert!(next_future_id() > 0);
}
