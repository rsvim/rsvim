//! Ex command runtime context.

use crate::buf::BufferId;
use crate::js::converter::*;
use crate::ui::tree::NodeId;
use compact_str::CompactString;

/// Command attribute name.
pub const BANG: &str = "bang";
pub const ARGS: &str = "args";
pub const CURRENT_BUFFER_ID: &str = "currentBufferId";
pub const CURRENT_WINDOW_ID: &str = "currentWindowId";

/// Default command attributes.
pub const BANG_DEFAULT: bool = false;
pub const ARGS_DEFAULT: Vec<CompactString> = vec![];
pub const CURRENT_BUFFER_ID_DEFAULT: BufferId = BufferId::negative_one();
pub const CURRENT_WINDOW_ID_DEFAULT: NodeId = NodeId::negative_one();

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandContext {
  #[builder(default = BANG_DEFAULT)]
  // bang
  pub bang: bool,

  #[builder(default = ARGS_DEFAULT)]
  pub args: Vec<CompactString>,

  #[builder(default = CURRENT_BUFFER_ID_DEFAULT)]
  pub current_buffer_id: BufferId,

  #[builder(default = CURRENT_WINDOW_ID_DEFAULT)]
  pub current_window_id: NodeId,
}

impl StructToV8 for CommandContext {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    debug_assert!(self.current_buffer_id > 0);
    debug_assert!(self.current_window_id > 0);

    // bang
    let bang_value = self.bang.to_v8(scope);
    crate::js::binding::set_property_to(scope, obj, BANG, bang_value.into());

    // args
    let args_value = self.args.to_v8(scope, |scope, i| i.to_v8(scope).into());
    crate::js::binding::set_property_to(scope, obj, ARGS, args_value.into());

    // currentBufferId
    let current_buffer_id_value = self.current_buffer_id.to_v8(scope);
    crate::js::binding::set_property_to(
      scope,
      obj,
      CURRENT_BUFFER_ID,
      current_buffer_id_value.into(),
    );

    // currentWindowId
    let current_window_id_value = self.current_window_id.to_v8(scope);
    crate::js::binding::set_property_to(
      scope,
      obj,
      CURRENT_WINDOW_ID,
      current_window_id_value.into(),
    );

    obj
  }
}
