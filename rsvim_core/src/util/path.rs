//! File path utils.

use std::ffi::OsStr;
use std::path::Path;

/// Map path to its parent, or remain unchanged if there's no parent.
///
pub fn parent_or_remain<S: AsRef<OsStr> + ?Sized>(s: &S) -> &Path {
  Path::new(s).parent().unwrap_or(Path::new(s))
}

/// Convert path to string.
pub fn to_string(path: &Path) -> &str {
  path.as_os_str().to_str().unwrap()
}
