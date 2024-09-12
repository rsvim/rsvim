//! The JavaScript runtime.

#![allow(dead_code)]

use std::sync::Once;

use crate::buf::BuffersArc;
use crate::result::VoidResult;
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

static INIT: Once = Once::new();

fn init_v8_platform() {
  let platform = v8::new_default_platform(0, false).make_shared();
  v8::V8::initialize_platform(platform);
  v8::V8::initialize();
}

pub struct JsRuntime {
  isolate: v8::OwnedIsolate,
  config_file: String,
}

impl JsRuntime {
  pub fn new(config_file: String) -> Self {
    INIT.call_once(init_v8_platform);
    let isolate = v8::Isolate::new(Default::default());

    JsRuntime {
      isolate,
      config_file,
    }
  }

  pub async fn run(&mut self, data_access: JsDataAccess) -> VoidResult {
    Ok(())
  }
}

#[derive(Debug)]
/// The mutable data passed to each state handler, and allow them access the editor.
pub struct JsDataAccess {
  pub state: StateArc,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
}

impl JsDataAccess {
  pub fn new(state: StateArc, tree: TreeArc, buffers: BuffersArc) -> Self {
    JsDataAccess {
      state,
      tree,
      buffers,
    }
  }
}
