//! `TextEncoder` APIs.

use crate::get_cppgc_handle;
use crate::is_v8_bool;
use crate::is_v8_str;
use crate::js::binding;
use crate::js::converter::*;
use crate::prelude::*;
use crate::wrap_cppgc_handle;
use compact_str::ToCompactString;
use encoding_rs::CoderResult;
use encoding_rs::Decoder;
use encoding_rs::DecoderResult;
use encoding_rs::Encoding;
use itertools::Itertools;
use std::cell::RefCell;

// Returns v8 BackingStore data, read (chars), written (bytes)
fn encode_impl<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  payload: v8::Local<'s, v8::String>,
  bufsize: usize,
) -> (Vec<u8>, usize, usize) {
  let mut buf: Vec<u8> = Vec::with_capacity(bufsize);
  let mut read: usize = 0;

  let written = payload.write_utf8_uninit_v2(
    scope,
    // Step-1: Get uninit buffer inside vec
    buf.spare_capacity_mut(),
    v8::WriteFlags::kReplaceInvalidUtf8,
    Some(&mut read),
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

  let result = v8::Object::new(scope);
  let read_value = (read as u32).to_v8(scope);
  binding::set_property_to(scope, result, "read", read_value.into());
  let written_value = (written as u32).to_v8(scope);
  binding::set_property_to(scope, result, "written", written_value.into());

  rv.set(result.into());
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
  debug_assert!(args.length() == 4);
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

  let decoder_wrapper =
    wrap_cppgc_handle!(scope, decoder_handle, RefCell<Decoder>);

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
  debug_assert!(args.get(1).is_object());
  let decoder_wrapper = args.get(1).to_object(scope).unwrap();
  let decoder = get_cppgc_handle!(scope, decoder_wrapper, RefCell<Decoder>);
  let mut decoder = decoder.borrow_mut();
  debug_assert!(is_v8_bool!(args.get(2)));
  let fatal = bool::from_v8(scope, args.get(2).to_boolean(scope));
  debug_assert!(is_v8_bool!(args.get(3)));
  let stream = bool::from_v8(scope, args.get(3).to_boolean(scope));
  trace!(
    "|decode_stream| data:{:?}, fatal:{:?}, stream:{:?}",
    data, fatal, stream
  );

  decode_impl(scope, &mut rv, &mut decoder, &data, fatal, stream);
}
