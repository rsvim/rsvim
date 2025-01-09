//! Display width index (line-wise) for vim buffer.

use crate::buf::opt::BufferLocalOptions;
use crate::buf::unicode;
use ropey::RopeSlice;

use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
/// Display width index (line-wise) for vim buffer. It manages all the
/// [`ColIndex`](crate::buf::ColIndex) and handles the details.
pub struct LineLindex {}
