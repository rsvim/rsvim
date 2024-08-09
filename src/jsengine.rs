//! The V8 Javascript engine.

use std::sync::Once;

static INIT: Once = Once::new();

fn init_v8_platform() {
  let platform = v8::new_default_platform(0, false).make_shared();
  v8::V8::initialize_platform(platform);
  v8::V8::initialize();
}

pub struct JsEngine {
  isolate: v8::OwnedIsolate,
}

impl JsEngine {
  pub fn new() -> Self {
    INIT.call_once(init_v8_platform);
    let isolate = v8::Isolate::new(Default::default());

    JsEngine { isolate }
  }
}
