use super::opt::*;

#[test]
pub fn options1() {
  let opt1 = WindowOptionsBuilder::default()
    .wrap(true)
    .line_break(true)
    .scroll_off(3)
    .build()
    .unwrap();
  assert!(opt1.wrap());
  assert!(opt1.line_break());
  assert_eq!(opt1.scroll_off(), 3);

  let opt2 = WindowOptionsBuilder::default().build().unwrap();
  assert_eq!(opt2.wrap(), WRAP);
  assert_eq!(opt2.line_break(), LINE_BREAK);
  assert_eq!(opt2.scroll_off(), SCROLL_OFF);
}
