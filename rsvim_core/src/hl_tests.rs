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

    assert_eq!(*cs.background(), Color::Black);
    assert_eq!(*cs.foreground(), Color::White);

    assert!(cs.scope().get("syn.boolean").is_some());
    assert!(cs.scope().get("syn.boolean").unwrap().bg.is_none());
    assert_eq!(
      cs.scope().get("syn.boolean").unwrap().fg,
      Some(Color::Magenta)
    );
    assert_eq!(
      cs.scope().get("syn.boolean").unwrap().attr,
      Attributes::none()
    );

    assert!(cs.scope().get("syn.variable").is_some());
    assert!(cs.scope().get("syn.variable").unwrap().bg.is_none());
    assert_eq!(
      cs.scope().get("syn.variable").unwrap().fg,
      Some(Color::Cyan)
    );
    assert_eq!(
      cs.scope().get("syn.variable").unwrap().attr,
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
    assert_eq!(cs.scope().len(), 4);

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
      assert_eq!(cs.scope().get(expect.0), expect.1.as_ref());
    }

    assert_eq!(
      *cs.background(),
      Color::Rgb {
        r: 0x0,
        g: 0x0,
        b: 0x0
      }
    );
    assert_eq!(*cs.foreground(), Color::White);
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
    assert_eq!(cs.scope().len(), 4);

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
      assert_eq!(cs.scope().get(expect.0), expect.1.as_ref());
    }

    assert_eq!(
      *cs.background(),
      Color::Rgb {
        r: 0x0,
        g: 0x0,
        b: 0x0
      }
    );
    assert_eq!(
      *cs.foreground(),
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
