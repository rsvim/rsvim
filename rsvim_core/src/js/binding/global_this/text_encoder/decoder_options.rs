//! TextDecoder options

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::js::binding;
use crate::js::converter::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::str::FromStr;

flags_impl!(Flags, u8, FATAL, IGNORE_BOM);

/// Attribute names.
pub const FATAL: &str = "fatal";
pub const IGNORE_BOM: &str = "ignoreBOM";

/// Default attribute values.
pub const FATAL_DEFAULT: bool = false;
pub const IGNORE_BOM_DEFAULT: bool = false;

// fatal=false
// ignoreBOM=false
const FLAGS: Flags = Flags::empty();

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct DecoderOptions {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // fatal
  // ignoreBOM
  flags: Flags,
}

flags_builder_impl!(DecoderOptionsBuilder, flags, Flags, fatal, ignore_bom);
