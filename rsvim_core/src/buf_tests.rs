use super::buf::*;

#[test]
fn next_buffer_id1() {
  assert!(next_buffer_id() > 0);
}
