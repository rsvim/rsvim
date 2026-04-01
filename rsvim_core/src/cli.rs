//! Command line options.

use clap::Parser;
use std::path::PathBuf;

const ABOUT: &str = "RSVIM - The VIM editor reinvented in Rust+TypeScript";

#[derive(Debug, Clone, Parser)]
#[command(
  disable_version_flag = true,
  about = ABOUT,
)]
/// Command line options.
pub struct CliOptions {
  #[arg(short = 'V', long = "version", help = "Print version")]
  version: bool,

  #[arg(help = "Edit file(s)")]
  // Files
  file: Vec<PathBuf>,
}

impl CliOptions {
  /// Input files.
  pub fn file(&self) -> &Vec<PathBuf> {
    &self.file
  }

  #[cfg(test)]
  pub fn new(version: bool, file: Vec<PathBuf>) -> Self {
    Self { version, file }
  }

  #[cfg(test)]
  pub fn empty() -> Self {
    Self {
      version: false,
      file: vec![],
    }
  }
}
