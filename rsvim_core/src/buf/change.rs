//! Text changes on a buffer.

use crate::prelude::*;
use compact_str::CompactString;

pub struct Insert {
  line_idx: usize,
  char_idx: usize,
  payload: CompactString,
}

pub struct Delete {
  line_idx: usize,
  char_idx: usize,
  count: usize,
}

pub struct Replace {
  start_line_idx: usize,
  start_char_idx: usize,
  end_line_idx: usize,
  end_char_idx: usize,
  payload: CompactString,
}

pub struct MultiLineReplace {
  lines: BTreeMap<usize, Replace>,
}

pub enum Change {
  Insert(Insert),
  Delete(Delete),
  Replace(Replace),
  MultiLineReplace(MultiLineReplace),
}
