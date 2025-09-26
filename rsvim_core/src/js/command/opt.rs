//! Ex command options.

use crate::buf::BufferId;

/// Command option names.
pub const FORCE_NAME: &str = "force";
pub const BUFFER_NAME: &str = "buffer";

/// Default command options.
pub const FORCE_VALUE: bool = true;
pub const BUFFER_VALUE: Option<BufferId> = None;

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
pub struct CommandOptions {
  #[builder(default = FORCE_VALUE)]
  pub force: bool,

  #[builder(default = BUFFER_VALUE)]
  pub buffer: Option<BufferId>,
}

impl CommandOptions {
  pub fn from_v8_object<'a>(
    scope: &mut v8::HandleScope,
    value: v8::Local<'a, v8::Object>,
  ) -> Self {
    let mut builder = CommandOptionsBuilder::default();

    // force
    let force_name = v8::String::new(scope, FORCE_NAME).unwrap();
    match value.get(scope, force_name.into()) {
      Some(force_value) => {
        let force = force_value.to_boolean(scope).boolean_value(scope);
        builder.force(force);
      }
      None => { /* do nothing */ }
    }

    // buffer
    let buffer_name = v8::String::new(scope, BUFFER_NAME).unwrap();
    match value.get(scope, buffer_name.into()) {
      Some(buffer_value) => {
        if buffer_value.is_int32() {
          let buf_id = buffer_value.to_int32(scope).unwrap().value();
          builder.buffer(Some(buf_id));
        }
      }
      None => { /* do nothing */ }
    }

    builder.build().unwrap()
  }

  pub fn into_v8_object<'a>(
    &self,
    scope: &mut v8::HandleScope<'a>,
  ) -> v8::Local<'a, v8::Object> {
    let obj = v8::Object::new(scope);

    // internal fields
    {
      // bang
      let attr_bang_field = v8::String::new(scope, "bang").unwrap();
      let attr_bang_value = v8::Boolean::new(scope, self.bang);
      obj.set(scope, attr_bang_field.into(), attr_bang_value.into());

      // nargs
      let attr_nargs_field = v8::String::new(scope, "nargs").unwrap();
      let attr_nargs_value =
        v8::String::new(scope, &self.nargs.to_string()).unwrap();
      obj.set(scope, attr_nargs_field.into(), attr_nargs_value.into());
    }

    obj
  }
}
