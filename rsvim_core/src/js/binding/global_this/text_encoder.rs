//! `TextEncoder` APIs.

use compact_str::CompactString;

use crate::js::binding;
use crate::js::converter::*;
use crate::prelude::*;
// use icu::normalizer::ComposingNormalizerBorrowed;
// use icu::normalizer::DecomposingNormalizerBorrowed;

#[allow(deprecated)]
// Returns v8 BackingStore data, read (chars), written (bytes)
fn encode_impl<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  payload: v8::Local<'s, v8::String>,
) -> (v8::SharedRef<v8::BackingStore>, usize, usize) {
  let mut buf: Vec<u8> = vec![];
  let mut read: usize = 0;

  // FIXME: Update to `write_utf8_v8` follow deno's implementation:
  //
  // https://github.com/denoland/deno/blob/v2.5.4/ext/web/08_text_encoding.js#L256
  // https://github.com/denoland/deno/blob/v2.5.4/ext/web/lib.rs#L367
  let written = payload.write_utf8(
    scope,
    &mut buf,
    Some(&mut read),
    v8::WriteOptions::NO_NULL_TERMINATION
      | v8::WriteOptions::REPLACE_INVALID_UTF8,
  );
  trace!("|encode_utf8| written:{:?}, read:{:?}", written, read);

  let store = v8::ArrayBuffer::new_backing_store_from_vec(buf);

  (store.make_shared(), read, written)
}

/// `TextEncoder.encode` API.
pub fn encode<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let payload = args.get(0).to_string(scope).unwrap();
  trace!("|encode| payload:{:?}", payload.to_rust_string_lossy(scope));

  let (store, _read, _written) = encode_impl(scope, payload);

  let buf = v8::ArrayBuffer::with_backing_store(scope, &store);
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
  let payload = args.get(0).to_string(scope).unwrap();
  let buf = args.get(1);
  debug_assert!(buf.is_uint8_array());
  trace!("|encode_into| payload:{:?}", payload);
  let buf = buf.cast::<v8::Uint8Array>();

  let store = buf.get_backing_store();
  debug_assert!(store.is_some());
  let mut store = store.unwrap();

  let (new_store, read, written) = encode_impl(scope, payload);
  store.clone_from(&new_store);

  let rv_obj = v8::Object::new(scope);
  let read_value = to_v8(scope, read as f64);
  binding::set_property_to(scope, rv_obj, "read", read_value);
  let written_value = to_v8(scope, written as f64);
  binding::set_property_to(scope, rv_obj, "written", written_value);

  rv.set(rv_obj.into());
}

/// `TextEncoder.encoding` property.
pub fn encoding<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 0);

  let encoding_value = to_v8(scope, encoding_rs::UTF_8.name());
  rv.set(encoding_value);
}

/// `new TextDecoder()` API.
pub fn create_decoder<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  let name = from_v8::<CompactString>(scope, args.get(0));

  let encoding_value = to_v8(scope, encoding_rs::UTF_8.name());
  rv.set(encoding_value);
}
