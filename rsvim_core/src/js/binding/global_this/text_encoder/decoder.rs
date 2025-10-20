//! TextDecoder and its options

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::from_v8_impl;
use crate::js::binding;
use crate::js::converter::*;
use crate::to_v8_impl;
use compact_str::CompactString;
use compact_str::ToCompactString;

flags_impl!(Flags, u8, FATAL, IGNORE_BOM);

/// Option names.
pub const FATAL: &str = "fatal";
pub const IGNORE_BOM: &str = "ignoreBOM";

/// Default option values.
pub const _FATAL_DEFAULT: bool = false;
pub const _IGNORE_BOM_DEFAULT: bool = false;

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

impl DecoderOptions {
  pub fn fatal(&self) -> bool {
    self.flags.contains(Flags::FATAL)
  }

  pub fn ignore_bom(&self) -> bool {
    self.flags.contains(Flags::IGNORE_BOM)
  }
}

from_v8_impl!(DecoderOptions, [(bool, fatal), (bool, ignore_bom)], []);
to_v8_impl!(DecoderOptions, [fatal, ignore_bom], [], [], []);

pub const ENCODING: &str = "encoding";

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct Decoder {
  pub options: DecoderOptions,
  pub encoding: CompactString,
}

from_v8_impl!(
  Decoder,
  [(String, encoding), (bool, fatal), (bool, ignore_bom)],
  []
);
to_v8_impl!(Decoder, [encoding, fatal, ignore_bom], [], [], []);
