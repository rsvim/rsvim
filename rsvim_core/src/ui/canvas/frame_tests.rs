use super::frame::*;

use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use crate::ui::canvas::frame::cell::Cell;
use crate::ui::canvas::frame::cursor::Cursor;

use compact_str::{CompactString, ToCompactString};
use crossterm::style::{Attributes, Color};
use geo::point;

#[test]
fn pos2range1() {
  let frame_size = U16Size::new(10, 10);
  let frame = Frame::new(frame_size, Cursor::default());
  assert_eq!(frame.pos2range(point!(x: 0, y:0), 7), 0..7);
  assert_eq!(frame.pos2range(point!(x: 7, y:2), 23), 27..50);
  assert_eq!(frame.pos2range(point!(x: 8, y:9), 1), 98..99);
  assert_eq!(frame.pos2range(point!(x: 9, y:9), 1), 99..100);
}

#[test]
fn idx2range1() {
  let frame_size = U16Size::new(10, 10);
  let frame = Frame::new(frame_size, Cursor::default());
  assert_eq!(frame.idx2range(0, 7), 0..7);
  assert_eq!(frame.idx2range(27, 23), 27..50);
  assert_eq!(frame.idx2range(98, 1), 98..99);
  assert_eq!(frame.idx2range(99, 1), 99..100);
}

#[test]
fn xy2idx1() {
  let frame_size = U16Size::new(10, 10);
  let frame = Frame::new(frame_size, Cursor::default());
  assert_eq!(frame.xy2idx(0, 7), 70);
  assert_eq!(frame.xy2idx(7, 3), 37);
  assert_eq!(frame.xy2idx(1, 0), 1);
  assert_eq!(frame.xy2idx(0, 9), 90);
  assert_eq!(frame.xy2idx(9, 9), 99);
}

#[test]
fn pos2idx1() {
  let frame_size = U16Size::new(10, 10);
  let frame = Frame::new(frame_size, Cursor::default());
  assert_eq!(frame.pos2idx(point!(x:0, y:7)), 70);
  assert_eq!(frame.pos2idx(point!(x:7, y:3)), 37);
  assert_eq!(frame.pos2idx(point!(x:1, y:0)), 1);
  assert_eq!(frame.pos2idx(point!(x:0, y:9)), 90);
  assert_eq!(frame.pos2idx(point!(x:9, y:9)), 99);
}

#[test]
fn idx2xy1() {
  let frame_size = U16Size::new(10, 10);
  let frame = Frame::new(frame_size, Cursor::default());
  assert_eq!(frame.idx2xy(70), (0, 7));
  assert_eq!(frame.idx2xy(37), (7, 3));
  assert_eq!(frame.idx2xy(1), (1, 0));
  assert_eq!(frame.idx2xy(90), (0, 9));
  assert_eq!(frame.idx2xy(99), (9, 9));
}

#[test]
fn idx2pos1() {
  let frame_size = U16Size::new(10, 10);
  let frame = Frame::new(frame_size, Cursor::default());
  assert_eq!(frame.idx2pos(70), point!(x:0, y:7));
  assert_eq!(frame.idx2pos(37), point!(x:7, y:3));
  assert_eq!(frame.idx2pos(1), point!(x:1, y:0));
  assert_eq!(frame.idx2pos(90), point!(x:0, y:9));
  assert_eq!(frame.idx2pos(99), point!(x:9, y:9));
}

#[test]
fn set_cell1() {
  // test_log_init();
  let frame_size = U16Size::new(10, 10);
  let mut frame = Frame::new(frame_size, Cursor::default());

  let inputs: Vec<(U16Pos, char)> = vec![
    (point!(x: 0, y: 0), 'A'),
    (point!(x: 7, y: 8), 'B'),
    (point!(x: 1, y: 3), 'C'),
    (point!(x: 9, y: 2), 'D'),
    (point!(x: 9, y: 9), 'E'),
    (point!(x: 2, y: 9), 'F'),
    (point!(x: 9, y: 7), 'G'),
  ];

  for (i, input) in inputs.iter().enumerate() {
    let mut c = Cell::default();
    c.set_symbol(input.1.to_compact_string());
    let actual = frame.set_cell(input.0, c);
    info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
    assert_eq!(actual.symbol(), CompactString::new(""));
    assert_eq!(actual.fg(), Color::Reset);
    assert_eq!(actual.bg(), Color::Reset);
    assert_eq!(actual.attrs(), Attributes::default());
  }
  for (i, input) in inputs.iter().enumerate() {
    let actual = frame.get_cell(input.0);
    info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
    assert_eq!(actual.symbol(), input.1.to_compact_string());
    assert_eq!(actual.fg(), Color::Reset);
    assert_eq!(actual.bg(), Color::Reset);
    assert_eq!(actual.attrs(), Attributes::default());
  }
}

#[test]
fn set_empty_cell1() {
  // test_log_init();
  let frame_size = U16Size::new(10, 10);
  let mut frame = Frame::new(frame_size, Cursor::default());

  let inputs: Vec<(U16Pos, char)> = vec![
    (point!(x: 0, y: 0), 'A'),
    (point!(x: 7, y: 8), 'B'),
    (point!(x: 1, y: 3), 'C'),
    (point!(x: 9, y: 2), 'D'),
    (point!(x: 9, y: 9), 'E'),
    (point!(x: 2, y: 9), 'F'),
    (point!(x: 9, y: 7), 'G'),
  ];

  for (i, input) in inputs.iter().enumerate() {
    let mut c = Cell::default();
    c.set_symbol(input.1.to_compact_string());
    let actual = frame.set_cell(input.0, c);
    info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
    assert_eq!(actual.symbol(), CompactString::new(""));
    assert_eq!(actual.fg(), Color::Reset);
    assert_eq!(actual.bg(), Color::Reset);
    assert_eq!(actual.attrs(), Attributes::default());
  }
  for (i, input) in inputs.iter().enumerate() {
    let actual = frame.get_cell(input.0);
    info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
    assert_eq!(actual.symbol(), input.1.to_compact_string());
    assert_eq!(actual.fg(), Color::Reset);
    assert_eq!(actual.bg(), Color::Reset);
    assert_eq!(actual.attrs(), Attributes::default());
  }
  for (i, input) in inputs.iter().enumerate() {
    let mut c = Cell::default();
    c.set_symbol(input.1.to_compact_string());
    let actual = frame.set_empty_cell(input.0);
    info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
    assert_eq!(actual.symbol(), input.1.to_compact_string());
  }
  for (i, input) in inputs.iter().enumerate() {
    let actual = frame.get_cell(input.0);
    info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
    assert_eq!(actual.symbol(), CompactString::new(""));
  }
}

#[test]
fn cells_at1() {
  // test_log_init();
  let frame_size = U16Size::new(10, 10);
  let mut frame = Frame::new(frame_size, Cursor::default());

  let inputs: Vec<(U16Pos, char)> = vec![
    (point!(x: 0, y: 0), 'A'),
    (point!(x: 7, y: 1), 'B'),
    (point!(x: 1, y: 2), 'C'),
    (point!(x: 6, y: 3), 'D'),
    (point!(x: 5, y: 4), 'E'),
    (point!(x: 4, y: 5), 'F'),
    (point!(x: 2, y: 6), 'G'),
    (point!(x: 0, y: 7), 'H'),
    (point!(x: 9, y: 8), 'I'),
    (point!(x: 3, y: 9), 'J'),
  ];
  let expects = [
    "A         ",
    "       B  ",
    " C        ",
    "      D   ",
    "     E    ",
    "    F     ",
    "  G       ",
    "H         ",
    "         I",
    "   J      ",
  ];

  for (i, input) in inputs.iter().enumerate() {
    let mut c = Cell::default();
    c.set_symbol(input.1.to_compact_string());
    let actual = frame.set_cell(input.0, c);
    info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
    assert_eq!(actual.symbol(), CompactString::new(""));
  }
  info!("1-raw_symbols:{:?}", frame.raw_symbols(),);
  let all_cells = frame.get_cells();
  for i in 0..10 {
    let pos: U16Pos = point!(x:0, y:i);
    let cells = frame.get_cells_at(pos, 10);
    let actual = cells
      .iter()
      .map(|c| {
        if c.symbol().is_empty() {
          " ".to_string()
        } else {
          c.symbol().to_string()
        }
      })
      .collect::<Vec<_>>()
      .join("");
    let expect = expects[i as usize];
    info!(
      "{i:?} pos:{pos:?}, cells:{cells:?}, actual:{actual:?}, expect:{expect:?}"
    );
    assert_eq!(actual, expect);

    let idx = frame.pos2idx(pos);
    let cells = &all_cells[idx..(idx + 10)];
    let actual = cells
      .iter()
      .map(|c| {
        if c.symbol().is_empty() {
          " ".to_string()
        } else {
          c.symbol().to_string()
        }
      })
      .collect::<Vec<_>>()
      .join("");
    assert_eq!(actual, expect);
  }

  let actual = frame
    .raw_symbols()
    .iter()
    .map(|sv| {
      sv.iter()
        .map(|c| {
          if c.is_empty() {
            " ".to_string()
          } else {
            c.to_string()
          }
        })
        .collect::<Vec<_>>()
        .join("")
    })
    .collect::<Vec<_>>();
  info!(
    "2-raw_symbols:{:?}, actual:{:?}",
    frame.raw_symbols(),
    actual
  );
  assert_eq!(expects.len(), actual.len());
  for (i, expect) in expects.iter().enumerate() {
    let a = actual[i].clone();
    assert_eq!(a, expect.to_string());
  }
}

#[test]
fn set_cells_at1() {
  test_log_init();
  let frame_size = U16Size::new(10, 10);
  let mut frame = Frame::new(frame_size, Cursor::default());

  let inputs: Vec<(U16Pos, &str)> = vec![
    (point!(x: 0, y: 0), "ABCD"),
    (point!(x: 7, y: 1), "EFGHIJK"),
    (point!(x: 1, y: 2), "LMN"),
    (point!(x: 6, y: 3), "OP"),
    (point!(x: 5, y: 4), "Q"),
    (point!(x: 4, y: 5), ""),
    (point!(x: 2, y: 6), "RSTUV"),
    (point!(x: 0, y: 7), "'WXYZ"),
    (point!(x: 9, y: 8), "abcdefghijk"),
    (point!(x: 3, y: 9), "opqrstu"),
  ];

  let expects = [
    "ABCD      ",
    "       EFG",
    "HLMN      ",
    "      OP  ",
    "     Q    ",
    "          ",
    "  RSTUV   ",
    "'WXYZ     ",
    "         a",
    "bcdopqrstu",
  ];

  for (i, input) in inputs.iter().enumerate() {
    let actual = frame
      .set_cells_at(input.0, input.1.chars().map(Cell::with_char).collect());
    info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
    assert!(actual.len() == input.1.len());
  }
  let actuals = frame.raw_symbols_with_placeholder();
  assert_eq!(actuals.len(), expects.len());
  for (i, expect) in expects.into_iter().enumerate() {
    let actual = actuals[i].join("");
    info!("{:?} actual:{:?}, expect:{:?}", i, actual, expect);
    assert!(actual.len() == expect.len());
    assert_eq!(actual, expect);
  }
}
