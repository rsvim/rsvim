use super::hl::*;
use crate::cli::CliOptions;
use crate::evloop::EventLoop;
use crate::prelude::*;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::Operation;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use assert_fs::prelude::PathChild;
use compact_str::ToCompactString;
use std::time::Duration;

#[cfg(test)]
mod parser {
  use super::*;

  #[test]
  fn toml1() {
    let payload : &str = r#"
[syn]
attribute = "white"
boolean = {{ fg = "yellow", bold = true }}

[ui]
background = "#000000"

[palette]
white = "#ffffff"
black = "#000000"
yellow = "#ffff00"
"#;

    let payload_table = payload.parse::<toml::Table>().unwrap();
  }
}
