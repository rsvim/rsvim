//! Sub-process command.

use crate::from_v8_prop;
use crate::is_v8_array;
use crate::js::converter::*;
use crate::prelude::*;
use crate::to_v8_prop;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::collections::HashMap;

/// Command option names.
pub const ARGS: &str = "args";
pub const CWD: &str = "cwd";
pub const CLEAR_ENV: &str = "clearEnv";
pub const ENVS: &str = "envs";
pub const STDIN: &str = "stdin";

/// Default command options.
pub const ARGS_DEFAULT: Vec<String> = vec![];
pub const CWD_DEFAULT: Option<CompactString> = None;
pub const CLEAR_ENV_DEFAULT: bool = false;
pub const ENVS_DEFAULT: HashMap<CompactString, CompactString> = HashMap::new();
pub const STDIN_DEFAULT: &str = "null";

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct ProcCommandOptions {
  #[builder(default = ARGS_DEFAULT)]
  pub args: Vec<String>,

  #[builder(default = CWD_DEFAULT)]
  pub cwd: Option<CompactString>,

  #[builder(default = CLEAR_ENV_DEFAULT)]
  pub clear_env: bool,

  #[builder(default = ENVS_DEFAULT)]
  pub envs: bool,

  #[builder(default = STDIN_DEFAULT.to_compact_string())]
  pub stdin: CompactString,
}

impl StructFromV8 for ProcCommandOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = ProcCommandOptionsBuilder::default();

    // args
    let args_name = ARGS.to_v8(scope);
    debug_assert!(
      obj
        .has_own_property(scope, args_name.into())
        .unwrap_or(false)
    );
    let args_value = obj.get(scope, args_name.into()).unwrap();
    debug_assert!(is_v8_array!(args_value));

    from_v8_prop!(builder, obj, scope, bool, force);
    from_v8_prop!(builder, obj, scope, CompactString, alias, optional);

    builder.build().unwrap()
  }
}

impl StructToV8 for CommandOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, force);
    to_v8_prop!(self, obj, scope, alias, optional);

    obj
  }
}
