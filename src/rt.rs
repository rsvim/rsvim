//! The JavaScript runtime.

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
  config_entry: String,
}

impl JsRuntime {
  pub fn new(config_entry: String) -> Self {
    INIT.call_once(init_v8_platform);
    let isolate = v8::Isolate::new(Default::default());

    JsRuntime {
      isolate,
      config_entry,
    }
  }

  pub async fn run(&mut self) -> Result<(), String> {}
}
