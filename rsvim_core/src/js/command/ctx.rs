//! Ex command runtime context.

use crate::buf::BufferId;
use crate::js::converter::*;
use crate::to_v8_prop;
use crate::ui::tree::TreeNodeId;
use compact_str::CompactString;

/// Command attribute name.
pub const BANG: &str = "bang";
pub const ARGS: &str = "args";
pub const CURRENT_BUFFER_ID: &str = "currentBufferId";
pub const CURRENT_WINDOW_ID: &str = "currentWindowId";

/// Default command attributes.
pub const BANG_DEFAULT: bool = false;
pub const ARGS_DEFAULT: Vec<CompactString> = vec![];
pub const CURRENT_BUFFER_ID_DEFAULT: Option<BufferId> = None;
pub const CURRENT_WINDOW_ID_DEFAULT: Option<TreeNodeId> = None;

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandContext {
  #[builder(default = BANG_DEFAULT)]
  // bang
  pub bang: bool,

  #[builder(default = ARGS_DEFAULT)]
  pub args: Vec<CompactString>,

  #[builder(default = CURRENT_BUFFER_ID_DEFAULT)]
  pub current_buffer_id: Option<BufferId>,

  #[builder(default = CURRENT_WINDOW_ID_DEFAULT)]
  pub current_window_id: Option<BufferId>,
}

impl StructToV8 for CommandContext {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    debug_assert!(self.current_buffer_id.is_some());
    debug_assert!(self.current_window_id.is_some());

    to_v8_prop!(self, obj, scope, bang);
    to_v8_prop!(self, obj, scope, args, Vec);
    to_v8_prop!(self, obj, scope, current_buffer_id, optional);
    to_v8_prop!(self, obj, scope, current_window_id, optional);

    obj
  }
}
