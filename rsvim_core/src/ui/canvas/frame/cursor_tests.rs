use super::cursor::*;
use crate::point;
use crate::prelude::*;

#[test]
fn default1() {
  let c = Cursor::default();
  assert!(!c.blinking());
  assert!(!c.hidden());
  assert_eq!(c.style(), CursorStyle::SteadyBlock);
}

#[test]
fn debug1() {
  let cursors = [
    Cursor::default(),
    Cursor::new(
      point!(0_u16, 10_u16),
      false,
      true,
      CursorStyle::SteadyUnderScore,
    ),
    Cursor::new(point!(7_u16, 3_u16), true, false, CursorStyle::BlinkingBar),
  ];
  for c in cursors.iter() {
    info!("{:?}", c);
  }
}
