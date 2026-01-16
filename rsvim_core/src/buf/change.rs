//! Buffer changes.

use crate::buf::text::Text;
use crate::prelude::*;
use compact_str::CompactString;
use path_absolutize::Absolutize;
use std::fs::Metadata;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insert {
  char_idx: usize,
  payload: CompactString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delete {
  char_idx: usize,
  count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Basic unit of a change operation:
/// - Insert payload at an absolute char index.
/// - Delete `n` characters at an absolute char index.
///
/// The "Replace" operation can be converted into delete+insert operations.
///
/// The change operation doesn't maintain current cursor's position, so a
/// buffer can change without need to know where the cursor is.
///
/// NOTE: Ropey provide two types of coordinate system:
/// 1. 2-Dimension on line number and char index per line.
/// 2. 1-Dimension on absolute char index per whole buffer.
pub enum Operation {
  Insert(Insert),
  Delete(Delete),
}

#[derive(Debug, Clone)]
pub struct Changes {
  operations: Vec<Operation>,
}
