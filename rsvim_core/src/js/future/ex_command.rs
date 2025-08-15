//! Vim ex command.

use crate::js::msg::{self as jsmsg, JsRuntimeToEventLoopMessage};
use crate::js::{self, JsFuture, JsFutureId, JsRuntime};
use crate::prelude::*;

use std::rc::Rc;
use std::time::Duration;

struct ExCommandFuture {
  future_id: JsFutureId,
  cb: Rc<v8::Global<v8::Function>>,
  params: Rc<Vec<v8::Global<v8::Value>>>,
}
