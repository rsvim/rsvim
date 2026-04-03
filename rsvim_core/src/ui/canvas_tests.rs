use super::canvas::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use itertools::Itertools;
use std::cell::RefCell;
use std::rc::Rc;

fn int2letter(i: u8) -> char {
  (i + 65) as char
}

#[test]
fn new1() {
  let can = Canvas::new(size!(3, 4));
  assert_eq!(can.frame().size(), can.prev_frame().size());
  assert_eq!(*can.frame().cursor(), *can.prev_frame().cursor());
}

#[test]
fn _shade_cursor1() {
  test_log_init();
  let mut can = Canvas::new(size!(10, 10));

  let cursor1 = Cursor::default();
  can.frame_mut().set_cursor(cursor1);
  let actual1 = Rc::new(RefCell::new(vec![]));
  can._shade_cursor(actual1.clone());
  can._shade_done();
  assert!(actual1.borrow().is_empty());

  let cursor2 =
    Cursor::new(point!(3, 7), false, true, CursorStyle::BlinkingBar);
  can.frame_mut().set_cursor(cursor2);
  let actual2 = Rc::new(RefCell::new(vec![]));
  can._shade_cursor(actual2.clone());
  can._shade_done();
  info!("actual2:{:?}", actual2.borrow());
  assert!(!actual2.borrow().is_empty());
  assert_eq!(actual2.borrow().len(), 3);
  assert_eq!(
    actual2
      .borrow()
      .iter()
      .filter(|sh| {
        if let ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(x, y)) = sh
        {
          *x == 3 && *y == 7
        } else {
          false
        }
      })
      .collect::<Vec<_>>()
      .len(),
    1
  );
  assert_eq!(
    actual2
      .borrow()
      .iter()
      .filter(|sh| {
        matches!(
          sh,
          ShaderCommand::CursorDisableBlinking(
            crossterm::cursor::DisableBlinking
          )
        )
      })
      .collect::<Vec<_>>()
      .len(),
    0
  );
  assert_eq!(
    actual2
      .borrow()
      .iter()
      .filter(|sh| {
        matches!(sh, ShaderCommand::CursorHide(crossterm::cursor::Hide))
      })
      .collect::<Vec<_>>()
      .len(),
    1
  );
  assert_eq!(
    actual2
      .borrow()
      .iter()
      .filter(|sh| {
        matches!(
          sh,
          ShaderCommand::CursorSetCursorStyle(
            crossterm::cursor::SetCursorStyle::BlinkingBar
          )
        )
      })
      .collect::<Vec<_>>()
      .len(),
    1
  );

  let cursor3 =
    Cursor::new(point!(4, 5), true, true, CursorStyle::SteadyUnderScore);
  can.frame_mut().set_cursor(cursor3);
  let actual3 = Rc::new(RefCell::new(vec![]));
  can._shade_cursor(actual3.clone());
  can._shade_done();
  info!("actual3:{:?}", actual3.borrow());
  assert_eq!(actual3.borrow().len(), 3);
  assert_eq!(
    actual3
      .borrow()
      .iter()
      .filter(|sh| {
        if let ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(x, y)) = sh
        {
          *x == 4 && *y == 5
        } else {
          false
        }
      })
      .collect::<Vec<_>>()
      .len(),
    1
  );
  assert_eq!(
    actual3
      .borrow()
      .iter()
      .filter(|sh| {
        matches!(
          sh,
          ShaderCommand::CursorEnableBlinking(
            crossterm::cursor::EnableBlinking
          )
        )
      })
      .collect::<Vec<_>>()
      .len(),
    1
  );
  assert_eq!(
    actual3
      .borrow()
      .iter()
      .filter(|sh| {
        matches!(
          sh,
          ShaderCommand::CursorSetCursorStyle(
            crossterm::cursor::SetCursorStyle::SteadyUnderScore
          )
        )
      })
      .collect::<Vec<_>>()
      .len(),
    1
  );
}

#[test]
fn _make_consequential_shaders1() {
  test_log_init();
  let mut can = Canvas::new(size!(10, 10));

  can.frame_mut().set_cells_at(
    point!(2, 3),
    &(0..4).map(|i| Cell::with_char(int2letter(i))).collect_vec(),
  );
  let col = 2;
  let row = 3;
  let mut shaders = vec![];
  can._make_consequential_shaders(row, col, col + 4, &mut shaders);
  info!("shader:{:?}", shaders);
  assert!(matches!(
    shaders[0],
    ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(_, _))
  ));
  assert!(matches!(
    shaders[1],
    ShaderCommand::StylePrintStyledContentString(
      crossterm::style::PrintStyledContent(_)
    )
  ));
  if let ShaderCommand::StylePrintStyledContentString(
    crossterm::style::PrintStyledContent(contents),
  ) = &shaders[1]
  {
    assert_eq!(contents.content().to_string(), "ABCD".to_string());
  }
}

#[test]
fn diff1() {
  test_log_init();
  let mut can = Canvas::new(size!(10, 10));

  can.frame_mut().set_cells_at(
    point!(2, 3),
    &(0..4).map(|i| Cell::with_char(int2letter(i))).collect_vec(),
  );
  let mut actual1 = vec![];
  let mut actual2 = vec![];
  can._dirty_marks_diff(&mut actual1);
  can._brute_force_diff(&mut actual2);
  info!("dirty marks:{:?}", actual1);
  info!("brute force:{:?}", actual2);
  assert_eq!(actual1.len(), 2);
  assert!(matches!(
    actual1[0],
    ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(_, _))
  ));
  assert!(matches!(
    actual1[1],
    ShaderCommand::StylePrintStyledContentString(
      crossterm::style::PrintStyledContent(_)
    )
  ));
  if let ShaderCommand::StylePrintStyledContentString(
    crossterm::style::PrintStyledContent(contents),
  ) = &actual1[1]
  {
    assert_eq!(contents.content().to_string(), "ABCD".to_string());
  }
  assert!(actual2.len() > 10);
  assert!(matches!(
    actual2[0],
    ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(_, _))
  ));
  assert!(matches!(
    actual2[1],
    ShaderCommand::StylePrintStyledContentString(
      crossterm::style::PrintStyledContent(_)
    )
  ));
  if let ShaderCommand::StylePrintStyledContentString(
    crossterm::style::PrintStyledContent(contents),
  ) = &actual2[1]
  {
    assert_eq!(contents.content().to_string(), "          ".to_string());
  }
  assert!(matches!(
    actual2[7],
    ShaderCommand::StylePrintStyledContentString(
      crossterm::style::PrintStyledContent(_)
    )
  ));
  if let ShaderCommand::StylePrintStyledContentString(
    crossterm::style::PrintStyledContent(contents),
  ) = &actual2[7]
  {
    assert_eq!(contents.content().to_string(), "  ABCD    ".to_string());
  }
}

#[test]
fn diff2() {
  test_log_init();
  let mut can = Canvas::new(size!(10, 10));

  can.frame_mut().set_cells_at(
    point!(2, 3),
    &(0..4).map(|i| Cell::with_char(int2letter(i))).collect_vec(),
  );
  can.frame_mut().set_cells_at(
    point!(8, 5),
    &(5..9).map(|i| Cell::with_char(int2letter(i))).collect_vec(),
  );
  let mut actual = vec![];
  can._dirty_marks_diff(&mut actual);
  for (i, act) in actual.iter().enumerate() {
    info!("diff2 [{}]:{:?}", i, act);
  }
  assert_eq!(actual.len(), 6);

  // section-1
  assert!(matches!(
    actual[0],
    ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(2, 3))
  ));
  assert!(matches!(
    actual[1],
    ShaderCommand::StylePrintStyledContentString(
      crossterm::style::PrintStyledContent(_)
    )
  ));
  if let ShaderCommand::StylePrintStyledContentString(
    crossterm::style::PrintStyledContent(contents),
  ) = &actual[1]
  {
    assert_eq!(contents.content().to_string(), "ABCD".to_string());
  }

  // section-2
  assert!(matches!(
    actual[2],
    ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(8, 5))
  ));
  assert!(matches!(
    actual[3],
    ShaderCommand::StylePrintStyledContentString(
      crossterm::style::PrintStyledContent(_)
    )
  ));
  if let ShaderCommand::StylePrintStyledContentString(
    crossterm::style::PrintStyledContent(contents),
  ) = &actual[3]
  {
    assert_eq!(contents.content().to_string(), "FG".to_string());
  }

  // section-3
  assert!(matches!(
    actual[4],
    ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(0, 6))
  ));
  assert!(matches!(
    actual[5],
    ShaderCommand::StylePrintStyledContentString(
      crossterm::style::PrintStyledContent(_)
    )
  ));
  if let ShaderCommand::StylePrintStyledContentString(
    crossterm::style::PrintStyledContent(contents),
  ) = &actual[5]
  {
    assert_eq!(contents.content().to_string(), "HI".to_string());
  }
}
