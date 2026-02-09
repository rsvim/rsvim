use super::buf::*;

#[test]
fn next_buffer_id1() {
  assert!(BufferId::next() > 0);
}
