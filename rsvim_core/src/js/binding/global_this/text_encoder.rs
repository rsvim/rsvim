//! `TextEncoder` APIs.

mod decoder;
mod encoder;

use crate::is_v8_bool;
use crate::is_v8_obj;
use crate::is_v8_str;
use crate::js::binding;
use crate::js::binding::global_this::text_encoder::encoder::TextEncoder;
use crate::js::converter::*;
use crate::prelude::*;
use compact_str::ToCompactString;
use decoder::TextDecoder;
use decoder::TextDecoderBuilder;
use decoder::TextDecoderOptions;
use encoder::TextEncoderBuilder;
use std::cell::Cell;

/// `new TextEncoder()` API.
pub fn create_encoder<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 0);

  let encoder = TextEncoderBuilder::default().build().unwrap();
  let encoder = encoder.to_v8(scope);
  rv.set(encoder.into());
}

#[allow(deprecated)]
// Returns v8 BackingStore data, read (chars), written (bytes)
fn encode_impl<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  payload: v8::Local<'s, v8::String>,
) -> (v8::SharedRef<v8::BackingStore>, usize, usize) {
  let mut buf: Vec<u8> = vec![];
  let mut read: usize = 0;

  // FIXME: Update to `write_utf8_v8` follow deno's implementation:
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
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_obj!(args.get(0)));

  if cfg!(debug_assertions) {
    let encoder =
      TextEncoder::from_v8(scope, args.get(0).to_object(scope).unwrap());
    debug_assert_eq!(encoder.encoding, encoder::ENCODING_DEFAULT);
  }

  debug_assert!(is_v8_str!(args.get(1)));
  let payload = args.get(1).to_string(scope).unwrap();
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
  debug_assert!(args.length() == 3);
  debug_assert!(is_v8_obj!(args.get(0)));

  if cfg!(debug_assertions) {
    let encoder =
      TextEncoder::from_v8(scope, args.get(0).to_object(scope).unwrap());
    debug_assert_eq!(encoder.encoding, encoder::ENCODING_DEFAULT);
  }

  debug_assert!(is_v8_str!(args.get(1)));
  let payload = args.get(1).to_string(scope).unwrap();
  trace!("|encode_into| payload:{:?}", payload);
  debug_assert!(args.get(2).is_uint8_array());
  let buf = args.get(2).cast::<v8::Uint8Array>();

  let store = buf.get_backing_store();
  debug_assert!(store.is_some());
  let mut store = store.unwrap();

  let (new_store, read, written) = encode_impl(scope, payload);
  store.clone_from(&new_store);

  let rv_obj = v8::Object::new(scope);
  let read_value = (read as f64).to_v8(scope);
  binding::set_property_to(scope, rv_obj, "read", read_value.into());
  let written_value = (written as f64).to_v8(scope);
  binding::set_property_to(scope, rv_obj, "written", written_value.into());

  rv.set(rv_obj.into());
}

/// `new TextDecoder()` API.
pub fn create_decoder<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_str!(args.get(0)));
  let encoding = args.get(0).to_rust_string_lossy(scope).to_compact_string();

  debug_assert!(is_v8_obj!(args.get(1)));
  let options =
    TextDecoderOptions::from_v8(scope, args.get(1).to_object(scope).unwrap());

  if encoding_rs::Encoding::for_label(encoding.as_bytes()).is_some() {
    let decoder = TextDecoderBuilder::default()
      .encoding(encoding)
      .fatal(options.fatal())
      .ignore_bom(options.ignore_bom())
      .build()
      .unwrap();
    let decoder = decoder.to_v8(scope);
    rv.set(decoder.into());
  } else {
    let exception = TheErr::TextEncodingInvalid(encoding);
    binding::throw_range_error(scope, &exception);
  }
}

/// `TextDecoder.decode` API.
pub fn decode<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 3);
  debug_assert!(is_v8_obj!(args.get(0)));
  let _decoder =
    TextDecoder::from_v8(scope, args.get(0).to_object(scope).unwrap());

  debug_assert!(args.get(1).is_uint8_array());
  let buf = args.get(1).cast::<v8::Uint8Array>();
  let _buf: Vec<Cell<u8>> = buf.get_backing_store().unwrap().to_vec();

  debug_assert!(is_v8_obj!(args.get(2)));
  let options = args.get(2).to_object(scope).unwrap();
  let stream_name = "stream".to_v8(scope);
  let _stream = if options
    .has_own_property(scope, stream_name.into())
    .unwrap_or(false)
  {
    let stream = options.get(scope, stream_name.into()).unwrap();
    debug_assert!(is_v8_bool!(stream));
    bool::from_v8(scope, stream.to_boolean(scope))
  } else {
    false
  };

  // rv.set(decoder.into());
}
