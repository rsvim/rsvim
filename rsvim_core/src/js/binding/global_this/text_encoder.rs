//! `TextEncoder` APIs.

mod decoder;
mod encoder;

use crate::is_v8_obj;
use crate::is_v8_str;
use crate::js::binding;
use crate::js::converter::*;
use crate::prelude::*;
use compact_str::ToCompactString;
use decoder::DecodeOptions;
use decoder::ENCODING;
use decoder::FATAL;
use decoder::IGNORE_BOM;
use decoder::TextDecoder;
use decoder::TextDecoderOptions;
use encoder::EncodeIntoResultBuilder;
use encoding_rs::CoderResult;
use encoding_rs::Decoder;
use encoding_rs::DecoderResult;
use encoding_rs::Encoding;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

// Returns v8 BackingStore data, read (chars), written (bytes)
fn encode_impl<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  payload: v8::Local<'s, v8::String>,
) -> (Vec<u8>, usize, usize) {
  let mut buf: Vec<u8> = vec![];
  let mut read: usize = 0;

  // FIXME: Update to `write_utf8_v8` follow deno's implementation:
  // https://github.com/denoland/deno/blob/v2.5.4/ext/web/08_text_encoding.js#L256
  // https://github.com/denoland/deno/blob/v2.5.4/ext/web/lib.rs#L367
  #[allow(deprecated)]
  let written = payload.write_utf8(
    scope,
    &mut buf,
    Some(&mut read),
    v8::WriteOptions::NO_NULL_TERMINATION
      | v8::WriteOptions::REPLACE_INVALID_UTF8,
  );
  trace!("|encode_impl| written:{:?}, read:{:?}", written, read);

  (buf, read, written)
}

/// `TextEncoder.encode` API.
pub fn encode<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(args.get(0).is_array_buffer());
  debug_assert!(is_v8_str!(args.get(1)));
  let payload = args.get(0).to_string(scope).unwrap();
  trace!("|encode| payload:{:?}", payload.to_rust_string_lossy(scope));

  let (data, _read, _written) = encode_impl(scope, payload);

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
  if !args.get(1).is_array_buffer_view() {
    binding::throw_type_error(scope, &TheErr::BufferInvalid);
    return;
  }

  let payload = args.get(0).to_string(scope).unwrap();
  trace!(
    "|encode_into| payload:{:?}",
    payload.to_rust_string_lossy(scope)
  );

  let (data, read, written) = encode_impl(scope, payload);
  let store = v8::ArrayBuffer::new_backing_store_from_vec(data).make_shared();

  let output = args.get(1).cast::<v8::ArrayBufferView>();

  let mut output_store = output.get_backing_store().unwrap();
  output_store.clone_from(&store);

  let encode_result = EncodeIntoResultBuilder::default()
    .read(read as u32)
    .written(written as u32)
    .build()
    .unwrap();
  let encode_result = encode_result.to_v8(scope);

  rv.set(encode_result.into());
}

/// `new TextDecoder()` API.
pub fn create_decoder<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_str!(args.get(0)));
  let encoding_label =
    args.get(0).to_rust_string_lossy(scope).to_compact_string();

  debug_assert!(is_v8_obj!(args.get(1)));
  let options =
    TextDecoderOptions::from_v8(scope, args.get(1).to_object(scope).unwrap());

  match Encoding::for_label(encoding_label.as_bytes()) {
    Some(coding) => {
      let decoder_handle = if options.ignore_bom() {
        RefCell::new(coding.new_decoder_without_bom_handling())
      } else {
        RefCell::new(coding.new_decoder_with_bom_removal())
      };

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
          let _encoding_label = encoding_label.clone();
          move |isolate| unsafe {
            drop(Box::from_raw(decoder_ptr));
            drop(v8::Weak::from_raw(isolate, weak_rc.get()));
            trace!(
              "|create_decoder| dropped TextDecoder:{:?}",
              _encoding_label
            );
          }
        }),
      );

      // Store the weak ref pointer into the "shared" cell.
      weak_rc.set(decoder_weak.into_raw());
      binding::set_internal_ref(scope, decoder_wrapper, 1, weak_rc);

      let encoding_value = encoding_label.to_v8(scope);
      binding::set_constant_to(
        scope,
        decoder_wrapper,
        ENCODING,
        encoding_value.into(),
      );
      let fatal_value = options.fatal().to_v8(scope);
      binding::set_constant_to(
        scope,
        decoder_wrapper,
        FATAL,
        fatal_value.into(),
      );
      let ignore_bom_value = options.ignore_bom().to_v8(scope);
      binding::set_constant_to(
        scope,
        decoder_wrapper,
        IGNORE_BOM,
        ignore_bom_value.into(),
      );

      rv.set(decoder_wrapper.into());
    }
    None => {
      binding::throw_range_error(
        scope,
        &TheErr::InvalidEncodingLabel(encoding_label),
      );
    }
  }
}

/// `TextDecoder.decode` API.
pub fn decode<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 3);
  debug_assert!(is_v8_obj!(args.get(0)));
  if !args.get(1).is_array_buffer_view() {
    binding::throw_type_error(scope, &TheErr::BufferInvalid);
    return;
  }

  let decoder_wrapper = args.get(0).to_object(scope).unwrap();
  let decoder_obj = TextDecoder::from_v8(scope, decoder_wrapper);
  let decoder_handle =
    binding::get_internal_ref::<RefCell<Decoder>>(scope, decoder_wrapper, 0);
  let mut decoder_handle = decoder_handle.borrow_mut();

  let buf = args.get(1).cast::<v8::ArrayBufferView>();
  let mut storage: Vec<u8> =
    Vec::with_capacity(v8::TYPED_ARRAY_MAX_SIZE_IN_HEAP);
  let buf = buf.get_contents(storage.as_mut_slice());

  debug_assert!(is_v8_obj!(args.get(2)));
  let options =
    DecodeOptions::from_v8(scope, args.get(2).to_object(scope).unwrap());
  trace!(
    "|decode| decoder_obj:{:?}, buf:{:?}, options:{:?}",
    decoder_obj, buf, options
  );

  let max_buffer_length = decoder_handle.max_utf16_buffer_length(buf.len());

  if max_buffer_length.is_none() {
    binding::throw_range_error(scope, &TheErr::ValueTooLarge(buf.len()));
    return;
  }

  let max_buffer_length = max_buffer_length.unwrap();
  let mut output = vec![0; max_buffer_length];

  if decoder_obj.fatal() {
    let (result, _read, written) = decoder_handle
      .decode_to_utf16_without_replacement(buf, &mut output, !options.stream());
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
      decoder_handle.decode_to_utf16(buf, &mut output, !options.stream());
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
