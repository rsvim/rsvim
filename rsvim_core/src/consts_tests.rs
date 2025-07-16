use super::consts::*;

#[test]
fn mutex_timeout1() {
  assert!(MUTEX_TIMEOUT_SECS() > 0);
}
