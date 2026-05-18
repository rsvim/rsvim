//! Ex command runtime context.

use crate::buf::BufferId;
use crate::js::converter::*;
use crate::ui::tree::NodeId;
use compact_str::CompactString;
use rsvim_macro::ToV8;

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

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder, ToV8)]
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
