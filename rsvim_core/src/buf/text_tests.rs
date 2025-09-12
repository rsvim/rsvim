use super::text::*;
use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::opt::FileFormatOption;
use crate::coord::U16Size;
use crate::tests::log::init as test_log_init;
use ropey::Rope;

#[test]
fn last_char1_unix() {
  test_log_init();

  let terminal_size = U16Size::new(10, 10);
  let opt = BufferOptionsBuilder::default()
    .file_format(FileFormatOption::Unix)
    .build()
    .unwrap();

  {
    let rope = Rope::from_str("hello\nworld\r");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(text.is_eol(0, 5));
    assert!(!text.is_eol(1, 4));
    assert!(text.is_eol(1, 5));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\n');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');

    let actual3 = text.last_char_on_line(1);
    assert!(actual3.is_some());
    assert_eq!(actual3.unwrap(), 5);
    assert_eq!(text.rope().line(1).char(5), '\r');

    let actual4 = text.last_char_on_line_no_eol(1);
    assert!(actual4.is_some());
    assert_eq!(actual4.unwrap(), 4);
    assert_eq!(text.rope().line(1).char(4), 'd');
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
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
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
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
  }
}

#[test]
fn last_char1_win() {
  test_log_init();

  let terminal_size = U16Size::new(10, 10);
  let opt = BufferOptionsBuilder::default()
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
    let rope = Rope::from_str("hello\r\nworld\r\n");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(text.is_eol(0, 5));
    assert!(text.is_eol(0, 6));
    assert!(!text.is_eol(1, 4));
    assert!(text.is_eol(1, 5));
    assert!(text.is_eol(1, 6));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 6);
    assert_eq!(text.rope().line(0).char(6), '\n');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');

    let actual3 = text.last_char_on_line(1);
    assert!(actual3.is_some());
    assert_eq!(actual3.unwrap(), 6);
    assert_eq!(text.rope().line(1).char(6), '\n');

    let actual4 = text.last_char_on_line_no_eol(1);
    assert!(actual4.is_some());
    assert_eq!(actual4.unwrap(), 4);
    assert_eq!(text.rope().line(1).char(4), 'd');
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
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
  }
}

#[test]
fn last_char1_mac() {
  test_log_init();

  let terminal_size = U16Size::new(10, 10);
  let opt = BufferOptionsBuilder::default()
    .file_format(FileFormatOption::Mac)
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
    let rope = Rope::from_str("hello\r\nworld\r");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(text.is_eol(0, 5));
    assert!(text.is_eol(0, 6));
    assert!(!text.is_eol(1, 4));
    assert!(text.is_eol(1, 5));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 6);
    assert_eq!(text.rope().line(0).char(6), '\n');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');

    let actual3 = text.last_char_on_line(1);
    assert!(actual3.is_some());
    assert_eq!(actual3.unwrap(), 5);
    assert_eq!(text.rope().line(1).char(5), '\r');

    let actual4 = text.last_char_on_line_no_eol(1);
    assert!(actual4.is_some());
    assert_eq!(actual4.unwrap(), 4);
    assert_eq!(text.rope().line(1).char(4), 'd');
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
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');
  }

  {
    let rope = Rope::from_str("hello\rworld\r");
    let text = Text::new(opt, terminal_size, rope);

    assert!(!text.is_eol(0, 4));
    assert!(text.is_eol(0, 5));
    assert!(!text.is_eol(1, 4));
    assert!(text.is_eol(1, 5));

    let actual1 = text.last_char_on_line(0);
    assert!(actual1.is_some());
    assert_eq!(actual1.unwrap(), 5);
    assert_eq!(text.rope().line(0).char(5), '\r');

    let actual2 = text.last_char_on_line_no_eol(0);
    assert!(actual2.is_some());
    assert_eq!(actual2.unwrap(), 4);
    assert_eq!(text.rope().line(0).char(4), 'o');

    let actual3 = text.last_char_on_line(1);
    assert!(actual3.is_some());
    assert_eq!(actual3.unwrap(), 5);
    assert_eq!(text.rope().line(1).char(5), '\r');

    let actual4 = text.last_char_on_line_no_eol(1);
    assert!(actual4.is_some());
    assert_eq!(actual4.unwrap(), 4);
    assert_eq!(text.rope().line(1).char(4), 'd');
  }
}
