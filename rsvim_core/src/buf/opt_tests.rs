use super::opt::*;
use crate::defaults;

#[test]
fn default1() {
  let opt1 = BufferOptionsBuilder::default().build().unwrap();
  assert_eq!(opt1.tab_stop(), TAB_STOP);
  assert_eq!(opt1.file_encoding(), FILE_ENCODING);
}
