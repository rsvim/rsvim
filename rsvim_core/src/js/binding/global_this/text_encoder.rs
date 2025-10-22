//! `TextEncoder` APIs.

mod decoder;
mod encoder;

use crate::is_v8_obj;
use crate::is_v8_str;
use crate::is_v8_u8array;
use crate::js::binding;
use crate::js::converter::*;
use crate::prelude::*;
use compact_str::ToCompactString;
use decoder::DecodeOptions;
use decoder::ENCODING;
use decoder::TextDecoder;
use decoder::TextDecoderOptions;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

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
  #[allow(deprecated)]
  let written = payload.write_utf8(
    scope,
    &mut buf,
    Some(&mut read),
    v8::WriteOptions::NO_NULL_TERMINATION
      | v8::WriteOptions::REPLACE_INVALID_UTF8,
  );
  trace!("|encode_utf8| written:{:?}, read:{:?}", written, read);

  let store = v8::ArrayBuffer::new_backing_store_from_vec(buf).make_shared();

  (store, read, written)
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
  debug_assert!(is_v8_str!(args.get(0)));
  let payload = args.get(0).to_string(scope).unwrap();
  trace!("|encode| payload:{:?}", payload.to_rust_string_lossy(scope));

  let (store, _read, _written) = encode_impl(scope, payload);

  debug_assert!(is_v8_u8array!(args.get(1)));
  let output = args.get(1).cast::<v8::Uint8Array>();

  let output_store = output.get_backing_store().unwrap();
  output_store.clone_from(&store);

  rv.set(buf.into());
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

  match encoding_rs::Encoding::for_label(encoding.as_bytes()) {
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
      let decoder_ptr = binding::set_internal_ref::<
        RefCell<encoding_rs::Decoder>,
      >(scope, decoder_wrapper, 0, decoder_handle);
      let weak_rc = Rc::new(Cell::new(None));

      // To automatically drop the decoder_handle instance when
      // V8 garbage collects the object that internally holds the Rust instance,
      // we use a Weak reference with a finalizer callback.
      let decoder_weak = v8::Weak::with_finalizer(
        scope,
        decoder_wrapper,
        Box::new({
          let weak_rc = weak_rc.clone();
          move |isolate| unsafe {
            drop(Box::from_raw(decoder_ptr));
            drop(v8::Weak::from_raw(isolate, weak_rc.get()));
          }
        }),
      );

      // Store the weak ref pointer into the "shared" cell.
      weak_rc.set(decoder_weak.into_raw());
      binding::set_internal_ref(scope, decoder_wrapper, 1, weak_rc);

      let encoding_value = encoding.to_v8(scope);
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
        ENCODING,
        fatal_value.into(),
      );
      let ignore_bom_value = options.ignore_bom().to_v8(scope);
      binding::set_constant_to(
        scope,
        decoder_wrapper,
        ENCODING,
        ignore_bom_value.into(),
      );

      rv.set(decoder_wrapper.into());
    }
    None => {
      binding::throw_range_error(
        scope,
        &TheErr::InvalidEncodingLabel(encoding),
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
  let decoder_wrapper = args.get(0).to_object(scope).unwrap();
  let decoder = TextDecoder::from_v8(scope, decoder_wrapper);
  let decoder_handle = binding::get_internal_ref::<RefCell<encoding_rs::Decoder>>(
    scope,
    decoder_wrapper,
    0,
  );
  let mut decoder_handle = decoder_handle.borrow_mut();

  debug_assert!(args.get(1).is_uint8_array());
  let buf = args.get(1).cast::<v8::Uint8Array>();
  let buf: Vec<u8> = buf
    .get_backing_store()
    .unwrap()
    .to_vec()
    .iter()
    .map(|c| c.get())
    .collect();

  debug_assert!(is_v8_obj!(args.get(2)));
  let options =
    DecodeOptions::from_v8(scope, args.get(2).to_object(scope).unwrap());

  let max_buffer_length = decoder_handle.max_utf16_buffer_length(buf.len());

  if max_buffer_length.is_none() {
    binding::throw_range_error(scope, &TheErr::ValueTooLarge(buf.len()));
    return;
  }

  let max_buffer_length = max_buffer_length.unwrap();
  let mut output = String::with_capacity(max_buffer_length);

  if decoder.fatal() {
    let (result, _) = decoder_handle.decode_to_string_without_replacement(
      &buf,
      &mut output,
      !options.stream(),
    );
    match result {
      encoding_rs::DecoderResult::InputEmpty => {
        let output = output.to_v8(scope);
        rv.set(output.into());
      }
      encoding_rs::DecoderResult::OutputFull => {
        binding::throw_type_error(
          scope,
          &TheErr::BufferTooSmall(max_buffer_length),
        );
      }
      encoding_rs::DecoderResult::Malformed(_, _) => {
        binding::throw_type_error(scope, &TheErr::DataInvalid);
      }
    }
  } else {
    let (result, _, _written) =
      decoder_handle.decode_to_string(&buf, &mut output, !options.stream());
    match result {
      encoding_rs::CoderResult::InputEmpty => {
        let output = output.to_v8(scope);
        rv.set(output.into());
      }
      encoding_rs::CoderResult::OutputFull => {
        binding::throw_type_error(
          scope,
          &TheErr::BufferTooSmall(max_buffer_length),
        );
      }
    }
  }
}
