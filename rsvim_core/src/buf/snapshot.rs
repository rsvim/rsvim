//! Buffer snapshot.

use crate::buf::text::Text;
use crate::prelude::*;
use path_absolutize::Absolutize;
use std::fs::Metadata;
use std::path::Path;
use std::path::PathBuf;
use tokio::time::Instant;

pub struct BufferSnapshot {
  text: Text,
}
