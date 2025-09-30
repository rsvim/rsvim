//! Converters between rust and v8 values.

pub trait ToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value>;
}

pub trait FromV8CallbackArguments {
  fn from_v8_callback_arguments(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
  ) -> Self;
}

pub trait FromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self;
}

pub fn to_v8<'s, 'b, T>(
  scope: &mut v8::PinScope<'s, 'b>,
  input: T,
) -> v8::Local<'s, v8::Value>
where
  T: ToV8,
{
}
