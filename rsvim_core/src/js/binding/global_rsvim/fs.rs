//! APIs for `Rsvim.fs` namespace.

pub mod fs_file;
pub mod open;

use crate::js::binding::global_rsvim::fs::open::FsOpenOptions;
use crate::js::converter::*;
use crate::prelude::*;

/// `Rsvim.fs.open` API.
pub fn open<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  let filename = args.get(0).to_rust_string_lossy(scope);
  let options = FsOpenOptions::from_v8(scope, args.get(1));
  trace!("Rsvim.fs.open:{:?} {:?}", filename, options);
}
