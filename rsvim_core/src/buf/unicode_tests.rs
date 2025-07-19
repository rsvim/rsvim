use super::unicode::*;

use crate::buf::opt::{BufferLocalOptionsBuilder, FileFormatOption};
use crate::defaults::ascii::AsciiControlCodeFormatter;
use crate::test::log::init as test_log_init;

use ascii::AsciiChar;
use icu_properties::{CodePointMapData, props::EastAsianWidth};
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
    info!(
      "i:{i},c:{c:?},ascii char:{asciic:?},ascii formatted:{formatted:?}({formatted_len})"
    );
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

#[test]
fn ascii_characters_test1() {
  test_log_init();

  let ascii_characters =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

  let code_point_map = CodePointMapData::<EastAsianWidth>::new();

  for (i, c) in ascii_characters.chars().enumerate() {
    let w1 = UnicodeWidthChar::width_cjk(c);
    let w2 = code_point_map.get(c);
    let w2_name = match w2 {
      EastAsianWidth::Halfwidth => "Halfwidth",
      EastAsianWidth::Narrow => "Narrow",
      EastAsianWidth::Ambiguous => "Ambiguous",
      EastAsianWidth::Fullwidth => "Fullwidth",
      EastAsianWidth::Neutral => "Neutral",
      EastAsianWidth::Wide => "Wide",
      _ => "Unknown",
    };
    info!("i:{i},c:{c:?}, unicode_width:{w1:?}, icu:{w2:?}({w2_name})");
  }
}

#[test]
fn special_characters_test1() {
  test_log_init();

  let special_characters = vec![
    '!', '"', '#', '$', '%', '@', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
    ':', ';', '<', '=', '>', '?', '@', '[', ']', '\\', '^', '_', '`', '{', '}',
    '|', '~', 'Ç', 'ü', 'é', 'â', 'ä', 'à', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ï',
    'ì', 'Ä', 'Å', 'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', 'ø',
    '£', 'Ø', '×', 'ƒ', 'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º', '¿', '®', '¬',
    '½', '¼', '¡', '«', '»', '░', '▒', '▓', '│', '┤', 'Á', 'Â', 'À', '©', '╣',
    '║', '╗', '╝', '¢', '¥', '┐', 'Ó', 'ß', 'Ô', 'Ò', 'õ', 'Õ', 'µ', 'þ', 'Þ',
    'Ú', 'Û', 'Ù', 'ý', 'Ý', '¯', '´', '≡', '±', '‗', '¾', '¶', '§', '÷', '¸',
    '°', '¨', '·', '¹', '³', '²', '■',
  ];

  let code_point_map = CodePointMapData::<EastAsianWidth>::new();

  for (i, c) in special_characters.iter().enumerate() {
    let w1 = UnicodeWidthChar::width_cjk(*c);
    let w2 = code_point_map.get(*c);
    let w2_name = match w2 {
      EastAsianWidth::Halfwidth => "Halfwidth",
      EastAsianWidth::Narrow => "Narrow",
      EastAsianWidth::Ambiguous => "Ambiguous",
      EastAsianWidth::Fullwidth => "Fullwidth",
      EastAsianWidth::Neutral => "Neutral",
      EastAsianWidth::Wide => "Wide",
      _ => "Unknown",
    };
    info!("i:{i},c:{c:?}, unicode_width:{w1:?}, icu:{w2:?}({w2_name})");
  }
}

#[test]
fn cjk_characters_test1() {
  test_log_init();

  let cjk_characters = vec![
    '你', '好', 'こ', 'ん', 'に', 'ち', 'は', '안', '녕', '하', '세', '요',
  ];

  let code_point_map = CodePointMapData::<EastAsianWidth>::new();

  for (i, c) in cjk_characters.iter().enumerate() {
    let w1 = UnicodeWidthChar::width_cjk(*c);
    let w2 = code_point_map.get(*c);
    let w2_name = match w2 {
      EastAsianWidth::Halfwidth => "Halfwidth",
      EastAsianWidth::Narrow => "Narrow",
      EastAsianWidth::Ambiguous => "Ambiguous",
      EastAsianWidth::Fullwidth => "Fullwidth",
      EastAsianWidth::Neutral => "Neutral",
      EastAsianWidth::Wide => "Wide",
      _ => "Unknown",
    };
    info!("i:{i},c:{c:?}, unicode_width:{w1:?}, icu:{w2:?}({w2_name})");
  }
}
