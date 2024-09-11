//! The V8 javascript runtime.

#![allow(dead_code)]

use std::sync::Once;

static INIT: Once = Once::new();

fn init_v8_platform() {
  let platform = v8::new_default_platform(0, false).make_shared();
  v8::V8::initialize_platform(platform);
  v8::V8::initialize();
}

pub struct JsRuntime {
  isolate: v8::OwnedIsolate,
}

impl JsRuntime {
  pub fn new() -> Self {
    INIT.call_once(init_v8_platform);
    let isolate = v8::Isolate::new(Default::default());

    JsRuntime { isolate }
  }
}

impl Default for JsRuntime {
  fn default() -> Self {
    JsRuntime::new()
  }
}
