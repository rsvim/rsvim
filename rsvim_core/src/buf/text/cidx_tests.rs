use super::cidx::*;

use crate::buf::opt::{BufferLocalOptions, BufferLocalOptionsBuilder};
use crate::buf::text::Text;
use crate::prelude::*;
use crate::test::log::init as test_log_init;

use ropey::{Rope, RopeBuilder, RopeSlice};

fn make_default_opts() -> BufferLocalOptions {
  BufferLocalOptionsBuilder::default().build().unwrap()
}

fn make_rope_from_lines(lines: Vec<&str>) -> Rope {
  let mut rb: RopeBuilder = RopeBuilder::new();
  for line in lines.iter() {
    rb.append(line);
  }
  rb.finish()
}

fn make_text_from_rope(
  opts: BufferLocalOptions,
  terminal_size: U16Size,
  rp: Rope,
) -> Text {
  Text::new(opts, terminal_size, rp)
}

#[allow(clippy::unused_enumerate_index)]
fn print_text_line_details(text: Text, line_idx: usize, msg: &str) {
  let line = text.rope().get_line(line_idx).unwrap();

  if !msg.is_empty() {
    info!("line: {}", msg);
  } else {
    info!("line");
  }

  let mut payload = String::new();
  for c in line.chars() {
    let cs = text.char_symbol(c);
    payload.push_str(cs.as_ref());
  }
  info!("-{}-", payload);

  {
    let mut builder = String::new();
    let mut n = 0_usize;
    let mut w = 0_usize;
    let mut zero_width_chars: Vec<String> = vec![];
    let mut big_width_chars: Vec<String> = vec![];
    for (i, c) in line.chars().enumerate() {
      let cw = text.char_width(c);
      w += cw;
      n += 1;
      if cw == 0 {
        zero_width_chars.push(format!("{i}"));
      }
      if cw > 1 {
        big_width_chars.push(format!("{i}"));
      }
      if i % 5 == 0 {
        builder.push_str(&format!("{i}"));
      }
      if builder.len() < w {
        let diff = w - builder.len();
        builder.push_str(&" ".repeat(diff));
      }
    }
    info!(
      "-{}- Char Index, total:{} (width = 0 chars: count:{} indexes:{}, width > 1 chars: count:{} indexes:{})",
      builder,
      n,
      zero_width_chars.len(),
      zero_width_chars.join(","),
      big_width_chars.len(),
      big_width_chars.join(",")
    );
  }

  {
    let mut builder1 = String::new();
    let mut builder2 = String::new();
    let mut builder3 = String::new();
    let mut show2 = false;
    let mut show3 = false;
    let mut w = 0_usize;
    for (_i, c) in line.chars().enumerate() {
      let cw = text.char_width(c);
      w += cw;
      if w == 1 || w % 5 == 0 {
        if builder1.is_empty() || builder1.ends_with(" ") {
          builder1.push_str(&format!("{w}"));
        } else if cw > 0 {
          builder2.push_str(&format!("{w}"));
          show2 = true;
        } else {
          builder3.push_str(&format!("{w}"));
          show3 = true;
        }
      }

      if builder1.len() < w {
        let diff = w - builder1.len();
        builder1.push_str(&" ".repeat(diff));
      }
      if builder2.len() < w {
        let diff = w - builder2.len();
        builder2.push_str(&" ".repeat(diff));
      }
      if builder3.len() < w {
        let diff = w - builder3.len();
        builder3.push_str(&" ".repeat(diff));
      }
    }
    info!("-{}- Display Width, total width:{}", builder1, w);
    if show2 {
      info!(
        "-{}- Display Width (extra, conflicted with the above one)",
        builder2
      );
    }
    if show3 {
      info!("-{}- Display Width for width = 0 chars", builder3);
    }
  }

  {
    let mut builder = String::new();
    let mut w = 0_usize;
    let mut show = false;
    for (_i, c) in line.chars().enumerate() {
      let cw = text.char_width(c);
      w += cw;
      if cw > 1 && (builder.is_empty() || builder.ends_with(" ")) {
        builder.push_str(&" ".repeat(cw - 1));
        builder.push_str(&format!("{w}"));
        show = true;
      }

      if builder.len() < w {
        let diff = w - builder.len();
        builder.push_str(&" ".repeat(diff));
      }
    }
    if show {
      info!("-{}- Display Width for width > 1 chars", builder);
    }
  }
}

fn assert_width_at(
  options: &BufferLocalOptions,
  buf_line: &RopeSlice,
  actual: &mut ColumnIndex,
  expect: &[usize],
) {
  for (i, e) in expect.iter().enumerate() {
    let a = actual.width_until(options, buf_line, i);
    info!("width_at:{i} actual:{a:?}, expect:{e:?}");
    assert_eq!(a, *e);
  }
}

fn assert_width_at_rev(
  options: &BufferLocalOptions,
  buf_line: &RopeSlice,
  actual: &mut ColumnIndex,
  expect: &[(usize, usize)],
) {
  for (e, i) in expect.iter() {
    let a = actual.width_until(options, buf_line, *i);
    info!("width_at_rev:{i}, actual:{a:?}, expect:{e:?}");
    assert_eq!(a, *e);
  }
}

fn assert_width_before(
  options: &BufferLocalOptions,
  buf_line: &RopeSlice,
  actual: &mut ColumnIndex,
  expect: &[usize],
) {
  for (i, e) in expect.iter().enumerate() {
    let a = actual.width_before(options, buf_line, i);
    info!("width_before:{i} actual:{a:?}, expect:{e:?}");
    assert_eq!(a, *e);
  }
}

fn assert_width_before_rev(
  options: &BufferLocalOptions,
  buf_line: &RopeSlice,
  actual: &mut ColumnIndex,
  expect: &[(usize, usize)],
) {
  for (e, i) in expect.iter() {
    let a = actual.width_before(options, buf_line, *i);
    info!("width_before:{i}, actual:{a:?}, expect:{e:?}");
    assert_eq!(a, *e);
  }
}

#[test]
fn width1() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec!["Hello,\tRSVIM!\n"]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "width1");

  let mut actual = ColumnIndex::with_capacity(10);

  let expect: Vec<usize> =
    [(1..=6).collect(), (14..=20).collect(), vec![20, 20, 20, 20]].concat();
  assert_width_at(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<usize> =
    [(0..=6).collect(), (14..=20).collect(), vec![20, 20, 20]].concat();
  assert_width_before(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect);
}

#[test]
fn width2() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec![
    "This is a quite simple and small test lines.\n",
  ]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "width2");

  let mut actual = ColumnIndex::with_capacity(10);

  assert_eq!(actual.width_before(&options, &rope.line(0), 5), 5);
  assert_eq!(actual.width_until(&options, &rope.line(0), 43), 44);

  let expect: Vec<usize> = [(1..=44).collect(), vec![44, 44, 44, 44]].concat();

  let expect1: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect1);

  assert_width_at(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<usize> = [(0..=44).collect(), vec![44, 44, 44]].concat();

  let expect1: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect1);

  assert_width_before(&options, &rope.line(0), &mut actual, &expect);
}

#[test]
fn width3() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec![
    "But still\tit\\包含了好几种东西we want to test:\n",
  ]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "width3");

  let mut actual = ColumnIndex::with_capacity(10);

  let expect: Vec<usize> = [
    (1..=9).collect(),
    (17..=20).collect(),
    (22..=29)
      .scan(22, |state, i| {
        let diff: usize = i - *state;
        Some(*state + 2 * diff)
      })
      .collect(),
    (37..=52).collect(),
    vec![52, 52, 52, 52],
  ]
  .concat();
  assert_width_at(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<usize> = [
    (0..=9).collect(),
    (17..=20).collect(),
    (22..=29)
      .scan(22, |state, i| {
        let diff: usize = i - *state;
        Some(*state + 2 * diff)
      })
      .collect(),
    (37..=52).collect(),
    vec![52, 52, 52],
  ]
  .concat();
  assert_width_before(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect);
}

#[test]
fn width4() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec!["  1. When the\r"]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "width4");

  let mut actual = ColumnIndex::with_capacity(10);

  assert_eq!(actual.width_before(&options, &rope.line(0), 11), 11);
  assert_eq!(actual.width_until(&options, &rope.line(0), 10), 11);

  // For CR, on Windows it is 0 width, otherwise it is 2 width (^M).
  let cr_width = if cfg!(target_os = "windows") { 13 } else { 15 };

  let expect: Vec<usize> = [
    (1..=13).collect(),
    vec![cr_width, cr_width, cr_width, cr_width],
  ]
  .concat();
  assert_width_at(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<usize> =
    [(0..=13).collect(), vec![cr_width, cr_width, cr_width]].concat();
  assert_width_before(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect);
}

#[test]
fn width5() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec![
    "一行文本小到可以放入一个窗口中，那么line-wrap和word-wrap选项就不会影响排版。\n",
  ]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "width5");

  let mut actual = ColumnIndex::with_capacity(10);

  let expect: Vec<usize> = [
    (1..=18).map(|i| i * 2).collect(),
    (37..=45).collect(),
    vec![47],
    (48..=56).collect(),
    (58..=67)
      .scan(58, |state, i| {
        let diff: usize = i - *state;
        Some(*state + 2 * diff)
      })
      .collect(),
    vec![76, 76, 76, 76],
  ]
  .concat();
  assert_width_at(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<usize> = [
    (0..=18).map(|i| i * 2).collect(),
    (37..=45).collect(),
    vec![47],
    (48..=56).collect(),
    (58..=67)
      .scan(58, |state, i| {
        let diff: usize = i - *state;
        Some(*state + 2 * diff)
      })
      .collect(),
    vec![76, 76, 76, 76],
  ]
  .concat();
  assert_width_before(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect);
}

#[test]
fn width6() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec![
    "\t\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
  ]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "width6");

  let mut actual = ColumnIndex::with_capacity(10);

  assert_eq!(actual.width_before(&options, &rope.line(0), 1), 8);
  assert_eq!(actual.width_before(&options, &rope.line(0), 2), 16);
  assert_eq!(actual.width_until(&options, &rope.line(0), 2), 17);

  let expect: Vec<usize> =
    [vec![8, 16], (17..=129).collect(), vec![129, 129, 129, 129]].concat();

  let expect1: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_at_rev(&options, &rope.line(0), &mut actual, &expect1);

  assert_width_at(&options, &rope.line(0), &mut actual, &expect);

  let expect: Vec<usize> =
    [vec![0, 8, 16], (17..=129).collect(), vec![129, 129, 129]].concat();

  let expect1: Vec<(usize, usize)> = expect
    .iter()
    .enumerate()
    .map(|(i, e)| (*e, i))
    .rev()
    .collect();
  assert_width_before_rev(&options, &rope.line(0), &mut actual, &expect1);

  assert_width_before(&options, &rope.line(0), &mut actual, &expect);
}

#[test]
fn width7() {
  test_log_init();

  let options = make_default_opts();
  let rope = Rope::new();

  let mut actual = ColumnIndex::with_capacity(10);

  assert_eq!(actual.width_before(&options, &rope.line(0), 0), 0);
  assert_eq!(actual.width_before(&options, &rope.line(0), 1), 0);
  assert_eq!(actual.width_before(&options, &rope.line(0), 2), 0);
  assert_eq!(actual.width_before(&options, &rope.line(0), 10), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 0), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 1), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 2), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 10), 0);

  let rope = make_rope_from_lines(vec![]);
  let mut actual = ColumnIndex::with_capacity(10);

  assert_eq!(actual.width_before(&options, &rope.line(0), 0), 0);
  assert_eq!(actual.width_before(&options, &rope.line(0), 1), 0);
  assert_eq!(actual.width_before(&options, &rope.line(0), 2), 0);
  assert_eq!(actual.width_before(&options, &rope.line(0), 10), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 0), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 1), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 2), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 10), 0);

  let rope = make_rope_from_lines(vec![""]);
  let mut actual = ColumnIndex::with_capacity(10);

  assert_eq!(actual.width_before(&options, &rope.line(0), 0), 0);
  assert_eq!(actual.width_before(&options, &rope.line(0), 1), 0);
  assert_eq!(actual.width_before(&options, &rope.line(0), 2), 0);
  assert_eq!(actual.width_before(&options, &rope.line(0), 10), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 0), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 1), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 2), 0);
  assert_eq!(actual.width_until(&options, &rope.line(0), 10), 0);
}

fn assert_char_before(
  options: &BufferLocalOptions,
  buf_line: &RopeSlice,
  widx: &mut ColumnIndex,
  expect_before: &[(usize, Option<usize>)],
) {
  for (w, c) in expect_before.iter() {
    let actual = widx.char_before(options, buf_line, *w);
    info!("char_before expect char:{c:?} width:{w:?}, actual char:{actual:?}");
    assert_eq!(actual, *c);
    if c.is_some() {
      let actual = widx.width_until(options, buf_line, c.unwrap());
      info!(
        "width_at-1 char:{c:?} expect width:{w:?}, actual width:{actual:?}"
      );
      assert!(actual < *w);
    }
  }
}

fn assert_char_at(
  options: &BufferLocalOptions,
  buf_line: &RopeSlice,
  widx: &mut ColumnIndex,
  expect_until: &[(usize, Option<usize>)],
) {
  for (w, c) in expect_until.iter() {
    let actual = widx.char_at(options, buf_line, *w);
    info!("char_at expect char:{c:?} width:{w:?}, actual char:{actual:?}");
    assert_eq!(actual, *c);
    if c.is_some() {
      let actual = widx.width_until(options, buf_line, c.unwrap());
      info!(
        "width_at-2 char:{c:?} expect width:{w:?}, actual width:{actual:?}"
      );
      assert!(actual >= *w);
    } else {
      info!("width_at-2 char:{c:?} expect width:{w:?}");
    }
  }
}

fn assert_char_after(
  options: &BufferLocalOptions,
  buf_line: &RopeSlice,
  widx: &mut ColumnIndex,
  expect_after: &[(usize, Option<usize>)],
) {
  for (w, c) in expect_after.iter() {
    let actual = widx.char_after(options, buf_line, *w);
    info!("char_after expect char:{c:?} width:{w:?}, actual char:{actual:?}");
    assert_eq!(actual, *c);
    if c.is_some() {
      let actual = widx.width_until(options, buf_line, c.unwrap());
      info!(
        "width_at-3 char:{c:?} expect width:{w:?}, actual width:{actual:?}"
      );
      assert!(actual >= *w);
    }
  }
}

fn assert_last_char_until(
  options: &BufferLocalOptions,
  buf_line: &RopeSlice,
  widx: &mut ColumnIndex,
  expect_until: &[(usize, Option<usize>)],
) {
  for (w, c) in expect_until.iter() {
    let actual = widx.last_char_until(options, buf_line, *w);
    info!(
      "last_char_until expect char:{c:?} width:{w:?}, actual char:{actual:?}"
    );
    assert_eq!(actual, *c);
  }
}

#[test]
fn char1() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec!["These are\t很简单的test\tlines.\n"]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "char1");

  let mut widx = ColumnIndex::new();

  let expect_before: Vec<(usize, Option<usize>)> = vec![
    (0, None),
    (1, None),
    (5, Some(3)),
    (9, Some(7)),
    (10, Some(8)),
    (11, Some(8)),
    (16, Some(8)),
    (17, Some(8)),
    (18, Some(9)),
    (19, Some(9)),
    (20, Some(10)),
    (21, Some(10)),
    (22, Some(11)),
    (23, Some(11)),
    (24, Some(12)),
    (25, Some(12)),
    (26, Some(13)),
    (27, Some(14)),
    (28, Some(15)),
    (29, Some(16)),
    (30, Some(17)),
    (31, Some(17)),
    (32, Some(17)),
    (36, Some(17)),
    (37, Some(17)),
    (38, Some(18)),
    (39, Some(19)),
    (43, Some(23)),
    (44, None),
    (45, None),
  ];
  assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

  let expect_at: Vec<(usize, Option<usize>)> = vec![
    (0, None),
    (1, Some(0)),
    (5, Some(4)),
    (8, Some(7)),
    (9, Some(8)),
    (10, Some(9)),
    (11, Some(9)),
    (12, Some(9)),
    (13, Some(9)),
    (14, Some(9)),
    (15, Some(9)),
    (16, Some(9)),
    (17, Some(9)),
    (18, Some(10)),
    (19, Some(10)),
    (20, Some(11)),
    (21, Some(11)),
    (22, Some(12)),
    (23, Some(12)),
    (24, Some(13)),
    (25, Some(13)),
    (26, Some(14)),
    (27, Some(15)),
    (28, Some(16)),
    (29, Some(17)),
    (30, Some(18)),
    (35, Some(18)),
    (36, Some(18)),
    (37, Some(18)),
    (38, Some(19)),
    (39, Some(20)),
    (40, Some(21)),
    (41, Some(22)),
    (42, Some(23)),
    (43, Some(24)),
    (44, None),
    (45, None),
  ];
  assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

  let expect_after: Vec<(usize, Option<usize>)> = vec![
    (0, Some(0)),
    (1, Some(1)),
    (2, Some(2)),
    (5, Some(5)),
    (6, Some(6)),
    (7, Some(7)),
    (8, Some(8)),
    (9, Some(9)),
    (10, Some(10)),
    (11, Some(10)),
    (15, Some(10)),
    (16, Some(10)),
    (17, Some(10)),
    (18, Some(11)),
    (19, Some(11)),
    (20, Some(12)),
    (21, Some(12)),
    (22, Some(13)),
    (23, Some(13)),
    (24, Some(14)),
    (25, Some(14)),
    (26, Some(15)),
    (27, Some(16)),
    (28, Some(17)),
    (29, Some(18)),
    (30, Some(19)),
    (31, Some(19)),
    (32, Some(19)),
    (36, Some(19)),
    (37, Some(19)),
    (38, Some(20)),
    (39, Some(21)),
    (40, Some(22)),
    (41, Some(23)),
    (42, Some(24)),
    (43, Some(25)),
    (44, None),
    (45, None),
  ];
  assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);

  let expect_until: Vec<(usize, Option<usize>)> = vec![
    (0, None),
    (1, Some(0)),
    (5, Some(4)),
    (8, Some(7)),
    (9, Some(8)),
    (10, Some(9)),
    (11, Some(9)),
    (12, Some(9)),
    (13, Some(9)),
    (14, Some(9)),
    (15, Some(9)),
    (16, Some(9)),
    (17, Some(9)),
    (18, Some(10)),
    (19, Some(10)),
    (20, Some(11)),
    (21, Some(11)),
    (22, Some(12)),
    (23, Some(12)),
    (24, Some(13)),
    (25, Some(13)),
    (26, Some(14)),
    (27, Some(15)),
    (28, Some(16)),
    (29, Some(17)),
    (30, Some(18)),
    (35, Some(18)),
    (36, Some(18)),
    (37, Some(18)),
    (38, Some(19)),
    (39, Some(20)),
    (40, Some(21)),
    (41, Some(22)),
    (42, Some(23)),
    (43, Some(24)),
    (44, Some(24)),
    (45, Some(24)),
    (46, Some(24)),
  ];
  assert_last_char_until(&options, &rope.line(0), &mut widx, &expect_until);
}

#[test]
fn char2() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec!["\t"]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "char2");
  let mut widx = ColumnIndex::new();

  let expect_before: Vec<(usize, Option<usize>)> = vec![
    (0, None),
    (1, None),
    (3, None),
    (6, None),
    (7, None),
    (8, None),
    (9, None),
    (10, None),
  ];
  assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

  let expect_at: Vec<(usize, Option<usize>)> = vec![
    (0, None),
    (1, Some(0)),
    (2, Some(0)),
    (3, Some(0)),
    (4, Some(0)),
    (5, Some(0)),
    (6, Some(0)),
    (7, Some(0)),
    (8, Some(0)),
    (9, None),
    (10, None),
  ];
  assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

  let expect_after: Vec<(usize, Option<usize>)> = vec![
    (0, Some(0)),
    (1, None),
    (2, None),
    (3, None),
    (5, None),
    (6, None),
    (7, None),
    (8, None),
    (9, None),
    (10, None),
  ];
  assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);

  let expect_until: Vec<(usize, Option<usize>)> = vec![
    (0, None),
    (1, Some(0)),
    (2, Some(0)),
    (3, Some(0)),
    (4, Some(0)),
    (5, Some(0)),
    (6, Some(0)),
    (7, Some(0)),
    (8, Some(0)),
    (9, Some(0)),
    (10, Some(0)),
    (11, Some(0)),
    (12, Some(0)),
  ];
  assert_last_char_until(&options, &rope.line(0), &mut widx, &expect_until);
}

#[test]
fn char3() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec!["\n"]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "char3");
  let mut widx = ColumnIndex::with_capacity(10);

  let expect_before: Vec<(usize, Option<usize>)> = vec![
    (0, None),
    (1, None),
    (3, None),
    (6, None),
    (7, None),
    (8, None),
    (9, None),
    (10, None),
  ];
  assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

  let expect_at: Vec<(usize, Option<usize>)> = vec![
    (0, Some(0)),
    (1, None),
    (2, None),
    (3, None),
    (4, None),
    (5, None),
    (6, None),
    (7, None),
    (8, None),
    (9, None),
    (10, None),
  ];
  assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

  let expect_after: Vec<(usize, Option<usize>)> = vec![
    (0, Some(0)),
    (1, None),
    (2, None),
    (3, None),
    (5, None),
    (6, None),
    (7, None),
    (8, None),
    (9, None),
    (10, None),
  ];
  assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);

  let expect_until: Vec<(usize, Option<usize>)> = vec![
    (0, Some(0)),
    (1, Some(0)),
    (2, Some(0)),
    (3, Some(0)),
    (4, Some(0)),
    (5, Some(0)),
    (6, Some(0)),
    (7, Some(0)),
    (8, Some(0)),
    (9, Some(0)),
    (10, Some(0)),
  ];
  assert_last_char_until(&options, &rope.line(0), &mut widx, &expect_until);
}

#[test]
fn char4() {
  test_log_init();

  let options = make_default_opts();

  {
    let rope = Rope::new();

    let mut widx = ColumnIndex::new();

    let expect_before: Vec<(usize, Option<usize>)> =
      (0..50).map(|i| (i, None)).collect();
    assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

    let expect_at: Vec<(usize, Option<usize>)> =
      (0..50).map(|i| (i, None)).collect();
    assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

    let expect_after: Vec<(usize, Option<usize>)> =
      (0..50).map(|i| (i, None)).collect();
    assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
  }

  {
    let rope = make_rope_from_lines(vec![]);
    let mut widx = ColumnIndex::new();

    let expect_before: Vec<(usize, Option<usize>)> =
      (0..50).map(|i| (i, None)).collect();
    assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

    let expect_at: Vec<(usize, Option<usize>)> =
      (0..50).map(|i| (i, None)).collect();
    assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

    let expect_after: Vec<(usize, Option<usize>)> =
      (0..50).map(|i| (i, None)).collect();
    assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
  }

  {
    let rope = make_rope_from_lines(vec![""]);
    let buffer =
      make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
    print_text_line_details(buffer, 0, "char3-3");

    let mut widx = ColumnIndex::new();

    let expect_before: Vec<(usize, Option<usize>)> =
      (0..50).map(|i| (i, None)).collect();
    assert_char_before(&options, &rope.line(0), &mut widx, &expect_before);

    let expect_at: Vec<(usize, Option<usize>)> =
      (0..50).map(|i| (i, None)).collect();
    assert_char_at(&options, &rope.line(0), &mut widx, &expect_at);

    let expect_after: Vec<(usize, Option<usize>)> =
      (0..50).map(|i| (i, None)).collect();
    assert_char_after(&options, &rope.line(0), &mut widx, &expect_after);
  }
}

#[test]
fn truncate1() {
  test_log_init();

  let options = make_default_opts();
  let rope = make_rope_from_lines(vec!["Hello,\tRSVIM!\n"]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "truncate1");

  let mut widx = ColumnIndex::new();

  let expect_at: Vec<usize> =
    [(1..=6).collect(), (14..=20).collect(), vec![20, 20, 20, 20]].concat();

  for (c, w) in expect_at.iter().enumerate() {
    let actual = widx.width_until(&options, &rope.line(0), c);
    info!(
      "truncate1-width_at expect width:{w:?}, char:{c:}, actual width:{actual:?}"
    );
    assert_eq!(actual, *w);
    widx.truncate_since_char(c);
  }

  let expect_before: Vec<usize> =
    [(0..=6).collect(), (14..=20).collect(), vec![20, 20, 20]].concat();

  for (c, w) in expect_before.iter().enumerate() {
    let actual = widx.width_before(&options, &rope.line(0), c);
    info!(
      "truncate1-width_before expect width:{w:?}, char:{c:}, actual width:{actual:?}"
    );
    assert_eq!(actual, *w);
    widx.truncate_since_char(c);
  }
}

#[test]
fn truncate2() {
  test_log_init();

  let options = make_default_opts();
  let rope =
    make_rope_from_lines(vec!["This is a quite\t简单而且很小的test\tlines.\n"]);
  let buffer = make_text_from_rope(options, U16Size::new(10, 10), rope.clone());
  print_text_line_details(buffer, 0, "truncate2");
  let mut widx = ColumnIndex::with_capacity(10);

  let expect_before: Vec<(usize, Option<usize>)> = vec![
    (0, None),
    (1, None),
    (5, Some(3)),
    (10, Some(8)),
    (15, Some(13)),
    (16, Some(14)),
    (17, Some(14)),
    (22, Some(14)),
    (23, Some(14)),
    (24, Some(15)),
    (25, Some(15)),
    (26, Some(16)),
    (27, Some(16)),
    (28, Some(17)),
    (29, Some(17)),
    (30, Some(18)),
    (31, Some(18)),
    (32, Some(19)),
    (33, Some(19)),
    (34, Some(20)),
    (35, Some(20)),
    (36, Some(21)),
    (37, Some(21)),
    (38, Some(22)),
    (39, Some(23)),
  ];

  let expect_at: Vec<(usize, Option<usize>)> = vec![
    (0, None),
    (1, Some(0)),
    (5, Some(4)),
    (10, Some(9)),
    (15, Some(14)),
    (16, Some(15)),
    (17, Some(15)),
    (22, Some(15)),
    (23, Some(15)),
    (24, Some(16)),
    (25, Some(16)),
    (26, Some(17)),
    (27, Some(17)),
    (28, Some(18)),
    (29, Some(18)),
    (30, Some(19)),
    (31, Some(19)),
    (32, Some(20)),
    (33, Some(20)),
    (34, Some(21)),
    (35, Some(21)),
  ];

  for (w, c) in expect_before.iter() {
    let actual = widx.char_before(&options, &rope.line(0), *w);
    info!(
      "truncate2-char_before expect char:{c:?} width:{w:?}, actual char:{actual:?}"
    );
    assert_eq!(actual, *c);
    widx.truncate_since_width(*w);
  }
  for (w, c) in expect_at.iter() {
    let actual = widx.char_at(&options, &rope.line(0), *w);
    info!(
      "truncate2-char_at expect char:{c:?} width:{w:?}, actual char:{actual:?}"
    );
    assert_eq!(actual, *c);
    widx.truncate_since_width(*w);
  }
}
