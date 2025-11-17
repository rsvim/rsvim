use super::iframe::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use crate::ui::canvas::frame::cell::Cell;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;

#[test]
fn new1() {
  let sz = size!(2, 1);
  let f = Iframe::new(sz);
  assert_eq!(f.size().width, 2);
  assert_eq!(f.size().height, 1);
  assert_eq!(
    f.get_cells().len(),
    f.size().height as usize * f.size().width as usize
  );
  for c in f.get_cells().iter() {
    assert_eq!(c.symbol(), Cell::default().symbol());
  }
}

#[test]
fn pos2range1() {
  let frame_size = size!(10, 10);
  let frame = Iframe::new(frame_size);
  assert_eq!(frame.pos2range(point!(0, 0), 7), 0..7);
  assert_eq!(frame.pos2range(point!(7, 2), 23), 27..50);
  assert_eq!(frame.pos2range(point!(8, 9), 1), 98..99);
  assert_eq!(frame.pos2range(point!(9, 9), 1), 99..100);
}

#[test]
fn idx2range1() {
  let frame_size = size!(10, 10);
  let frame = Iframe::new(frame_size);
  assert_eq!(frame.idx2range(0, 7), 0..7);
  assert_eq!(frame.idx2range(27, 23), 27..50);
  assert_eq!(frame.idx2range(98, 1), 98..99);
  assert_eq!(frame.idx2range(99, 1), 99..100);
}

#[test]
fn xy2idx1() {
  let frame_size = size!(10, 10);
  let frame = Iframe::new(frame_size);
  assert_eq!(frame.xy2idx(0, 7), 70);
  assert_eq!(frame.xy2idx(7, 3), 37);
  assert_eq!(frame.xy2idx(1, 0), 1);
  assert_eq!(frame.xy2idx(0, 9), 90);
  assert_eq!(frame.xy2idx(9, 9), 99);
}

#[test]
fn pos2idx1() {
  let frame_size = size!(10, 10);
  let frame = Iframe::new(frame_size);
  assert_eq!(frame.pos2idx(point!(0, 7)), 70);
  assert_eq!(frame.pos2idx(point!(7, 3)), 37);
  assert_eq!(frame.pos2idx(point!(1, 0)), 1);
  assert_eq!(frame.pos2idx(point!(0, 9)), 90);
  assert_eq!(frame.pos2idx(point!(9, 9)), 99);
}

#[test]
fn idx2xy1() {
  let frame_size = size!(10, 10);
  let frame = Iframe::new(frame_size);
  assert_eq!(frame.idx2xy(70), (0, 7));
  assert_eq!(frame.idx2xy(37), (7, 3));
  assert_eq!(frame.idx2xy(1), (1, 0));
  assert_eq!(frame.idx2xy(90), (0, 9));
  assert_eq!(frame.idx2xy(99), (9, 9));
}

#[test]
fn idx2pos1() {
  let frame_size = size!(10, 10);
  let frame = Iframe::new(frame_size);
  assert_eq!(frame.idx2pos(70), point!(0, 7));
  assert_eq!(frame.idx2pos(37), point!(7, 3));
  assert_eq!(frame.idx2pos(1), point!(1, 0));
  assert_eq!(frame.idx2pos(90), point!(0, 9));
  assert_eq!(frame.idx2pos(99), point!(9, 9));
}

#[test]
fn set_cell1() {
  // test_log_init();
  let frame_size = size!(10, 10);
  let mut frame = Iframe::new(frame_size);

  let inputs: Vec<(U16Pos, char)> = vec![
    (point!(0, 0), 'A'),
    (point!(7, 8), 'B'),
    (point!(1, 3), 'C'),
    (point!(9, 2), 'D'),
    (point!(9, 9), 'E'),
    (point!(2, 9), 'F'),
    (point!(9, 7), 'G'),
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
  let frame_size = size!(10, 10);
  let mut frame = Iframe::new(frame_size);

  let inputs: Vec<(U16Pos, char)> = vec![
    (point!(0, 0), 'A'),
    (point!(7, 8), 'B'),
    (point!(1, 3), 'C'),
    (point!(9, 2), 'D'),
    (point!(9, 9), 'E'),
    (point!(2, 9), 'F'),
    (point!(9, 7), 'G'),
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
  let frame_size = size!(10, 10);
  let mut frame = Iframe::new(frame_size);

  let inputs: Vec<(U16Pos, char)> = vec![
    (point!(0, 0), 'A'),
    (point!(7, 1), 'B'),
    (point!(1, 2), 'C'),
    (point!(6, 3), 'D'),
    (point!(5, 4), 'E'),
    (point!(4, 5), 'F'),
    (point!(2, 6), 'G'),
    (point!(0, 7), 'H'),
    (point!(9, 8), 'I'),
    (point!(3, 9), 'J'),
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
    let pos: U16Pos = point!(0, i);
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
  // test_log_init();
  let frame_size = size!(10, 10);
  let mut frame = Iframe::new(frame_size);

  let inputs: Vec<(U16Pos, &str)> = vec![
    (point!(0, 0), "ABCD"),
    (point!(7, 1), "EFGHIJK"),
    (point!(1, 2), "LMN"),
    (point!(6, 3), "OP"),
    (point!(5, 4), "Q"),
    (point!(4, 5), ""),
    (point!(2, 6), "RSTUV"),
    (point!(0, 7), "'WXYZ"),
    (point!(9, 8), "abcdefghijk"),
    (point!(3, 9), "opqrstu"),
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

#[test]
fn clone1() {
  test_log_init();

  let size1 = size!(5, 5);
  let mut frame1 = Iframe::new(size1);
  for i in 0..25 {
    let pos = frame1.idx2pos(i);
    frame1.set_cell(pos, Cell::with_char('a'));
  }
  assert_eq!(frame1.get_cells().len(), 25);
  for c in frame1.get_cells() {
    assert_eq!(c.symbol(), 'a'.to_compact_string());
  }
  info!("frame1:{frame1:?}");

  let size2 = size!(3, 3);
  let mut frame2 = Iframe::new(size2);
  for i in 0..9 {
    let pos = frame2.idx2pos(i);
    frame2.set_cell(pos, Cell::with_char('b'));
  }
  frame2.clone_from(&frame1);
  assert_eq!(frame2.get_cells().len(), 25);
  for c in frame2.get_cells() {
    assert_eq!(c.symbol(), 'a'.to_compact_string());
  }
  info!("frame2:{frame2:?}");

  let size3 = size!(7, 7);
  let mut frame3 = Iframe::new(size3);
  for i in 0..49 {
    let pos = frame3.idx2pos(i);
    frame3.set_cell(pos, Cell::with_char('c'));
  }
  frame3.clone_from(&frame1);
  assert_eq!(frame3.get_cells().len(), 25);
  for c in frame3.get_cells() {
    assert_eq!(c.symbol(), 'a'.to_compact_string());
  }
  info!("frame3:{frame3:?}");
}
