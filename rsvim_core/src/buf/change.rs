//! Buffer changes.

use crate::buf::text::Text;
use crate::prelude::*;
use compact_str::CompactString;
use path_absolutize::Absolutize;
use std::fs::Metadata;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

pub struct Retain {
  char_idx: usize,
}

pub struct Insert {
  payload: CompactString,
}

pub struct Delete {
  count: usize,
}

pub struct ChangeOperation {
  text: Text,
}
