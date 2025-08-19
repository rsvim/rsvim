use super::opt::*;

use crate::defaults;

#[test]
pub fn options1() {
  let opt1 = LocalOptionsBuilder::default()
    .wrap(true)
    .line_break(true)
    .scroll_off(3)
    .build()
    .unwrap();
  assert!(opt1.wrap());
  assert!(opt1.line_break());
  assert_eq!(opt1.scroll_off(), 3);

  let opt2 = LocalOptionsBuilder::default().build().unwrap();
  assert_eq!(opt2.wrap(), defaults::win::WRAP);
  assert_eq!(opt2.line_break(), defaults::win::LINE_BREAK);
  assert_eq!(opt2.scroll_off(), defaults::win::SCROLL_OFF);
}
