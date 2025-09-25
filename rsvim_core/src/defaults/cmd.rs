//! Vim buffer's options default value.

use crate::buf::BufferId;
use crate::js::command::attr::Nargs;

pub const NARGS: Nargs = Nargs::Zero;
pub const BANG: bool = false;
pub const BUFFER: Option<BufferId> = None;
pub const FORCE: bool = true;
