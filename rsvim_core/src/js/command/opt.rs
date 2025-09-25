//! Ex command options.

/// Command option names.
pub const FORCE_NAME: &str = "force";

/// Default command options.
pub const FORCE_VALUE: bool = true;

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
pub struct CommandOptions {
  #[builder(default = FORCE_VALUE)]
  pub force: bool,
}

impl CommandOptions {
  pub fn from_object<'a>(
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
}
