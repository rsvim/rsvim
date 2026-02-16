use super::hl::*;
use crossterm::style::{Attributes};
use crossterm::style::{Color};
use crossterm::style::Attribute;

#[cfg(test)]
mod parse_toml {
use super::*;

  #[test]
  fn toml1() {
    let payload : &str = r#"
[syn]
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
"#;

    let colorscheme_table = payload.parse::<toml::Table>().unwrap();
    let cs = ColorScheme::from_toml("toml1", colorscheme_table).unwrap();
    assert_eq!(cs.syntax().len(), 4);
    assert!(cs.syntax().get("syn.attribute").is_some());
    assert_eq!(cs.syntax().get("syn.attribute").unwrap(), Highlight {id: "syn.attribute", fg: Option<Color::White>, bg: None, attr: Attributes::none()});
    assert_eq!(cs.syntax().get("syn.boolean").unwrap(), Highlight {id: "syn.boolean", fg: Option<Color::White>, bg: None, attr: Attributes::none()});
    assert_eq!(cs.syntax().get("syn.carriage-return"), None);
    assert_eq!(cs.syntax().get("syn.comment").unwrap(), Highlight { id: "syn.comment", fg: Option<Color::Rgb { r: 0xc0, g: 0xc0, b: 0xc0 }>, bg: Option<Color::Rgb { r: 0x0, g: 0x0, b: 0x0 }>, attr: Attributes::none().with(Attribute::Bold).with(Attribute::Italic).with(Attribute::Underlined) });
    assert_eq!(cs.syntax().get("syn.keyword").unwrap(), Highlight { id: "syn.keyword", fg: Option<Color::Rgb { r: 0xff, g: 0xff, b: 0xff}>, bg: Option<Color::Rgb { r: 0x0, g: 0xff, b: 0x0 }>, attr: Attributes::none().with(Attribute::Italic) });
    assert_eq!(cs.ui().len(), 1);
    assert_eq!(cs.ui().get("ui.background").unwrap(), Highlight { id: "ui.background", fg: Option<Color::Rgb { r: 0x0, g: 0x0, b: 0x0}>, bg: None, attr: Attributes::none() });
  }
}
