use super::hl::*;
use compact_str::ToCompactString;
use crossterm::style::Attribute;
use crossterm::style::Attributes;
use crossterm::style::Color;

#[cfg(test)]
mod parse_toml {
  use super::*;

  #[test]
  fn default1() {
    let cs_manager = ColorSchemeManager::new();
    let cs = cs_manager.get(DEFAULT).unwrap();

    assert!(cs.colors().get("ui.background").is_none());
    assert!(cs.colors().get("ui.foreground").is_none());

    assert!(cs.highlights().get("scope.boolean").is_some());
    assert!(cs.highlights().get("scope.boolean").unwrap().bg.is_none());
    assert_eq!(
      cs.highlights().get("scope.boolean").unwrap().fg,
      Some(Color::Magenta)
    );
    assert_eq!(
      cs.highlights().get("scope.boolean").unwrap().attr,
      Attributes::none()
    );

    assert!(cs.highlights().get("scope.variable").is_some());
    assert!(cs.highlights().get("scope.variable").unwrap().bg.is_none());
    assert_eq!(
      cs.highlights().get("scope.variable").unwrap().fg,
      Some(Color::Cyan)
    );
    assert_eq!(
      cs.highlights().get("scope.variable").unwrap().attr,
      Attributes::none()
    );
  }

  #[test]
  fn toml1() {
    let payload: &str = r##"
[scope]
attribute = "white"
boolean = { fg = "yellow", bold = true }
comment = { fg = "#c0c0c0", bg = "#000000", bold = true, italic = true, underlined = true }
keyword = { fg = "#ffffff", bg = "green", italic = true }

[ui]
background = "#000000"

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
    assert_eq!(cs.highlights().len(), 4);

    let scope_expects = [
      (
        "scope.attribute",
        Some(Highlight {
          id: "scope.attribute".to_compact_string(),
          fg: Some(Color::White),
          bg: None,
          attr: Attributes::none(),
        }),
      ),
      (
        "scope.boolean",
        Some(Highlight {
          id: "scope.boolean".to_compact_string(),
          fg: Some(Color::Rgb {
            r: 0xff,
            g: 0xff,
            b: 0x00,
          }),
          bg: None,
          attr: Attributes::none().with(Attribute::Bold),
        }),
      ),
      ("scope.carriage-return", None),
      (
        "scope.comment",
        Some(Highlight {
          id: "scope.comment".to_compact_string(),
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
        "scope.keyword",
        Some(Highlight {
          id: "scope.keyword".to_compact_string(),
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
    for expect in scope_expects.iter() {
      assert_eq!(cs.highlights().get(expect.0), expect.1.as_ref());
    }

    assert_eq!(
      *cs.colors().get("ui.background").unwrap(),
      Color::Rgb {
        r: 0x0,
        g: 0x0,
        b: 0x0
      }
    );
    assert!(cs.colors().get("ui.foreground").is_none());
  }

  #[test]
  fn toml2() {
    let payload: &str = r##"
[scope]
attribute = "white"
boolean = { fg = "yellow", bold = true }
comment = { fg = "#c0c0c0", bg = "#000000", bold = true, italic = true, underlined = true }
keyword = { fg = "red", bg = "green", italic = true }

[ui]
foreground = "#fff"
background = "#000000"
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("toml2", colorscheme_table).unwrap();
    assert_eq!(cs.highlights().len(), 4);

    let scope_expects = [
      (
        "scope.attribute",
        Some(Highlight {
          id: "scope.attribute".to_compact_string(),
          fg: Some(Color::White),
          bg: None,
          attr: Attributes::none(),
        }),
      ),
      (
        "scope.boolean",
        Some(Highlight {
          id: "scope.boolean".to_compact_string(),
          fg: Some(Color::Yellow),
          bg: None,
          attr: Attributes::none().with(Attribute::Bold),
        }),
      ),
      ("scope.carriage-return", None),
      (
        "scope.comment",
        Some(Highlight {
          id: "scope.comment".to_compact_string(),
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
        "scope.keyword",
        Some(Highlight {
          id: "scope.keyword".to_compact_string(),
          fg: Some(Color::Red),
          bg: Some(Color::Green),
          attr: Attributes::none().with(Attribute::Italic),
        }),
      ),
    ];
    for expect in scope_expects.iter() {
      assert_eq!(cs.highlights().get(expect.0), expect.1.as_ref());
    }

    assert_eq!(
      *cs.colors().get("ui.background").unwrap(),
      Color::Rgb {
        r: 0x0,
        g: 0x0,
        b: 0x0
      }
    );
    assert_eq!(
      *cs.colors().get("ui.foreground").unwrap(),
      Color::Rgb {
        r: 0xff,
        g: 0xff,
        b: 0xff
      }
    );
  }

  #[test]
  fn failed1() {
    let payload: &str = r##"
[scope]
attribute = "#zxcvas"
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("failed1", colorscheme_table);
    assert!(cs.is_err());
  }
}
