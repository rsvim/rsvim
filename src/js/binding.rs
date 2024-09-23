//! Js runtime bindings.

/// Adds a read-only property with the given name and value, into the given object.
pub fn set_constant_to(
  scope: &mut v8::HandleScope<'_>,
  target: v8::Local<v8::Object>,
  name: &str,
  value: v8::Local<v8::Value>,
) {
  let key = v8::String::new(scope, name).unwrap();
  target.define_own_property(scope, key.into(), value, v8::PropertyAttribute::READ_ONLY);
}

/// Adds a `Function` object which calls the given Rust function
pub fn set_function_to(
  scope: &mut v8::HandleScope<'_>,
  target: v8::Local<v8::Object>,
  name: &'static str,
  callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
  let key = v8::String::new(scope, name).unwrap();
  let template = v8::FunctionTemplate::new(scope, callback);
  let val = template.get_function(scope).unwrap();

  target.set(scope, key.into(), val.into());
}

/// Creates an object with a given name under a `target` object.
/// Returns the created object.
pub fn create_object_under<'s>(
  scope: &mut v8::HandleScope<'s>,
  target: v8::Local<v8::Object>,
  name: &'static str,
) -> v8::Local<'s, v8::Object> {
  let template = v8::ObjectTemplate::new(scope);
  let key = v8::String::new(scope, name).unwrap();
  let value = template.new_instance(scope).unwrap();

  target.set(scope, key.into(), value.into());
  value
}

/// Populates a new JavaScript context with low-level Rust bindings.
pub fn create_new_context<'s>(scope: &mut v8::HandleScope<'s, ()>) -> v8::Local<'s, v8::Context> {
  // Here we need an EscapableHandleScope so V8 doesn't drop the
  // newly created HandleScope on return.(https://v8.dev/docs/embed#handles-and-garbage-collection)
  let scope = &mut v8::EscapableHandleScope::new(scope);

  // Create and enter a new JavaScript context.
  let context = v8::Context::new(scope, Default::default());
  let global = context.global(scope);
  let scope = &mut v8::ContextScope::new(scope, context);

  // set_function_to(scope, global, "print", global_print);
  // set_function_to(scope, global, "reportError", global_report_error);
  // set_function_to(scope, global, "$$queueMicro", global_queue_micro);

  // Expose low-level functions to JavaScript.
  // process::initialize(scope, global);

  scope.escape(context)
}

/// Sets error code to exception if possible.
pub fn set_exception_code(
  scope: &mut v8::HandleScope<'_>,
  exception: v8::Local<v8::Value>,
  error: &Error,
) {
  let exception = exception.to_object(scope).unwrap();
  if let Some(error) = error.downcast_ref::<IoError>() {
    if let Some(code) = extract_error_code(error) {
      let key = v8::String::new(scope, "code").unwrap();
      let value = v8::String::new(scope, &format!("ERR_{code}")).unwrap();
      exception.set(scope, key.into(), value.into());
    }
  }
}

/// Useful utility to throw v8 exceptions.
pub fn throw_exception(scope: &mut v8::HandleScope, error: &Error) {
  let message = error.to_string().to_owned();
  let message = v8::String::new(scope, &message).unwrap();
  let exception = v8::Exception::error(scope, message);
  set_exception_code(scope, exception, error);
  scope.throw_exception(exception);
}

/// Useful utility to throw v8 type errors.
pub fn throw_type_error(scope: &mut v8::HandleScope, message: &str) {
  let message = v8::String::new(scope, message).unwrap();
  let exception = v8::Exception::type_error(scope, message);
  scope.throw_exception(exception);
}
