//! Vim cmdline.

use crate::content::TemporaryContentsWk;

#[derive(Debug, Clone)]
/// The Vim cmdline.
pub struct Cmdline {
  // Temporary contents for cmdline content.
  _contents: TemporaryContentsWk,
}
