//! Text changes on a buffer.

use compact_str::CompactString;

pub enum Change {}

pub struct Insert {
  line_idx: usize,
  char_idx: usize,
  payload: CompactString,
}
pub struct Delete {}
