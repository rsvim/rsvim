//! File path utils.

use std::ffi::OsStr;
use std::path::Path;

/// Map path to its parent, or remain unchanged if it has no parent.
pub fn maybe_parent<S: AsRef<OsStr> + ?Sized>(s: &S) -> &Path {
  Path::new(s).parent().unwrap_or(Path::new(s))
}
