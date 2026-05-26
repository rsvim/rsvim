//! Child-process command options.

use crate::js::converter::*;
use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use itertools::Itertools;

/// Command option names.
pub const ARGS: &str = "args";
pub const CWD: &str = "cwd";
pub const CLEAR_ENV: &str = "clearEnv";
pub const ENVS: &str = "envs";
pub const STDIN: &str = "stdin";

/// Default command options.
pub const CWD_DEFAULT: Option<CompactString> = None;
pub const CLEAR_ENV_DEFAULT: bool = false;
pub const STDIN_DEFAULT: &str = "null";

#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  derive_builder::Builder,
  rsvim_macro::ToV8,
  rsvim_macro::FromV8,
)]
pub struct ProcCommandOptions {
  #[builder(default = Vec::new())]
  pub args: Vec<CompactString>,

  #[builder(default = CWD_DEFAULT)]
  pub cwd: Option<CompactString>,

  #[builder(default = CLEAR_ENV_DEFAULT)]
  pub clear_env: bool,

  #[builder(default = FoldMap::new())]
  pub envs: FoldMap<CompactString, CompactString>,

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
    debug_assert!(args_value.is_array());
    let args = match v8::Local::<v8::Array>::try_from(args_value) {
      Ok(args_array) => (0..args_array.length())
        .map(|i| {
          let arg = args_array.get_index(scope, i).unwrap();
          arg.to_rust_string_lossy(scope).to_compact_string()
        })
        .collect_vec(),
      Err(_) => unreachable!(),
    };
    builder.args(args);

    from_v8_prop!(builder, obj, scope, CompactString, cwd, optional);
    from_v8_prop!(builder, obj, scope, bool, clear_env);

    // envs
    let envs_name = ENVS.to_v8(scope);
    debug_assert!(
      obj
        .has_own_property(scope, envs_name.into())
        .unwrap_or(false)
    );
    let envs_value = obj.get(scope, envs_name.into()).unwrap();
    debug_assert!(envs_value.is_object());
    let envs_value = envs_value.to_object(scope).unwrap();
    let envs = match envs_value.get_property_names(
      scope,
      v8::GetPropertyNamesArgsBuilder::new()
        .mode(v8::KeyCollectionMode::OwnOnly)
        .build(),
    ) {
      Some(keys_array) => {
        let mut envs: FoldMap<CompactString, CompactString> =
          FoldMap::with_capacity(keys_array.length() as usize);

        for i in 0..keys_array.length() {
          let k = match keys_array.get_index(scope, i) {
            Some(k) => k,
            None => continue,
          };
          let v = match envs_value.get(scope, k) {
            Some(v) => v,
            None => continue,
          };
          envs.insert(
            k.to_rust_string_lossy(scope).to_compact_string(),
            v.to_rust_string_lossy(scope).to_compact_string(),
          );
        }
        envs
      }
      None => FoldMap::new(),
    };
    builder.envs(envs);

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
