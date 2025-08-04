use super::canvas::*;

use crate::prelude::*;
use crate::test::log::init as test_log_init;

use compact_str::ToCompactString;
use geo::point;

fn int2letter(i: u8) -> char {
  (i + 65) as char
}

#[test]
fn new1() {
  let can = Canvas::new(U16Size::new(3, 4));
  assert_eq!(can.frame().size(), can.prev_frame().size());
  assert_eq!(*can.frame().cursor(), *can.prev_frame().cursor());
}

#[test]
fn _shade_cursor1() {
  test_log_init();
  let mut can = Canvas::new(U16Size::new(10, 10));

  let cursor1 = Cursor::default();
  can.frame_mut().set_cursor(cursor1);
  let actual1 = can._shade_cursor();
  can._shade_done();
  assert!(actual1.is_empty());

  let cursor2 =
    Cursor::new(point!(x:3, y:7), false, true, CursorStyle::BlinkingBar);
  can.frame_mut().set_cursor(cursor2);
  let actual2 = can._shade_cursor();
  can._shade_done();
  info!("actual2:{:?}", actual2);
  assert!(!actual2.is_empty());
  assert_eq!(actual2.len(), 3);
  assert_eq!(
    actual2
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
    Cursor::new(point!(x:4, y:5), true, true, CursorStyle::SteadyUnderScore);
  can.frame_mut().set_cursor(cursor3);
  let actual3 = can._shade_cursor();
  can._shade_done();
  info!("actual3:{:?}", actual3);
  assert_eq!(actual3.len(), 3);
  assert_eq!(
    actual3
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
fn _next_same_cell_in_row1() {
  test_log_init();
  let mut can = Canvas::new(U16Size::new(10, 10));

  can
    .frame_mut()
    .set_cells_at(point!(x:0,y:0), vec![Cell::with_char('A'); 20]);
  for i in 0..10 {
    let actual = can._next_same_cell_in_row(0, i);
    info!("1-{:?} actual:{:?}", i, actual);
    assert_eq!(actual, 10);
  }
  for i in 0..10 {
    let actual = can._next_same_cell_in_row(1, i);
    info!("2-{:?} actual:{:?}", i, actual);
    assert_eq!(actual, 10);
  }
}

#[test]
fn _next_same_cell_in_row2() {
  test_log_init();
  let mut can = Canvas::new(U16Size::new(10, 10));

  can.frame_mut().set_cells_at(
    point!(x:3,y:5),
    (0..9)
      .map(|i| Cell::with_char(int2letter(i)))
      .collect::<Vec<_>>(),
  );
  let chars = (0_u8..9_u8)
    .map(|i| int2letter(i).to_compact_string())
    .collect::<Vec<_>>();
  info!(
    "frame:{:?}",
    can
      .frame()
      .raw_symbols()
      .iter()
      .map(|cs| cs
        .iter()
        .map(CompactString::to_string)
        .collect::<Vec<_>>()
        .join(""))
      .collect::<Vec<_>>()
  );
  for col in 0..10 {
    for row in 0..10 {
      let actual = can._next_same_cell_in_row(row, col);
      info!("row:{:?}, col:{:?}, actual:{:?}", row, col, actual);
      if !(5..7).contains(&row) {
        assert_eq!(actual, col);
      } else if row == 5 && (3..10).contains(&col) {
        assert_eq!(actual, 10);
        info!(
          "chars:{:?}, symbol:{:?}",
          chars,
          can.frame().get_cell(point!(x:col, y:row)).symbol()
        );
        assert!(
          chars.contains(can.frame().get_cell(point!(x:col, y:row)).symbol())
        );
      } else if row == 6 && (0..2).contains(&col) {
        assert_eq!(actual, 2);
        info!(
          "chars:{:?}, symbol:{:?}",
          chars,
          can.frame().get_cell(point!(x:col, y:row)).symbol()
        );
        assert!(
          chars.contains(can.frame().get_cell(point!(x:col, y:row)).symbol())
        );
      } else {
        assert_eq!(actual, col);
      }
    }
  }
}

#[test]
fn _next_same_cell_in_row3() {
  test_log_init();
  let mut can = Canvas::new(U16Size::new(10, 10));

  can.frame_mut().set_cells_at(
    point!(x:2,y:3),
    (0..4)
      .map(|i| Cell::with_char(int2letter(i)))
      .collect::<Vec<_>>(),
  );
  let mut char_index = 0_u8;
  info!(
    "frame:{:?}",
    can
      .frame()
      .raw_symbols()
      .iter()
      .map(|cs| cs
        .iter()
        .map(CompactString::to_string)
        .collect::<Vec<_>>()
        .join(""))
      .collect::<Vec<_>>()
  );
  for col in 0..10 {
    for row in 0..10 {
      let actual = can._next_same_cell_in_row(row, col);
      info!("row:{:?}, col:{:?}, actual:{:?}", row, col, actual);
      if row != 3 {
        assert_eq!(actual, col);
      } else if (2..6).contains(&col) {
        assert_eq!(actual, 6);
        assert_eq!(
          int2letter(char_index).to_compact_string(),
          can.frame().get_cell(point!(x:col, y:row)).symbol()
        );
        char_index += 1;
      } else {
        assert_eq!(actual, col);
      }
    }
  }
}

#[test]
fn _make_printable_shader1() {
  test_log_init();
  let mut can = Canvas::new(U16Size::new(10, 10));

  can.frame_mut().set_cells_at(
    point!(x:2,y:3),
    (0..4)
      .map(|i| Cell::with_char(int2letter(i)))
      .collect::<Vec<_>>(),
  );
  let col = 2;
  let row = 3;
  let col_end_at = can._next_same_cell_in_row(row, col);
  let shaders = can._make_printable_shaders(row, col, col_end_at);
  info!("shader:{:?}", shaders);
  assert_eq!(shaders.len(), 2);
  assert!(matches!(
    shaders[0],
    ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(_, _))
  ));
  assert!(matches!(
    shaders[1],
    ShaderCommand::StylePrintString(crossterm::style::Print(_))
  ));
  if let ShaderCommand::StylePrintString(crossterm::style::Print(contents)) =
    &shaders[1]
  {
    assert_eq!(*contents, "ABCD".to_string());
  }
}

#[test]
fn diff1() {
  test_log_init();
  let mut can = Canvas::new(U16Size::new(10, 10));

  can.frame_mut().set_cells_at(
    point!(x:2,y:3),
    (0..4)
      .map(|i| Cell::with_char(int2letter(i)))
      .collect::<Vec<_>>(),
  );
  let actual1 = can._dirty_marks_diff();
  let actual2 = can._brute_force_diff();
  info!("dirty marks:{:?}", actual1);
  info!("brute force:{:?}", actual2);
  assert_eq!(actual1.len(), 2);
  assert!(matches!(
    actual1[0],
    ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(_, _))
  ));
  assert!(matches!(
    actual1[1],
    ShaderCommand::StylePrintString(crossterm::style::Print(_))
  ));
  if let ShaderCommand::StylePrintString(crossterm::style::Print(contents)) =
    &actual1[1]
  {
    assert_eq!(*contents, "ABCD".to_string());
  }
  assert_eq!(actual2.len(), 2);
  assert!(matches!(
    actual2[0],
    ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(_, _))
  ));
  assert!(matches!(
    actual2[1],
    ShaderCommand::StylePrintString(crossterm::style::Print(_))
  ));
  if let ShaderCommand::StylePrintString(crossterm::style::Print(contents)) =
    &actual2[1]
  {
    assert_eq!(*contents, "ABCD".to_string());
  }
}
