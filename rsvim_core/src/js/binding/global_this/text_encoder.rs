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
  let payload = args.get(0).to_string(scope).unwrap();
  trace!("|encode| payload:{:?}", payload.to_rust_string_lossy(scope));

  let payload = payload.write_utf8_v2());

  let nfc = ComposingNormalizerBorrowed::new_nfc();
  let normalized = nfc.normalize(&payload);
  let (result, _actual_encoding, _had_unmappable) =
    encoding_rs::UTF_8.encode(&normalized);

  let store = v8::ArrayBuffer::new_backing_store_from_vec(result.into_owned());
  let buf = v8::ArrayBuffer::with_backing_store(scope, &store.make_shared());
  let buf = v8::Uint8Array::new(scope, buf, 0, buf.byte_length()).unwrap();

  rv.set(buf.into());
}

/// `TextEncoder.encodeInto` API.
pub fn encode_into<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  let payload = from_v8::<String>(scope, args.get(0));
  let buf = args.get(1);
  debug_assert!(buf.is_uint8_array());
  trace!("|encode_into| payload:{:?}", payload);

  let nfc = ComposingNormalizerBorrowed::new_nfc();
  let normalized = nfc.normalize(&payload);
  let (result, _actual_encoding, _had_unmappable) =
    encoding_rs::UTF_8.encode(&normalized);

  let store = v8::ArrayBuffer::new_backing_store_from_vec(result.into_owned());
  let buf = v8::ArrayBuffer::with_backing_store(scope, &store.make_shared());
  let buf = v8::Uint8Array::new(scope, buf, 0, buf.byte_length()).unwrap();

  rv.set(buf.into());
}
