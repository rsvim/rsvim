//! `TextEncoder` APIs.

mod encoder;

use crate::is_v8_bool;
use crate::is_v8_obj;
use crate::is_v8_str;
use crate::js::binding;
use crate::js::converter::*;
use crate::prelude::*;
use compact_str::ToCompactString;
use encoder::EncodeIntoResultBuilder;
use encoding_rs::CoderResult;
use encoding_rs::Decoder;
use encoding_rs::DecoderResult;
use encoding_rs::Encoding;
use itertools::Itertools;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

// Returns v8 BackingStore data, read (chars), written (bytes)
fn encode_impl<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  payload: v8::Local<'s, v8::String>,
  buf_size: usize,
) -> (Vec<u8>, usize, usize) {
  let mut buf: Vec<u8> = Vec::with_capacity(buf_size);
  let mut read: usize = 0;

  // FIXME: Update to `write_utf8_v2` API.
  // This implementation follows deno's "op_encoding_encode_into" API:
  // https://github.com/denoland/deno/blob/v2.5.4/ext/web/lib.rs#L367
  #[allow(deprecated)]
  let written = payload.write_utf8_uninit(
    scope,
    // Step-1: Get uninit buffer inside vec
    buf.spare_capacity_mut(),
    Some(&mut read),
    v8::WriteOptions::NO_NULL_TERMINATION
      | v8::WriteOptions::REPLACE_INVALID_UTF8,
  );
  unsafe {
    // Step-2: Set length for the buffer, because it doesn't know memory
    // changes made by v8 "writeUtf8" API.
    buf.set_len(written);
  }
  trace!(
    "|encode_impl| written:{:?}, read:{:?}, buf.len:{:?}",
    written,
    read,
    buf.len()
  );

  (buf, read, written)
}

/// `TextEncoder.encode` API.
pub fn encode<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  debug_assert!(is_v8_str!(args.get(0)));
  let payload = args.get(0).to_string(scope).unwrap();
  trace!("|encode| payload:{:?}", payload.to_rust_string_lossy(scope));

  let buf_size = payload.utf8_length(scope);
  let (data, _read, _written) = encode_impl(scope, payload, buf_size);
  debug_assert_eq!(_written, data.len());

  let store = v8::ArrayBuffer::new_backing_store_from_vec(data);
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
  debug_assert!(is_v8_str!(args.get(0)));
  let payload = args.get(0).to_string(scope).unwrap();
  trace!(
    "|encode_into| payload:{:?}",
    payload.to_rust_string_lossy(scope)
  );

  debug_assert!(args.get(1).is_array_buffer());
  let buf = args.get(1).cast::<v8::ArrayBuffer>();
  let buf_size = buf.byte_length();
  let buf_store = buf.get_backing_store();

  let (data, read, written) = encode_impl(scope, payload, buf_size);

  debug_assert_eq!(written, data.len());
  debug_assert!(data.len() <= buf_size);
  for (i, b) in data.iter().enumerate() {
    buf_store[i].set(*b);
  }

  let encode_result = EncodeIntoResultBuilder::default()
    .read(read as u32)
    .written(written as u32)
    .build()
    .unwrap();
  let encode_result = encode_result.to_v8(scope);

  rv.set(encode_result.into());
}

/// Check whether encoding label is valid.
pub fn check_encoding_label<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  debug_assert!(is_v8_str!(args.get(0)));
  let label = args.get(0).to_rust_string_lossy(scope).to_compact_string();
  let valid = Encoding::for_label(label.as_bytes()).is_some();
  rv.set_bool(valid);
}

fn create_decoder_impl(label: &str, ignore_bom: bool) -> Decoder {
  let encoding = Encoding::for_label(label.as_bytes()).unwrap();
  if ignore_bom {
    encoding.new_decoder_without_bom_handling()
  } else {
    encoding.new_decoder_with_bom_removal()
  }
}

fn decode_impl<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  rv: &mut v8::ReturnValue,
  decoder: &mut Decoder,
  data: &[u8],
  fatal: bool,
  stream: bool,
) {
  let max_buffer_length = decoder.max_utf16_buffer_length(data.len());

  if max_buffer_length.is_none() {
    binding::throw_range_error(scope, &TheErr::ValueTooLarge(data.len()));
    return;
  }

  let max_buffer_length = max_buffer_length.unwrap();
  let mut output = vec![0; max_buffer_length];

  if fatal {
    let (result, _read, written) =
      decoder.decode_to_utf16_without_replacement(data, &mut output, !stream);
    match result {
      DecoderResult::InputEmpty => {
        output.truncate(written);
        let output = v8::String::new_from_two_byte(
          scope,
          &output,
          v8::NewStringType::Normal,
        )
        .unwrap();
        rv.set(output.into());
      }
      DecoderResult::OutputFull => {
        binding::throw_type_error(
          scope,
          &TheErr::BufferTooSmall(max_buffer_length),
        );
      }
      DecoderResult::Malformed(_, _) => {
        binding::throw_type_error(scope, &TheErr::DataInvalid);
      }
    }
  } else {
    let (result, _read, written, _had_errors) =
      decoder.decode_to_utf16(data, &mut output, !stream);
    match result {
      CoderResult::InputEmpty => {
        output.truncate(written);
        let output = v8::String::new_from_two_byte(
          scope,
          &output,
          v8::NewStringType::Normal,
        )
        .unwrap();
        rv.set(output.into());
      }
      CoderResult::OutputFull => {
        binding::throw_type_error(
          scope,
          &TheErr::BufferTooSmall(max_buffer_length),
        );
      }
    }
  }
}

/// `TextDecoder.decode` API on non-stream, single pass.
pub fn decode_single<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(args.get(0).is_array_buffer());
  let data = args.get(0).cast::<v8::ArrayBuffer>();
  let data: Vec<u8> = data
    .get_backing_store()
    .iter()
    .map(|b| b.get())
    .collect_vec();
  debug_assert!(is_v8_str!(args.get(1)));
  let label = args.get(1).to_rust_string_lossy(scope);
  debug_assert!(is_v8_bool!(args.get(2)));
  let fatal = bool::from_v8(scope, args.get(2).to_boolean(scope));
  debug_assert!(is_v8_bool!(args.get(3)));
  let ignore_bom = bool::from_v8(scope, args.get(3).to_boolean(scope));
  trace!(
    "|decode_single| data:{:?}, label:{:?}, fatal:{:?}, ignore_bom:{:?}",
    data, label, fatal, ignore_bom
  );

  let mut decoder = create_decoder_impl(&label, ignore_bom);
  decode_impl(scope, &mut rv, &mut decoder, &data, fatal, false);
}

/// `new TextDecoder()` API for stream decoding.
pub fn create_stream_decoder<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_str!(args.get(0)));
  let label = args.get(0).to_rust_string_lossy(scope);
  debug_assert!(is_v8_bool!(args.get(1)));
  let ignore_bom = bool::from_v8(scope, args.get(1).to_boolean(scope));
  trace!(
    "|create_stream_decoder| label:{:?}, ignore_bom:{:?}",
    label, ignore_bom
  );

  let decoder_handle = RefCell::new(create_decoder_impl(&label, ignore_bom));

  let decoder_wrapper = v8::ObjectTemplate::new(scope);

  // Allocate internal field:
  // 1. encoding_rs::Decoder
  // 2. weak_rc
  decoder_wrapper.set_internal_field_count(2);
  let decoder_wrapper = decoder_wrapper.new_instance(scope).unwrap();

  let decoder_ptr = binding::set_internal_ref::<RefCell<Decoder>>(
    scope,
    decoder_wrapper,
    0,
    decoder_handle,
  );
  let weak_rc = Rc::new(Cell::new(None));

  // To automatically drop the decoder_handle instance when
  // V8 garbage collects the object that internally holds the Rust instance,
  // we use a Weak reference with a finalizer callback.
  let decoder_weak = v8::Weak::with_finalizer(
    scope,
    decoder_wrapper,
    Box::new({
      let weak_rc = weak_rc.clone();
      let _encoding_label = label.clone();
      move |isolate| unsafe {
        drop(Box::from_raw(decoder_ptr));
        drop(v8::Weak::from_raw(isolate, weak_rc.get()));
        trace!("|create_decoder| dropped TextDecoder:{:?}", _encoding_label);
      }
    }),
  );

  // Store the weak ref pointer into the "shared" cell.
  weak_rc.set(decoder_weak.into_raw());
  binding::set_internal_ref(scope, decoder_wrapper, 1, weak_rc);

  rv.set(decoder_wrapper.into());
}

/// `TextDecoder.decode` API.
pub fn decode_stream<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 4);
  debug_assert!(args.get(0).is_array_buffer());
  let data = args.get(0).cast::<v8::ArrayBuffer>();
  let data: Vec<u8> = data
    .get_backing_store()
    .iter()
    .map(|b| b.get())
    .collect_vec();
  debug_assert!(is_v8_obj!(args.get(1)));
  let decoder_wrapper = args.get(1).to_object(scope).unwrap();
  let decoder =
    binding::get_internal_ref::<RefCell<Decoder>>(scope, decoder_wrapper, 0);
  let mut decoder = decoder.borrow_mut();
  debug_assert!(is_v8_bool!(args.get(2)));
  let fatal = bool::from_v8(scope, args.get(2).to_boolean(scope));
  trace!(
    "|decode_stream| data:{:?}, data:{:?}, fatal:{:?}",
    data, data, fatal
  );
  debug_assert!(is_v8_bool!(args.get(3)));
  let stream = bool::from_v8(scope, args.get(3).to_boolean(scope));
  trace!(
    "|decode_stream| data:{:?}, data:{:?}, fatal:{:?}, stream:{:?}",
    data, data, fatal, stream
  );

  decode_impl(scope, &mut rv, &mut decoder, &data, fatal, false);
}
