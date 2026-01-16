//! Buffer snapshot.

use crate::buf::text::Text;
use crate::prelude::*;
use path_absolutize::Absolutize;
use std::fs::Metadata;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

pub struct BufferSnapshot {
  text: Text,
  filename: Option<PathBuf>,
  absolute_filename: Option<PathBuf>,
  metadata: Option<Metadata>,
  last_sync_time: Option<Instant>,
}
