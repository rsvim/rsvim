//! File path utils.

use crate::prelude::*;
use std::ffi::OsStr;

/// Map path to its parent, or remain unchanged if it has no parent.
pub fn parent_or_remain<S: AsRef<OsStr> + ?Sized>(s: &S) -> &Path {
  Path::new(s).parent().unwrap_or(Path::new(s))
}

/// Convert path to string.
pub fn path_string(path: &Path) -> &str {
  path.as_os_str().to_str().unwrap()
}
