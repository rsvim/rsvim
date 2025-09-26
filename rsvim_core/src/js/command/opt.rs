//! Ex command options.

/// Command option names.
pub const FORCE_NAME: &str = "force";
pub const ALIAS_NAME: &str = "alias";

/// Default command options.
pub const FORCE_VALUE: bool = true;
pub const ALIAS_VALUE: Option<String> = None;

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
pub struct CommandOptions {
  #[builder(default = FORCE_VALUE)]
  pub force: bool,
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

    builder.build().unwrap()
  }

  pub fn into_v8_object<'a>(
    &self,
    scope: &mut v8::HandleScope<'a>,
  ) -> v8::Local<'a, v8::Object> {
    let obj = v8::Object::new(scope);

    // force
    let force_field = v8::String::new(scope, "force").unwrap();
    let force_value = v8::Boolean::new(scope, self.force);
    obj.set(scope, force_field.into(), force_value.into());

    obj
  }
}
