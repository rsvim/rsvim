use super::opt::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;

#[test]
fn default1() {
  let opt1 = BufferOptionsBuilder::default().build().unwrap();
  assert_eq!(opt1.tab_stop(), TAB_STOP);
  assert_eq!(opt1.file_encoding(), FILE_ENCODING);
}

#[test]
fn fmt1() {
  test_log_init();
  let opt1 = BufferOptionsBuilder::default().build().unwrap();
  info!("opt1:{:?}", opt1);
}
