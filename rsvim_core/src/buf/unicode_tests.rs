use super::unicode::*;

use crate::buf::opt::{BufferLocalOptionsBuilder, FileFormatOption};
use crate::defaults::ascii::AsciiControlCodeFormatter;
use crate::test::log::init as test_log_init;

use ascii::AsciiChar;
use tracing::info;
use unicode_width::UnicodeWidthChar;

#[test]
fn char_width1() {
  test_log_init();

  for i in 0_u8..32_u8 {
    let c = i as char;
    let asciic = AsciiChar::from_ascii(c).unwrap();
    let opt = BufferLocalOptionsBuilder::default().build().unwrap();
    let asciifmt = AsciiControlCodeFormatter::from(asciic);
    let formatted = format!("{asciifmt}");
    let formatted_len = formatted.len();
    info!("i:{i},c:{c:?},ascii char:{asciic:?},ascii formatted:{formatted:?}({formatted_len})");
    if asciic == AsciiChar::Tab {
      assert_eq!(char_width(&opt, c), opt.tab_stop() as usize);
    } else if asciic == AsciiChar::LineFeed {
      assert_eq!(char_width(&opt, c), 0);
    } else if asciic == AsciiChar::CarriageReturn {
      if opt.file_format() == FileFormatOption::Unix {
        assert_eq!(char_width(&opt, c), formatted_len);
      } else {
        assert_eq!(char_width(&opt, c), 0);
      }
    } else {
      assert_eq!(char_width(&opt, c), formatted_len);
    }
  }

  {
    let c = 'A';
    let opt = BufferLocalOptionsBuilder::default().build().unwrap();
    let formatted = format!("{c}");
    let formatted_width = UnicodeWidthChar::width_cjk(c).unwrap();
    info!("c:{c:?},formatted:{formatted:?}({formatted_width})");
    assert_eq!(char_width(&opt, c), formatted_width);
  }

  {
    let c = '好';
    let opt = BufferLocalOptionsBuilder::default().build().unwrap();
    let formatted = format!("{c}");
    let formatted_width = UnicodeWidthChar::width_cjk(c).unwrap();
    info!("c:{c:?},formatted:{formatted:?}({formatted_width})");
    assert_eq!(char_width(&opt, c), formatted_width);
  }
}
