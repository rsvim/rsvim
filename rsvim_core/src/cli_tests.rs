use crate::prelude::*;
use crate::test::log::init as test_log_init;

use clap::Parser;
use std::path::PathBuf;

const ABOUT: &str = "The VIM editor reinvented in Rust+TypeScript.";

#[derive(Parser, Debug, Clone, Default)]
#[command(
  disable_version_flag = true,
  about = ABOUT,
  long_about = ABOUT,
)]
/// Command line options.
pub struct ClapOptions {
  #[arg(short = 'V', long = "version", help = "Print version")]
  version: bool,

  #[arg(help = "Edit file(s)")]
  file: Vec<PathBuf>,
}

impl ClapOptions {
  /// Input files.
  pub fn file(&self) -> &Vec<PathBuf> {
    &self.file
  }

  /// Version.
  pub fn version(&self) -> bool {
    self.version
  }

  pub fn new(version: bool, file: Vec<PathBuf>) -> Self {
    Self { version, file }
  }
}

#[test]
fn clap_help() {
  test_log_init();
  let clap_opts = ClapOptions::new();
  let help = clap_opts.get_help().unwrap();
  info!("clap help:\n{help}");
}
