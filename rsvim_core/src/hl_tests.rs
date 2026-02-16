use super::hl::*;
use crossterm::style::{Attributes, Color};

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
    assert_eq!(cs.ui().len(), 1);
  }
}
