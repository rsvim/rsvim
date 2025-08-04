//! Command line options.

use std::path::PathBuf;

#[derive(Debug, Clone)]
/// Command line options.
pub struct CliOpt {
  /// Input files.
  pub file: Vec<PathBuf>,
}
