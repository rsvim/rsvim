use super::hl::*;
use compact_str::ToCompactString;
use crossterm::style::Attribute;
use crossterm::style::Attributes;
use crossterm::style::Color;

#[cfg(test)]
mod parse_toml {
  use super::*;

  #[test]
  fn toml1() {
    let payload: &str = r##"
[syn]
attribute = "white"
boolean = { fg = "yellow", bold = true }
comment = { fg = "#c0c0c0", bg = "#000000", bold = true, italic = true, underlined = true }
keyword = { fg = "#ffffff", bg = "green", italic = true }

[ui]
background = { bg = "#000000" }

[palette]
# white = "#ffffff"
black = "#000000"
yellow = "#ffff00"
green = "#00ff00"

# Never used
grey = "#c0c0c0"
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("toml1", colorscheme_table).unwrap();
    assert_eq!(cs.syntax().len(), 4);

    let syntax_expects = [
      (
        "syn.attribute",
        Some(Highlight {
          id: "syn.attribute".to_compact_string(),
          fg: Some(Color::White),
          bg: None,
          attr: Attributes::none(),
        }),
      ),
      (
        "syn.boolean",
        Some(Highlight {
          id: "syn.boolean".to_compact_string(),
          fg: Some(Color::Rgb {
            r: 0xff,
            g: 0xff,
            b: 0x00,
          }),
          bg: None,
          attr: Attributes::none().with(Attribute::Bold),
        }),
      ),
      ("syn.carriage-return", None),
      (
        "syn.comment",
        Some(Highlight {
          id: "syn.comment".to_compact_string(),
          fg: Some(Color::Rgb {
            r: 0xc0,
            g: 0xc0,
            b: 0xc0,
          }),
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attr: Attributes::none()
            .with(Attribute::Bold)
            .with(Attribute::Italic)
            .with(Attribute::Underlined),
        }),
      ),
      (
        "syn.keyword",
        Some(Highlight {
          id: "syn.keyword".to_compact_string(),
          fg: Some(Color::Rgb {
            r: 0xff,
            g: 0xff,
            b: 0xff,
          }),
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0xff,
            b: 0x0,
          }),
          attr: Attributes::none().with(Attribute::Italic),
        }),
      ),
    ];
    for expect in syntax_expects.iter() {
      assert_eq!(cs.syntax().get(expect.0), expect.1.as_ref());
    }

    let ui_expects = [
      (
        "ui.background",
        Some(Highlight {
          id: "ui.background".to_compact_string(),
          fg: None,
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attr: Attributes::none(),
        }),
      ),
      ("ui.foreground", None),
    ];

    for expect in ui_expects.iter() {
      assert_eq!(cs.ui().get(expect.0), expect.1.as_ref());
    }

    for expect in syntax_expects.iter() {
      assert_eq!(cs.get(expect.0), expect.1.as_ref());
    }
    for expect in ui_expects.iter() {
      assert_eq!(cs.get(expect.0), expect.1.as_ref());
    }
  }

  #[test]
  fn toml2() {
    let payload: &str = r##"
[syn]
attribute = "white"
boolean = { fg = "yellow", bold = true }
comment = { fg = "#c0c0c0", bg = "#000000", bold = true, italic = true, underlined = true }
keyword = { fg = "red", bg = "green", italic = true }

[ui]
foreground = "#fff"
background = { bg = "#000000" }
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("toml2", colorscheme_table).unwrap();
    assert_eq!(cs.syntax().len(), 4);

    let syntax_expects = [
      (
        "syn.attribute",
        Some(Highlight {
          id: "syn.attribute".to_compact_string(),
          fg: Some(Color::White),
          bg: None,
          attr: Attributes::none(),
        }),
      ),
      (
        "syn.boolean",
        Some(Highlight {
          id: "syn.boolean".to_compact_string(),
          fg: Some(Color::Yellow),
          bg: None,
          attr: Attributes::none().with(Attribute::Bold),
        }),
      ),
      ("syn.carriage-return", None),
      (
        "syn.comment",
        Some(Highlight {
          id: "syn.comment".to_compact_string(),
          fg: Some(Color::Rgb {
            r: 0xc0,
            g: 0xc0,
            b: 0xc0,
          }),
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attr: Attributes::none()
            .with(Attribute::Bold)
            .with(Attribute::Italic)
            .with(Attribute::Underlined),
        }),
      ),
      (
        "syn.keyword",
        Some(Highlight {
          id: "syn.keyword".to_compact_string(),
          fg: Some(Color::Red),
          bg: Some(Color::Green),
          attr: Attributes::none().with(Attribute::Italic),
        }),
      ),
    ];
    for expect in syntax_expects.iter() {
      assert_eq!(cs.syntax().get(expect.0), expect.1.as_ref());
    }

    let ui_expects = [
      (
        "ui.background",
        Some(Highlight {
          id: "ui.background".to_compact_string(),
          fg: None,
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attr: Attributes::none(),
        }),
      ),
      (
        "ui.foreground",
        Some(Highlight {
          id: "ui.foreground".to_compact_string(),
          fg: Some(Color::Rgb {
            r: 0xff,
            g: 0xff,
            b: 0xff,
          }),
          bg: None,
          attr: Attributes::none(),
        }),
      ),
    ];

    for expect in ui_expects.iter() {
      assert_eq!(cs.ui().get(expect.0), expect.1.as_ref());
    }

    for expect in syntax_expects.iter() {
      assert_eq!(cs.get(expect.0), expect.1.as_ref());
    }
    for expect in ui_expects.iter() {
      assert_eq!(cs.get(expect.0), expect.1.as_ref());
    }
  }

  #[test]
  fn failed1() {
    let payload: &str = r##"
[syn]
attribute = "#zxcvas"
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("failed1", colorscheme_table);
    assert!(cs.is_err());
  }
}
