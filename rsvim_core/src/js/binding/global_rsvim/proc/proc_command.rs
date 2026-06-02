//! Child-process command options.

use crate::js::JsFuture;
use crate::js::binding;
use crate::js::converter::*;
use crate::js::resource::ResourceId;
use crate::js::resource::ResourceTableArc;
use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::str::FromStr;

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Hash,
  strum_macros::Display,
  strum_macros::EnumString,
)]
pub enum Stdio {
  #[strum(serialize = "null")]
  Null,

  #[strum(serialize = "piped")]
  Piped,

  #[strum(serialize = "inherit")]
  Inherit,
}

impl FromV8 for Stdio {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(value.is_string() || value.is_string_object());
    let result = value.to_string(scope).unwrap().to_rust_string_lossy(scope);
    Stdio::from_str(&result).unwrap()
  }
}

impl ToV8 for Stdio {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    self.to_string().to_v8(scope)
  }
}

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
  #[builder(default = vec![])]
  pub args: Vec<CompactString>,

  #[builder(default = None)]
  pub cwd: Option<CompactString>,

  #[builder(default = false)]
  pub clear_env: bool,

  #[builder(default = false)]
  pub detached: bool,

  #[builder(default = FoldMap::new())]
  pub env: FoldMap<CompactString, CompactString>,

  #[builder(default = Stdio::Null)]
  pub stdin: Stdio,

  #[builder(default = Stdio::Piped)]
  pub stdout: Stdio,

  #[builder(default = Stdio::Piped)]
  pub stderr: Stdio,
}

pub struct SpawnChildProcessFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for SpawnChildProcessFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|SpawnChildProcessFuture|");

    let result = self.maybe_result.take().unwrap();

    // Handle when something goes wrong with opening the file.
    if let Err(e) = result {
      let message = v8::String::new(scope, &e.to_string()).unwrap();
      let exception = v8::Exception::error(scope, message);
      binding::set_exception_code(scope, exception, &e);
      self.promise.open(scope).reject(scope, exception);
      return;
    }

    // Otherwise, get the result and deserialize it.
    let result = result.unwrap();

    // Deserialize bytes into a file-descriptor.
    let file_rid = postcard::from_bytes::<ResourceId>(&result).unwrap();
    let file_rid = Into::<i32>::into(file_rid);
    let file_rid = file_rid.to_v8(scope);

    self.promise.open(scope).resolve(scope, file_rid).unwrap();
  }
}

pub fn spawn_child_process(
  resource_table: ResourceTableArc,
  exec_path: &CompactString,
  options: &ProcCommandOptions,
) -> TheResult<ResourceId> {
  let mut command = std::process::Command::new(exec_path);

  command.args(options.args.clone().into_iter());
  if let Some(cwd) = &options.cwd {
    command.current_dir(cwd);
  }
  if options.clear_env {
    command.env_clear();
  }
  command.envs(
    options
      .env
      .clone()
      .into_iter()
      .map(|(k, v)| (k.into_string(), v)),
  );
  command.stdin(match options.stdin {
    Stdio::Null => std::process::Stdio::null(),
    Stdio::Piped => std::process::Stdio::piped(),
    Stdio::Inherit => std::process::Stdio::inherit(),
  });
  command.stdout(match options.stdout {
    Stdio::Null => std::process::Stdio::null(),
    Stdio::Piped => std::process::Stdio::piped(),
    Stdio::Inherit => std::process::Stdio::inherit(),
  });
  command.stderr(match options.stderr {
    Stdio::Null => std::process::Stdio::null(),
    Stdio::Piped => std::process::Stdio::piped(),
    Stdio::Inherit => std::process::Stdio::inherit(),
  });

  match command.spawn() {
    Ok(child) => {}
    Err(e) => {
      let cmd = if !options.args.is_empty() {
        format!("{} {}", exec_path, options.args.join(" ")).to_compact_string()
      } else {
        exec_path.to_compact_string()
      };
      return Err(TheErr::SpawnChildProcessFailed(cmd, e.to_compact_string()));
    }
  }

  Ok(ResourceId::next())
}
