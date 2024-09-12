//! The JavaScript runtime.

#![allow(dead_code)]

use std::sync::Once;
use tokio::fs;
use tracing::{debug, error};

use crate::buf::BuffersArc;
use crate::result::{ErrorCode, VoidResult};
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

static INIT: Once = Once::new();

fn into_str(buf: &[u8], bufsize: usize) -> String {
  String::from_utf8_lossy(&buf[0..bufsize]).into_owned()
}

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

  pub async fn start(&mut self, data_access: JsDataAccess) -> VoidResult {
    let scope = &mut v8::HandleScope::new(&mut self.isolate);

    // Create the `vim` global object {

    let vim_obj = v8::ObjectTemplate::new(scope);
    vim_obj.set_accessor_property(v8::String::new(scope, "vim").unwrap().into(), value);

    // Create the `vim` global object }

    let context = v8::Context::new(scope, Default::default());
    let scope = &mut v8::ContextScope::new(scope, context);

    debug!("Load config file {:?}", self.config_file.as_str());
    match fs::read_to_string(self.config_file.as_str()).await {
      Ok(source) => {}
      Err(e) => {
        let msg = format!(
          "Failed to load user config file {:?} with error {:?}",
          self.config_file.as_str(),
          e
        );
        error!("{msg}");
        return Err(ErrorCode::Message(msg));
      }
    }

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
