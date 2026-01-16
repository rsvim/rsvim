//! Buffer changes.

use crate::buf::text::Text;
use crate::prelude::*;
use compact_str::CompactString;
use path_absolutize::Absolutize;
use std::fs::Metadata;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

pub struct Insert {
  char_idx: usize,
  payload: CompactString,
}

pub struct Delete {
  char_idx: usize,
  count: usize,
}

pub struct Replace {
  start_char_idx: usize,
  end_char_idx: usize,
  payload: CompactString,
}

pub struct BufferChange {
  text: Text,
}
