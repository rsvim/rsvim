use super::hl::*;

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
white = "#ffffff"
black = "#000000"
yellow = "#ffff00"
green = "#00ff00"

# Never used
grey = "#c0c0c0"
"#;

    let payload_table = payload.parse::<toml::Table>().unwrap();
  }
}
