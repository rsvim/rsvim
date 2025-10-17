//! `TextEncoder` APIs.

use crate::js;
use crate::js::JsFuture;
use crate::js::JsRuntime;
use crate::js::JsTimerId;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use icu::normalizer::ComposingNormalizerBorrowed;
use icu::normalizer::DecomposingNormalizerBorrowed;
use std::rc::Rc;

/// `TextEncoder.encode` API.
pub fn encode<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let payload = from_v8::<String>(scope, args.get(0));
  trace!("|encode| payload:{:?}", payload);

  let nfc = ComposingNormalizerBorrowed::new_nfc();
  let normalized = nfc.normalize(&payload);
  let (result, _actual_encoding, _had_unmappable) =
    encoding_rs::UTF_8.encode(&normalized);

  let buf = to_v8_uint8_array(scope, result.into_owned());

  rv.set(buf.into());
}
