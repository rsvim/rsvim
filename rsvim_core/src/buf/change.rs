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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Basic unit of a change operation:
/// - Insert payload at an absolute char index.
/// - Delete `n` characters at an absolute char index.
///
/// NOTE: Ropey provide two types of coordinate system:
/// 1. 2-Dimension on line number and char index per line.
/// 2. 1-Dimension on absolute char index per whole buffer.
pub enum ChangeOperation {
  Retain(usize),
}
