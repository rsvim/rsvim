use super::hl::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use crossterm::style::Attribute;
use crossterm::style::Attributes;
use crossterm::style::Color;

#[cfg(test)]
mod parse_toml {
  use super::*;

  #[test]
  fn default1() {
    test_log_init();

    let cs_manager = ColorSchemeManager::new();
    let cs = cs_manager.get(DEFAULT).unwrap();
    info!("cs:{:?}", cs);

    assert!(cs.colors().get("ui.background").is_some());
    assert!(cs.colors().get("ui.text").is_some());

    assert!(cs.highlights().get("boolean").is_some());
    info!("boolean:{:?}", cs.highlights().get("boolean"));
    assert_eq!(
      cs.highlights().get("boolean").unwrap().bg,
      Some(Color::Black)
    );
    assert_eq!(
      cs.highlights().get("boolean").unwrap().fg,
      Some(Color::Magenta)
    );
    assert_eq!(
      cs.highlights().get("boolean").unwrap().attrs,
      Attributes::none()
    );

    assert!(cs.highlights().get("variable").is_some());
    info!("variable:{:?}", cs.highlights().get("variable"));
    assert_eq!(
      cs.highlights().get("variable").unwrap().bg,
      Some(Color::Black)
    );
    assert_eq!(
      cs.highlights().get("variable").unwrap().fg,
      Some(Color::Cyan)
    );
    assert_eq!(
      cs.highlights().get("variable").unwrap().attrs,
      Attributes::none()
    );
  }

  #[test]
  fn toml1() {
    test_log_init();

    let payload: &str = r##"
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
        "attribute",
        Some(Highlight {
          fg: Some(Color::White),
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attrs: Attributes::none(),
        }),
      ),
      (
        "boolean",
        Some(Highlight {
          fg: Some(Color::Rgb {
            r: 0xff,
            g: 0xff,
            b: 0x00,
          }),
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attrs: Attributes::none().with(Attribute::Bold),
        }),
      ),
      ("carriage-return", None),
      (
        "comment",
        Some(Highlight {
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
          attrs: Attributes::none()
            .with(Attribute::Bold)
            .with(Attribute::Italic)
            .with(Attribute::Underlined),
        }),
      ),
      (
        "keyword",
        Some(Highlight {
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
          attrs: Attributes::none().with(Attribute::Italic),
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
    assert!(cs.colors().get("ui.text").is_none());
  }

  #[test]
  fn toml2() {
    test_log_init();

    let payload: &str = r##"
attribute = "white"
boolean = { fg = "yellow", bold = true }
comment = { fg = "#c0c0c0", bg = "#000000", bold = true, italic = true, underlined = true }
keyword = { fg = "red", bg = "green", italic = true }

[ui]
text = "#fff"
background = "#000000"
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("toml2", colorscheme_table).unwrap();
    assert_eq!(cs.highlights().len(), 4);

    let scope_expects = [
      (
        "attribute",
        Some(Highlight {
          fg: Some(Color::White),
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attrs: Attributes::none(),
        }),
      ),
      (
        "boolean",
        Some(Highlight {
          fg: Some(Color::Yellow),
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attrs: Attributes::none().with(Attribute::Bold),
        }),
      ),
      ("carriage-return", None),
      (
        "comment",
        Some(Highlight {
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
          attrs: Attributes::none()
            .with(Attribute::Bold)
            .with(Attribute::Italic)
            .with(Attribute::Underlined),
        }),
      ),
      (
        "keyword",
        Some(Highlight {
          fg: Some(Color::Red),
          bg: Some(Color::Green),
          attrs: Attributes::none().with(Attribute::Italic),
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
      *cs.colors().get("ui.text").unwrap(),
      Color::Rgb {
        r: 0xff,
        g: 0xff,
        b: 0xff
      }
    );
  }

  #[test]
  fn toml3() {
    test_log_init();

    let payload: &str = r##"
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
    assert!(!cs.highlights().is_empty());

    let scope_expects = [
      (
        "attribute",
        Some(Highlight {
          fg: Some(Color::White),
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attrs: Attributes::none(),
        }),
      ),
      (
        "boolean",
        Some(Highlight {
          fg: Some(Color::Rgb {
            r: 0xff,
            g: 0xff,
            b: 0x00,
          }),
          bg: Some(Color::Rgb {
            r: 0x0,
            g: 0x0,
            b: 0x0,
          }),
          attrs: Attributes::none().with(Attribute::Bold),
        }),
      ),
      ("carriage-return", None),
      (
        "comment",
        Some(Highlight {
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
          attrs: Attributes::none()
            .with(Attribute::Bold)
            .with(Attribute::Italic)
            .with(Attribute::Underlined),
        }),
      ),
      (
        "keyword",
        Some(Highlight {
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
          attrs: Attributes::none().with(Attribute::Italic),
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
    assert!(cs.colors().get("ui.text").is_none());
  }

  #[test]
  fn failed1() {
    test_log_init();

    let payload: &str = r##"
attribute = "#zxcvas"
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("failed1", colorscheme_table);
    assert!(cs.is_err());
    if let Err(e) = cs {
      info!("error:{:?}", e.to_string());
    }
  }

  #[test]
  fn failed2() {
    test_log_init();

    let payload: &str = r##"
attribute = "white"
rustsrcour = "#ffffff"
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("failed2", colorscheme_table);
    assert!(cs.is_err());
    if let Err(e) = cs {
      info!("error:{:?}", e.to_string());
    }
  }

  #[test]
  fn failed3() {
    test_log_init();

    let payload: &str = r##"
attribute = "beijing"
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("failed3", colorscheme_table);
    assert!(cs.is_err());
    if let Err(e) = cs {
      info!("error:{:?}", e.to_string());
    }
  }

  #[test]
  fn failed4() {
    test_log_init();

    let payload: &str = r##"
[ui]
text = { fg = "white" }
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("failed4", colorscheme_table);
    assert!(cs.is_err());
    if let Err(e) = cs {
      info!("error:{:?}", e.to_string());
    }
  }

  #[test]
  fn failed5() {
    test_log_init();

    let payload: &str = r##"
[palette]
white = true
"##;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("failed5", colorscheme_table);
    assert!(cs.is_err());
    if let Err(e) = cs {
      info!("error:{:?}", e.to_string());
    }
  }
}
