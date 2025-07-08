use super::text::*;

use crate::buf::opt::{BufferLocalOptionsBuilder, FileFormatOption};
use crate::coord::U16Size;
use crate::defaults::ascii::AsciiControlCodeFormatter;
use crate::test::log::init as test_log_init;

use ascii::AsciiChar;
use ropey::{Rope, RopeBuilder};
use tracing::info;
use unicode_width::UnicodeWidthChar;

#[test]
fn last_char1_unix() {
  test_log_init();

  let terminal_size = U16Size::new(10, 10);
  let opt = BufferLocalOptionsBuilder::default()
    .file_format(FileFormatOption::Unix)
    .build()
    .unwrap();

  {
    let rope = Rope::from_str("hello\n");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(text.is_eol(0, 5));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\n');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
  }

  {
    let rope = Rope::from_str("hello");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
  }

  {
    let rope = Rope::from_str("hello\r\n");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(!text.is_eol(0, 5));
    assert!(text.is_eol(0, 6));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 6);
    assert_eq!(text.rope().line(0).char(6), '\n');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');
  }

  {
    let rope = Rope::from_str("hello\r");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(!text.is_eol(0, 5));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');
  }
}

#[test]
fn last_char1_win() {
  test_log_init();

  let terminal_size = U16Size::new(10, 10);
  let opt = BufferLocalOptionsBuilder::default()
    .file_format(FileFormatOption::Dos)
    .build()
    .unwrap();

  {
    let rope = Rope::from_str("hello\n");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(text.is_eol(0, 5));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\n');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
  }

  {
    let rope = Rope::from_str("hello");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
  }

  {
    let rope = Rope::from_str("hello\r\n");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(text.is_eol(0, 5));
    assert!(text.is_eol(0, 6));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 6);
    assert_eq!(text.rope().line(0).char(6), '\n');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');
  }

  {
    let rope = Rope::from_str("hello\r");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(!text.is_eol(0, 5));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');
  }
}

#[test]
fn last_char1_mac() {
  test_log_init();

  let terminal_size = U16Size::new(10, 10);
  let opt = BufferLocalOptionsBuilder::default()
    .file_format(FileFormatOption::Mac)
    .build()
    .unwrap();

  {
    let rope = Rope::from_str("hello\n");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(!text.is_eol(0, 5));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\n');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
  }

  {
    let rope = Rope::from_str("hello");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
  }

  {
    let rope = Rope::from_str("hello\r\n");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(text.is_eol(0, 5));
    assert!(!text.is_eol(1, 0));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 6);
    assert_eq!(text.rope().line(0).char(6), '\n');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');
  }

  {
    let rope = Rope::from_str("hello\r");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(text.is_eol(0, 5));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');
  }
}
